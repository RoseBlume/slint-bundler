use std::process::Command;
use std::path::Path;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;
use image;

use crate::bundle::linux::write_desktop_file;


pub fn bundle_standalone() {
    println!("Creating standalone AppImage...");

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

    // Ensure release binary exists
    let release_bin = Path::new("target").join("release").join(&package_name);
    if !release_bin.exists() {
        eprintln!("Release binary not found at {}. Make sure `cargo build --release` ran successfully.", release_bin.display());
        return;
    }

    // Build an AppDir tree in a tempdir
    let tmp = tempdir().expect("failed to create tempdir");
    let appdir = tmp.path().join(format!("{}.AppDir", package_name));
    let usr_bin = appdir.join("usr").join("bin");
    let applications_dir = appdir.join("usr").join("share").join("applications");
    let icons_root = appdir.join("usr").join("share").join("icons").join("hicolor");
    fs::create_dir_all(&usr_bin).expect("failed to create usr/bin");
    fs::create_dir_all(&applications_dir).expect("failed to create applications dir");
    fs::create_dir_all(&icons_root).expect("failed to create icons root");

    // Copy binary into AppDir usr/bin
    let dest_bin = usr_bin.join(&package_name);
    fs::copy(&release_bin, &dest_bin).expect("failed to copy binary into AppDir");
    let mut perms = fs::metadata(&dest_bin).expect("meta").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&dest_bin, perms).expect("failed to set permissions");

    // Write desktop file and icons into AppDir
    write_desktop_file(&package_name, &applications_dir).expect("failed to write desktop file");
    let icons_dir = Path::new("icons");
    if icons_dir.exists() && icons_dir.is_dir() {
        for entry in fs::read_dir(icons_dir).expect("failed to read icons dir").flatten() {
            let path = entry.path();
            if !path.is_file() { continue; }
            if let Ok(img) = image::open(&path) {
                let w = img.width();
                let h = img.height();
                let size_dir = icons_root.join(format!("{}x{}", w, h)).join("apps");
                fs::create_dir_all(&size_dir).expect("failed to create icon size dir");
                let dest = size_dir.join(format!("{}.png", package_name));
                let mut fout = fs::File::create(&dest).expect("failed to create icon dest");
                img.write_to(&mut fout, image::ImageFormat::Png).expect("failed to write icon png");
            }
        }
    }

    // derive arch
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        "arm" | "armv7" | "armv7l" => "armhfp",
        "x86" | "i386" => "i386",
        _ => "noarch",
    };

    // Prepare output path
    let out_dir = Path::new("target").join("release").join("bundle").join("standalone");
    fs::create_dir_all(&out_dir).expect("failed to create output bundle dir");
    let out_path = out_dir.join(format!("{}_{}_{}.AppImage", package_name, version, arch));

    // Prefer to use `appimagetool` if available; create AppImage from AppDir
    if which::which("appimagetool").is_ok() {
        // run appimagetool <AppDir> <outpath>
        let status = Command::new("appimagetool")
            .arg(&appdir)
            .arg(&out_path)
            .status()
            .expect("failed to run appimagetool");
        if status.success() {
            println!("Created {}", out_path.display());
            return;
        } else {
            eprintln!("appimagetool failed (exit {}). Falling back to creating AppDir archive.", status);
        }
    } else {
        eprintln!("appimagetool not found in PATH. Creating AppDir tar.gz as fallback (not a runnable AppImage).");
    }

    // Fallback: create a tar.gz snapshot of the AppDir (note: not a proper AppImage)
    let fd = fs::File::create(&out_path.with_extension("tar.gz")).expect("failed to create fallback output file");
    let enc = flate2::write::GzEncoder::new(fd, flate2::Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all("./", &appdir).expect("failed to append appdir");
    tar.finish().expect("failed to finish tar");

    println!("Wrote AppDir archive fallback at {} (not an AppImage)", out_path.with_extension("tar.gz").display());
}