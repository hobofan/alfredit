use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct PrefContents {
    custom_sites: LinkedHashMap<String, CustomSite>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct CustomSite {
    enabled: bool,
    keyword: String,
    text: String,
    url: String,
    utf8: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CsvLine {
    key: String,
    enabled: bool,
    keyword: String,
    text: String,
    url: String,
    utf8: bool,
}

fn map_to_lines(mut map: LinkedHashMap<String, CustomSite>) -> Vec<CsvLine> {
    map.entries()
        .map(|entry| {
            let key = entry.key().to_owned();
            let value = entry.get().to_owned();
            CsvLine {
                key,
                enabled: value.enabled,
                keyword: value.keyword,
                text: value.text,
                url: value.url,
                utf8: value.utf8,
            }
        })
        .collect()
}

fn lines_to_map(lines: Vec<CsvLine>) -> LinkedHashMap<String, CustomSite> {
    lines
        .into_iter()
        .map(|line| {
            (
                line.key,
                CustomSite {
                    enabled: line.enabled,
                    keyword: line.keyword,
                    text: line.text,
                    url: line.url,
                    utf8: line.utf8,
                },
            )
        })
        .collect()
}

fn main() {
    #[allow(non_snake_case)]
    let ALFRED_PREFERNCES_PATH: PathBuf = dirs::home_dir().unwrap().join("./Library/Application Support/Alfred/Alfred.alfredpreferences/preferences/features/websearch/prefs.plist");

    let mut contents: PrefContents = plist::from_file(&ALFRED_PREFERNCES_PATH).unwrap();

    let lines = map_to_lines(contents.custom_sites.clone());

    let mut child = Command::new("vipe")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");

    {
        let mut wtr = csv::Writer::from_writer(child.stdin.as_mut().expect("Failed to open stdin"));
        for line in lines {
            wtr.serialize(line).unwrap();
        }
        wtr.flush().unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read stdout");
    let mut rdr = csv::Reader::from_reader(output.stdout.as_slice());
    let lines: Vec<CsvLine> = rdr.deserialize().collect::<Result<Vec<_>, _>>().unwrap();

    contents.custom_sites = lines_to_map(lines);
    plist::to_file_xml(&ALFRED_PREFERNCES_PATH, &contents).unwrap();
}
