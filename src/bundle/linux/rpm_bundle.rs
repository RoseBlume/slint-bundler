use std::path::Path;
use std::fs;
use image;
use rpm::{PackageBuilder, FileOptions};
use crate::bundle::linux::filename_arch_name;
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;
use crate::bundle::linux::effective_arch;
use crate::bundle::linux::write_desktop_file;

pub fn bundle_rpm() {
    println!("Creating .rpm package...");

    // Read package name and version from Cargo.toml
    let manifest = std::fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");
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

    // Path to compiled binary
    let release_bin = Path::new("target").join("release").join(&package_name);
    if !release_bin.exists() {
        eprintln!("Release binary not found at {}. Make sure `cargo build --release` ran successfully.", release_bin.display());
        return;
    }

    let tmp = tempdir().expect("failed to create tempdir");
    let pkg_root = tmp.path().to_path_buf();

    // Create package layout
    let usr_bin = pkg_root.join("usr").join("bin");
    let applications_dir = pkg_root.join("usr").join("share").join("applications");
    let icons_root = pkg_root.join("usr").join("share").join("icons").join("hicolor");
    fs::create_dir_all(&usr_bin).expect("failed to create usr/bin");
    fs::create_dir_all(&applications_dir).expect("failed to create applications dir");
    fs::create_dir_all(&icons_root).expect("failed to create icons root");

    // Copy binary into staged tree
    let dest_bin = usr_bin.join(&package_name);
    fs::copy(&release_bin, &dest_bin).expect("failed to copy binary");
    let mut perms = fs::metadata(&dest_bin).expect("meta").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&dest_bin, perms).expect("failed to set permissions");

    // Write desktop file
    write_desktop_file(&package_name, &applications_dir).expect("failed to write desktop file");

    // Install icons from ./icons folder into the staged tree
    let icons_dir = Path::new("icons");
    if icons_dir.exists() && icons_dir.is_dir() {
        for entry in fs::read_dir(icons_dir).expect("failed to read icons dir").flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if let Ok(img) = image::open(&path) {
                let w = img.width();
                let h = img.height();
                let size_dir = icons_root.join(format!("{}x{}", w, h)).join("apps");
                fs::create_dir_all(&size_dir).expect("failed to create icon size dir");
                let dest = size_dir.join(format!("{}.png", package_name));
                let mut fout = fs::File::create(&dest).expect("failed to create icon dest");
                img.write_to(&mut fout, image::ImageFormat::Png).expect("failed to write icon png");
            } else {
                eprintln!("Warning: failed to read image {}", path.display());
            }
        }
    }

    // Build RPM programmatically using rpm crate
    let eff = effective_arch();
    let arch = filename_arch_name(&eff);

    let name = package_name.clone();
    let summary = package_name.clone();
    let license = "MIT";

    let mut builder = PackageBuilder::new(&name, &version, license, arch, &summary);

    // Add the staged binary
    builder = builder.with_file(&dest_bin, FileOptions::new(format!("/usr/bin/{}", name))).expect("failed to add binary to rpm");

    // Add desktop file
    let desktop_src = applications_dir.join(format!("{}.desktop", name));
    builder = builder.with_file(&desktop_src, FileOptions::new(format!("/usr/share/applications/{}.desktop", name))).expect("failed to add desktop to rpm");

    // Add icons: walk staged icons tree and add files
    if icons_root.exists() {
        for size_entry in fs::read_dir(&icons_root).unwrap().flatten() {
            let size_dir = size_entry.path();
            let apps_dir = size_dir.join("apps");
            if apps_dir.exists() && apps_dir.is_dir() {
                for icon_entry in fs::read_dir(&apps_dir).unwrap().flatten() {
                    let src = icon_entry.path();
                    if src.is_file() {
                        let rel = src.strip_prefix(&pkg_root).unwrap();
                        let dest = format!("/{}", rel.to_string_lossy());
                        builder = builder.with_file(&src, FileOptions::new(dest)).expect("failed to add icon to rpm");
                    }
                }
            }
        }
    }

    // Write RPM to target/release/bundle/rpm
    let pkg = builder.build().expect("failed to build rpm package");
    let out_dir = Path::new("target").join("release").join("bundle").join("rpm");
    fs::create_dir_all(&out_dir).expect("failed to create rpm output dir");
    let out_path = out_dir.join(format!("{}_{}_{}.rpm", name, version, arch));
    let mut f = fs::File::create(&out_path).expect("failed to create rpm file");
    pkg.write(&mut f).expect("failed to write rpm");
    println!("Created {}", out_path.display());
}