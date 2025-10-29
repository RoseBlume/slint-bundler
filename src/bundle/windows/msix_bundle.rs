use msix::{AppxManifest, manifest::Applications, manifest::Application/*, AppxManifest::Capabilities*/, manifest::Capability, manifest::Dependencies, manifest::Identity, Msix, manifest::Properties, manifest::Resources, manifest::Resource};
use msix::manifest::VisualElements;
use std::path::Path;
use std::path::PathBuf;
use std::fs;
use crate::bundle::windows::read_package_metadata;
/// Create an MSIX package using MakeAppx (if available)
pub fn bundle_msix() {
    println!("Creating MSIX package...");

    let (package_name, version) = read_package_metadata();
    let release_bin = Path::new("target").join("release").join(format!("{}.exe", package_name));

    if !release_bin.exists() {
        eprintln!(
            "Release binary not found at {}. Run `cargo build --release` first.",
            release_bin.display()
        );
        return;
    }

    let out_dir = Path::new("target").join("release").join("bundle").join("msix");
    if let Err(e) = fs::create_dir_all(&out_dir) {
        eprintln!("Failed to create output dir: {e}");
        return;
    }

    let out_path = out_dir.join(format!("{}_{}.msix", package_name, version));

    // Create staging folder
    let staging_dir = out_dir.join("staging");
    if staging_dir.exists() {
        if let Err(e) = fs::remove_dir_all(&staging_dir) {
            eprintln!("Failed to remove old staging dir: {e}");
            return;
        }
    }
    if let Err(e) = fs::create_dir_all(&staging_dir) {
        eprintln!("Failed to create staging dir: {e}");
        return;
    }

    // Copy binary into VFS structure
    let vfs_bin_dir = staging_dir
        .join("VFS")
        .join("ProgramFilesX64")
        .join(&package_name);
    if let Err(e) = fs::create_dir_all(&vfs_bin_dir) {
        eprintln!("Failed to create VFS path: {e}");
        return;
    }
    if let Err(e) = fs::copy(&release_bin, vfs_bin_dir.join(release_bin.file_name().unwrap())) {
        eprintln!("Failed to copy binary: {e}");
        return;
    }

    // Dummy assets
    let assets_dir = staging_dir.join("Assets");
    if let Err(e) = fs::create_dir_all(&assets_dir) {
        eprintln!("Failed to create Assets dir: {e}");
        return;
    }
    let source = PathBuf::from("icons/icon.png");
    let _ = fs::copy(source, assets_dir.join("Logo.png"));
    let source = PathBuf::from("icons/32x32.png");
    let _ = fs::copy(source, assets_dir.join("SmallLogo.png"));
    // --- Construct AppxManifest programmatically ---
    
    let mut manifest = AppxManifest::default();

    manifest.identity = Identity {
        name: Some(format!("com.slint.{}", package_name)),
        publisher: Some("CN=Slint Bundler".to_string()),
        version: Some(format!("{}.0", version)),
        processor_architecture: Some("x64".to_string()),
    };

    manifest.properties = Properties {
        display_name: Some(package_name.clone()),
        publisher_display_name: Some("Slint Bundler".to_string()),
        description: Some(format!("{} MSIX Package", package_name)),
        logo: Some("Assets\\Logo.png".to_string()),
    };

    manifest.resources = Resources {
        resource: vec![Resource {
            language: "en-us".to_string(),
        }],
    };

    manifest.dependencies = Dependencies {
        target_device_family: vec![],
    };

    manifest.capabilities = vec![
        Capability::Capability {
            name: "internetClient".to_string(),
        },
        Capability::Device {
            name: "microphone".to_string(),
        },
        Capability::Restricted {
            name: "documentsLibrary".to_string(),
        },
    ];

    manifest.applications = Applications {
        application: vec![Application {
            id: Some("io.github.RoseBlume.slintbundler".to_string()),
            executable: Some(format!(
                "VFS\\ProgramFilesX64\\{}\\{}",
                package_name,
                release_bin.file_name().unwrap().to_string_lossy()
            )),
            entry_point: Some("Windows.FullTrustApplication".to_string()),
            visual_elements: VisualElements {
                display_name: Some(package_name.clone()),
                description: Some("Rust app bundled by Slint".to_string()),
                background_color: Some("transparent".to_string()),
                logo_150x150: Some("Assets\\Logo.png".to_string()),
                logo_44x44: Some("Assets\\SmallLogo.png".to_string()),
                ..Default::default()
            },
        }],
    };

    
    
    
    //let manifest = <AppxManifest as std::default::Default>::default();

    // --- Build MSIX ---
    println!("Building MSIX package with msix crate...");

    let mut msix = match Msix::new(out_path.clone(), manifest, true) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to initialize Msix builder: {e}");
            return;
        }
    };

    if let Err(e) = msix.add_directory(&staging_dir, Path::new("")) {
        eprintln!("Failed to add directory to MSIX: {e}");
        return;
    }

    if let Err(e) = msix.add_icon(&assets_dir.join("Logo.png")) {
        eprintln!("Failed to add icon: {e}");
    }

    if let Err(e) = msix.finish(None) {
        eprintln!("Failed to finalize MSIX package: {e}");
        return;
    }

    println!("Created {}", out_path.display());
}