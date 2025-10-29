use std::process::Command;
use std::fs;
use std::path::{Path, PathBuf};
use crate::bundle::windows::{read_package_metadata, prettify_package_name};


/// Create an MSI installer using WiX Toolset (if available)
pub fn bundle_msi() {
    println!("Creating MSI package...");

    let (package_name, version) = read_package_metadata();
    

    let out_dir = std::path::Path::new("target")
        .join("release")
        .join("bundle")
        .join("msi");
    std::fs::create_dir_all(&out_dir).expect("failed to create output dir");
    let out_path = out_dir.join(format!("{}_{}.msi", package_name, version));
    let _ = prettify_package_name(&package_name);
    // Check for WiX Toolset (wix.exe)
    if which::which("wix.exe").is_err() {
        eprintln!("WiX Toolset (wix.exe) not found in PATH. Skipping MSI build.");
        return;
    }
    let release_bin = std::path::Path::new("target")
        .join("release")
        .join(format!("{}.exe", package_name));
    let bin_dest = Path::new("target")
        .join("release")
        .join("bundle")
        .join("msi")
        .join(format!("{}.exe", package_name));
    
    if !release_bin.exists() {
        eprintln!(
            "Release binary not found at {}. Run `cargo build --release` first.",
            release_bin.display()
        );
        return;
    }

    let ico_src = Path::new("icons").join("icon.ico");
    let icon = Path::new("target").join("release").join("bundle").join("msi").join(format!("{}_{}.ico", package_name, version));

    fs::copy(
        PathBuf::from(ico_src.clone()),
        PathBuf::from(icon.clone()),
    ).expect("Failed to copy");
    fs::copy(
        PathBuf::from(release_bin.clone()),
        PathBuf::from(bin_dest.clone()),
    ).expect("Failed to copy");
    // Create a temporary .wxs file for WiX
    
    let wxs_path = Path::new("target").join("release").join("bundle").join("msi").join("installer.wxs");
    let wxs_content = format!(
        r#"<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
    <Package Name="{name}" Language="1033" Version="{version}" Manufacturer="{name}" UpgradeCode="11111111-2222-3333-4444-555555555555">

        <!-- Icon embedded for shortcuts and ARP -->
        <Icon Id="ProductIcon" SourceFile="{name}_{version}.ico" />
        <Property Id="ARPPRODUCTICON" Value="ProductIcon" />

        <Feature Id="DefaultFeature" Level="1">
            <ComponentRef Id="MainExecutable" />
            <ComponentRef Id="AppIconComponent" />
            <ComponentRef Id="DesktopShortcutComponent" />
            <ComponentRef Id="StartMenuShortcutComponent" />
        </Feature>

        <StandardDirectory Id="ProgramFilesFolder">
            <Directory Id="INSTALLFOLDER" Name="{name}">
                <Component Id="MainExecutable">
                    <File Id="AppBinary" Source="{name}.exe" KeyPath="yes" />
                </Component>
                <Component Id="AppIconComponent">
                    <File Id="AppIconFile" Source="{name}_{version}.ico" />
                </Component>
            </Directory>
        </StandardDirectory>

        <StandardDirectory Id="DesktopFolder">
            <Component Id="DesktopShortcutComponent">
                <Shortcut Id="desktopShortcut" Name="{pretty_name}" Description="Launch {pretty_name}" Target="[INSTALLFOLDER]{name}.exe" WorkingDirectory="INSTALLFOLDER" Icon="ProductIcon" />
                <RemoveFolder Id="RemoveDesktopFolder" On="uninstall" />
                <RegistryValue Root="HKLM" Key="Software\\slint-rust-template" Name="installed" Type="integer" Value="1" KeyPath="yes" />
            </Component>
        </StandardDirectory>

        <StandardDirectory Id="ProgramMenuFolder">
            <Directory Id="ApplicationProgramsFolder" Name="{pretty_name}">
                <Component Id="StartMenuShortcutComponent">
                    <Shortcut Id="startMenuShortcut" Name="{pretty_name}" Description="Launch {pretty_name}" Target="[INSTALLFOLDER]{name}.exe" WorkingDirectory="INSTALLFOLDER" Icon="ProductIcon" />
                    <RemoveFolder Id="RemoveProgramMenuFolder" On="uninstall" />
                    <RegistryValue Root="HKLM" Key="Software\\{name}" Name="startmenu" Type="integer" Value="1" KeyPath="yes" />
                </Component>
            </Directory>
        </StandardDirectory>

    </Package>
</Wix>
"#,
        name = package_name,
        pretty_name = prettify_package_name(&package_name),
        version = version,
    );
    std::fs::write(&wxs_path, wxs_content).expect("failed to write wxs file");

    Command::new("cd target/release/bundle/msi");
    // Build the MSI using wix.exe
    let status = Command::new("wix.exe")
        //.current_dir(&out_dir_canonical)
        .arg("build")
        .arg("installer.wxs")
        .status()
        .expect("failed to run wix.exe");
    if !status.success() {
        eprintln!("wix.exe build failed");
        return;
    }

    println!("Created {}", out_path.display());
}