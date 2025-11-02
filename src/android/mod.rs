mod init;
mod tools;
mod build;
mod dev;
mod sign;
use init::{initialize_android_project, create_jni_dirs};
use tools::unpack_gradle_jar;
use sign::handle_sign;

pub use build::begin_build;
use dev::handle_dev;
use std::fs;
// pub const GRADLE_VERSION: &str = "8.9";
// pub const BUNDLE_TOOL_VERSION: &str = "1.18.2";
pub fn handle_android(args: &[String]) {
    match args[2].as_str() {
        "init" => {
            initialize_android_project().expect("Failed to initialize android project");
            create_jni_dirs();
            unpack_gradle_jar();

        },
        "build" => begin_build("--release"),
        "dev" => handle_dev(),
        "key" => handle_sign(args),
        _ => println!("Invalid option")
    }
    //unpack_gradle_jar(GRADLE_WRAPPER_PATH);
    //begin_build();
}


