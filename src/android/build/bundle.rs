use std::process::Command;
use std::path::{Path};
use std::env;
use std::fs;
pub fn begin_gradle_build() {
    println!("Beginning gradle build");

    let batch_file = r"android\\gradlew.bat build --project-dir android";
    let cmd = format!("gradlew");
    match Command::new("cmd")
        .args(&["/C", batch_file])
        // .arg("build")
        // .arg("--project-dir")
        // .arg("android")
        
        .status()
    {
        Ok(status) => {
            if status.success() {
                println!("Gradle build completed successfully");
            } else {
                match status.code() {
                    Some(code) => eprintln!("Gradle build failed with exit code: {}", code),
                    None => eprintln!("Gradle build failed and was terminated by signal"),
                }
            }
        }
        Err(err) => eprintln!("Failed to spawn gradle process: {}", err),
    }
}