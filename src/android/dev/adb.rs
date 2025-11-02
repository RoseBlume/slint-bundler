use std::process::Command;
use std::path::Path;
use std::env;
// "C:\Users\James\AppData\Local\Android\Sdk\platform-tools\adb.exe"
fn find_adb() -> String {
    let android_sdk = env::var("ANDROID_HOME").expect("ANDROID_HOME not set");
    format!("{}\\platform-tools\\adb.exe", android_sdk)
}

pub fn start_server() {

}

pub fn list_devices() {
    let adb = find_adb();
    let output = Command::new(adb)
    .args(["devices"])
    ;
}

pub fn perform_streamed_install() {
    let adb = find_adb();
    let output = Command::new(adb)
    .args(["install", "android/app/build/outputs/apk/debug/app-debug.apk"])
    .status();

}

