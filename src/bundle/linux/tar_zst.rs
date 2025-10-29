use std::path::Path;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;
use image;
use crate::bundle::linux::effective_arch;
use crate::bundle::linux::write_desktop_file;
use crate::bundle::linux::filename_arch_name;
pub fn bundle_tar_zst() {
    println!("Creating .tar.zst package (Arch)...");

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

    // Stage files into a tempdir (reuse logic similar to other bundlers)
    let tmp = tempdir().expect("failed to create tempdir");
    let pkg_root = tmp.path().to_path_buf();
    let usr_bin = pkg_root.join("usr").join("bin");
    let applications_dir = pkg_root.join("usr").join("share").join("applications");
    let icons_root = pkg_root.join("usr").join("share").join("icons").join("hicolor");
    fs::create_dir_all(&usr_bin).expect("failed to create usr/bin");
    fs::create_dir_all(&applications_dir).expect("failed to create applications dir");
    fs::create_dir_all(&icons_root).expect("failed to create icons root");

    // Copy binary
    let dest_bin = usr_bin.join(&package_name);
    fs::copy(&release_bin, &dest_bin).expect("failed to copy binary");
    let mut perms = fs::metadata(&dest_bin).expect("meta").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&dest_bin, perms).expect("failed to set permissions");

    // Write desktop file and icons into staged tree
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

    // Create output directory
    let out_dir = Path::new("target").join("release").join("bundle").join("tar.zst");
    fs::create_dir_all(&out_dir).expect("failed to create output bundle dir");
    // derive arch similar to RPM
    let eff = effective_arch();
    let arch = filename_arch_name(&eff);
    let out_path = out_dir.join(format!("{}_{}_{}.tar.zst", package_name, version, arch));

    // Create tar and compress with zstd
    let tar_fd = fs::File::create(&out_path).expect("failed to create output file");
    let zstd_enc = zstd::stream::write::Encoder::new(tar_fd, 0).expect("failed to create zstd encoder");
    let mut tar = tar::Builder::new(zstd_enc);
    tar.append_dir_all("usr", &pkg_root.join("usr")).expect("failed to append usr dir");
    // finish tar and get the inner encoder back
    let enc = tar.into_inner().expect("failed to into_inner tar");
    enc.finish().expect("failed to finish zstd");

    println!("Created {}", out_path.display());
}

