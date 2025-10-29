mod init;
mod tools;
mod build;
mod dev;
use init::{initialize_android_project, create_jni_dirs};
use tools::unpack_gradle_jar;

pub use build::begin_build;
use dev::handle_dev;
use std::fs;
// pub const GRADLE_VERSION: &str = "8.9";
// pub const BUNDLE_TOOL_VERSION: &str = "1.18.2";
pub fn handle_android(arg: &str) {
    match arg {
        "init" => {
            initialize_android_project().expect("Failed to initialize android project");
            create_jni_dirs();
            unpack_gradle_jar();

        },
        "build" => begin_build("--release"),
        "dev" => handle_dev(),
        &_ => println!("Invalid option")
    }
    //unpack_gradle_jar(GRADLE_WRAPPER_PATH);
    //begin_build();
}

pub fn read_package_metadata() -> (String, String) {
    let manifest = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");
    let package_name = manifest
        .lines()
        .find_map(|line| {
            if line.trim_start().starts_with("name") {
                line.split('=').nth(1).map(|v| v.trim().trim_matches('"').to_string())
            } else {
                None
            }
        })
        .expect("Could not find package name in Cargo.toml");

    let version = manifest
        .lines()
        .find_map(|line| {
            if line.trim_start().starts_with("version") {
                line.split('=').nth(1).map(|v| v.trim().trim_matches('"').to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "0.1.0".to_string());

    (package_name, version)
}


