mod buildtools;

use std::fs::File;
use std::io::BufReader;
use serde_json;
use serde::{Deserialize, Serialize};
use std::io::Read;
pub use buildtools::find_build_tools;





#[derive(Deserialize, Serialize)]
struct AppConfig {
    app_id: String,
    project_name: String,
    version: String,
    package_name: String
}


pub fn read_package_metadata() -> (String, String, String, String) {
    let file = File::open("slint-app.json").unwrap();

    // read entire file into a String and produce a &str for serde_json::from_str
    let mut file = file; // make mutable so we can read from it
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let reader: &str = &contents;

    let config: AppConfig = serde_json::from_str(reader).unwrap();
    (config.app_id, config.project_name, config.version, config.package_name)
}