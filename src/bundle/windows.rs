
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;
use zip::{write::FileOptions, ZipWriter};
use std::io::{Write, Seek};

/// Helper: Extract name and version from Cargo.toml
fn read_package_metadata() -> (String, String) {
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

/// Create an MSI installer using WiX Toolset (if available)
pub fn bundle_msi() {
    println!("Creating MSI package...");

    let (package_name, version) = read_package_metadata();
    let release_bin = Path::new("target").join("release").join(format!("{}.exe", package_name));

    if !release_bin.exists() {
        eprintln!("Release binary not found at {}. Run `cargo build --release` first.", release_bin.display());
        return;
    }

    let out_dir = Path::new("target").join("release").join("bundle").join("msi");
    fs::create_dir_all(&out_dir).expect("failed to create output dir");
    let out_path = out_dir.join(format!("{}_{}.msi", package_name, version));

    // Check for WiX Toolset
    if which::which("candle.exe").is_err() || which::which("light.exe").is_err() {
        eprintln!("WiX Toolset not found in PATH. Skipping MSI build.");
        return;
    }

    // Create a temporary .wxs file for WiX
    let tmp = tempdir().expect("failed to create tempdir");
    let wxs_path = tmp.path().join("installer.wxs");
    let wxs_content = format!(
        r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
            <Product Id="*" Name="{name}" Version="{version}" Manufacturer="Slint Bundler" UpgradeCode="12345678-1234-1234-1234-123456789abc">
                <Package InstallerVersion="500" Compressed="yes" InstallScope="perMachine" />
                <MediaTemplate />
                <Directory Id="TARGETDIR" Name="SourceDir">
                    <Directory Id="ProgramFilesFolder">
                        <Directory Id="INSTALLFOLDER" Name="{name}">
                            <Component Id="MainExecutable" Guid="*">
                                <File Id="AppBinary" Source="{bin}" KeyPath="yes" />
                            </Component>
                        </Directory>
                    </Directory>
                </Directory>
                <Feature Id="DefaultFeature" Level="1">
                    <ComponentRef Id="MainExecutable" />
                </Feature>
            </Product>
        </Wix>
        "#,
        name = package_name,
        version = version,
        bin = release_bin.display()
    );
    fs::write(&wxs_path, wxs_content).expect("failed to write wxs file");

    // Compile and link with WiX
    let wix_obj = tmp.path().join("installer.wixobj");
    let status = Command::new("candle.exe")
        .arg(&wxs_path)
        .arg("-o")
        .arg(&wix_obj)
        .status()
        .expect("failed to run candle.exe");
    if !status.success() {
        eprintln!("candle.exe failed");
        return;
    }

    let status = Command::new("light.exe")
        .arg(&wix_obj)
        .arg("-o")
        .arg(&out_path)
        .status()
        .expect("failed to run light.exe");
    if !status.success() {
        eprintln!("light.exe failed");
        return;
    }

    println!("Created {}", out_path.display());
}

/// Create an NSIS installer using makensis
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
    let out_path = out_dir.join(format!("{}_{}.exe", package_name, version));

    if which::which("makensis.exe").is_err() {
        eprintln!("makensis.exe not found in PATH. Skipping NSIS build.");
        return;
    }

    // Create a temporary NSIS script
    let tmp = tempdir().expect("failed to create tempdir");
    let nsis_script = tmp.path().join("installer.nsi");
    let script_content = format!(
        r#"
        !define APP_NAME "{name}"
        !define VERSION "{version}"
        OutFile "{out}"
        InstallDir "$PROGRAMFILES\{name}"
        Page Directory
        Page InstFiles

        Section
            SetOutPath "$INSTDIR"
            File "{bin}"
            CreateShortCut "$DESKTOP\{name}.lnk" "$INSTDIR\{exe}"
        SectionEnd
        "#,
        name = package_name,
        version = version,
        out = out_path.display(),
        bin = release_bin.display(),
        exe = release_bin.file_name().unwrap().to_string_lossy()
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

/// Create a standalone ZIP package
pub fn bundle_standalone() {
    println!("Creating standalone ZIP bundle...");

    let (package_name, version) = read_package_metadata();
    let release_bin = Path::new("target").join("release").join(format!("{}.exe", package_name));

    if !release_bin.exists() {
        eprintln!("Release binary not found at {}. Run `cargo build --release` first.", release_bin.display());
        return;
    }

    let out_dir = Path::new("target").join("release").join("bundle").join("standalone");
    fs::create_dir_all(&out_dir).expect("failed to create output dir");
    let out_path = out_dir.join(format!("{}_{}.zip", package_name, version));

    let zip_file = fs::File::create(&out_path).expect("failed to create zip file");
    let mut zip = ZipWriter::new(zip_file);
    let options: FileOptions = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let mut f = fs::File::open(&release_bin).expect("failed to open binary");
    zip.start_file(release_bin.file_name().unwrap().to_string_lossy(), options)
        .expect("failed to start zip file");
    std::io::copy(&mut f, &mut zip).expect("failed to write to zip");
    zip.finish().expect("failed to finish zip");

    println!("Created {}", out_path.display());
}
