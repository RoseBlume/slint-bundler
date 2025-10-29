use std::process::Command;
use std::fs;
use std::path::{Path, PathBuf};
use crate::bundle::windows::{read_package_metadata, prettify_package_name};

/*
const NSIS_PATHS: &[&str] = &[
    "C:\\Program Files (x86)\\NSIS",
    "C:\\Program Files\\NSIS"
];
*/


pub fn bundle_nsis() {
    println!("Creating NSIS installer...");

    let (package_name, version) = read_package_metadata();
    let release_bin = Path::new("target").join("release").join(format!("{}.exe", package_name));

    if !release_bin.exists() {
        eprintln!("Release binary not found at {}. Run `cargo build --release` first.", release_bin.display());
        return;
    }

    let out_dir = Path::new("target").join("release").join("bundle").join("nsis");
    fs::create_dir_all(&out_dir).expect("failed to create output dir");
    let out = out_dir.join(format!("{}_{}.exe", package_name, version));
    if which::which("makensis.exe").is_err() {
        eprintln!("makensis.exe not found in PATH. Skipping NSIS build.");
        return;
    }
    let ico = Path::new("icons").join("icon.ico");
    let icon = Path::new("target").join("release").join("bundle").join("nsis").join(format!("{}_{}.ico", package_name, version));

    fs::copy(
        PathBuf::from(ico.clone()),
        PathBuf::from(icon.clone()),
    ).expect("Failed to copy");
    let out_path = fs::canonicalize(&out).expect("");
    let bin_path = fs::canonicalize(&release_bin).expect("");
    let icon_path = fs::canonicalize(&icon).expect("");
    /*
    

    // Define the destination file path
    let mut file = fs::File::create(&out_path);
    let destination = fs::canonicalize(&out_path).expect("");
    println!("{}", destination.display());

    // Copy the file
    fs::copy(
        PathBuf::from(bin_path.clone()),//.clone().expect("Failed to convert bin_path to PathBuf").to_path_buf(), 
        PathBuf::from(destination),//.clone().expect("Failed to convert destination to PathBuf").to_path_buf()
    ).expect("Failed to copy");
    */
    let _ = fs::File::create(&out_path);
    // Create a temporary NSIS script
    let tmp = out_dir;
    let nsis_script = tmp.join("installer.nsi");
    let script_content = format!(
        r#"!include "MUI.nsh"
!define APP_NAME "{pretty_name}"
!define VERSION "{version}"
!define MUI_ICON "{ico}"
!insertmacro MUI_LANGUAGE "English"
OutFile "{out}"
InstallDir "$PROGRAMFILES\{name}"
Page Directory
Page InstFiles
InstallDirRegKey HKCU "Software\Modern UI Test" ""

Var StartMenuFolder
RequestExecutionLevel admin
!define MUI_STARTMENUPAGE_REGISTRY_ROOT "HKLM" 
!define MUI_STARTMENUPAGE_REGISTRY_KEY "Software\{name}" 
!define MUI_STARTMENUPAGE_REGISTRY_VALUENAME "Start Menu Folder"

Section
    SetOutPath "$INSTDIR"
    File "{bin}"
    File "{ico}"
    CreateDirectory "$SMPROGRAMS\$StartMenuFolder"
    CreateShortCut "$DESKTOP\{pretty_name}.lnk" "$INSTDIR\{exe}" "" "$INSTDIR\{name}_{version}.ico"
    CreateShortCut "$SMPROGRAMS\$StartMenuFolder\{pretty_name}.lnk" "$INSTDIR\{exe}" "" "$INSTDIR\{name}_{version}.ico"
    WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\{exe}"
    Delete "$INSTDIR\{name}_{version}.ico"
    Delete "$INSTDIR\uninstall.exe"
    Delete "$DESKTOP\{name}.lnk"
    Delete "$SMPROGRAMS\$StartMenuFolder\{name}.lnk"
    RMDir "$SMPROGRAMS\$StartMenuFolder"
    RMDir "$INSTDIR"
    DeleteRegKey HKLM "Software\{name}"
SectionEnd
        "#,
        name = package_name,
        pretty_name = prettify_package_name(&package_name),
        version = version,
        out = out_path.display(),
        bin = bin_path.display(),
        exe = release_bin.file_name().unwrap().to_string_lossy(),
        ico = icon_path.display()
    );
    fs::write(&nsis_script, script_content).expect("failed to write NSIS script");
    let status = Command::new("makensis.exe")
        .arg(&nsis_script)
        .status()
        .expect("failed to run makensis.exe");
    if !status.success() {
        eprintln!("makensis.exe failed");
        return;
    }

    println!("Created {}", out_path.display());
}