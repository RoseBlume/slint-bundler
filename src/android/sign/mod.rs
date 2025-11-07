#[cfg(target_os="windows")]
mod windows;
mod key;
use std::env;
use std::process::Command;

use std::io::{self, Write};
use key::get_distinguished_names;
use crate::utils::{read_package_metadata};
use crate::utils::find_build_tools;
use crate::help::generate_help_message;
// #[cfg(target_os="windows")]
// use windows::KEYTOOL;

pub fn handle_sign(args: &[String]) {
    match args[3].as_str() {
        "generate" => generate_key(&args[4]),
        "sign" => sign_bundle(args[4].clone()),
        _ => println!("{}", generate_help_message(&args))
    }
}
fn generate_key(keyfile: &str) {

    let mut alias = String::new();
    print!("Please enter your keystore Alias: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut alias).expect("Failed to read line");
    alias = alias.trim().to_string();

    let dname = &get_distinguished_names();

    let _ = Command::new("keytool")
                .args(
                    [
                        "-genkey", 
                        "-alias", 
                        &alias, 
                        "-keyalg", 
                        "RSA", 
                        "-keystore", 
                        &keyfile, 
                        "-keysize", 
                        "2048",
                        "-dname",
                        dname
                    ]
                )
                .status();
}


fn sign_bundle(keystore: String) {
    let (_app_id, _project_name, _version, package_name) = read_package_metadata();
    let apk_signer = format!("{}\\apksigner.bat", find_build_tools());
    println!("{}", apk_signer);
    let current_path: String = env::var("PATH").unwrap_or_default();
    env::set_var("PATH", format!("{};{}", current_path, apk_signer));
    let apk_path = "android\\app\\build\\outputs\\apk\\release\\app-release-unsigned.apk";
    let output_path = format!("{}-signed.apk", package_name);
    let _ = Command::new(apk_signer)
                .args(["sign", "--ks", keystore.as_str(), "--in", apk_path, "--out", &output_path])
                .status();
}