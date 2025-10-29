use std::fs;
use std::path::Path;
use tempfile::tempdir;
use std::os::unix::fs::PermissionsExt;
use crate::bundle::linux::effective_arch;
use crate::bundle::linux::write_desktop_file;
fn deb_arch_name(eff: &str) -> &str {
    match eff {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        "armhf" => "armhf",
        "i386" => "i386",
        "riscv64" => "riscv64",
        _ => "all",
    }
}

pub fn bundle_deb() {
    println!("Creating .deb package...");

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

    // Create standard Debian package layout
    let debian_dir = pkg_root.join("DEBIAN");
    fs::create_dir_all(&debian_dir).expect("failed to create DEBIAN dir");
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

    // Generate .desktop file
    write_desktop_file(&package_name, &applications_dir).expect("failed to write desktop file");

    // Install icons from ./icons folder
    let icons_dir = Path::new("icons");
    if icons_dir.exists() && icons_dir.is_dir() {
        let entries = fs::read_dir(icons_dir).expect("failed to read icons dir");
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() { continue; }
                // attempt to read image to get dimensions
                match image::open(&path) {
                    Ok(img) => {
                    let w = img.width();
                    let h = img.height();
                    let size_dir = icons_root.join(format!("{}x{}", w, h)).join("apps");
                    fs::create_dir_all(&size_dir).expect("failed to create icon size dir");
                    let dest = size_dir.join(format!("{}.png", package_name));
                    // write the image as PNG to the destination to ensure correct format
                    let mut fout = fs::File::create(&dest).expect("failed to create icon dest");
                    img.write_to(&mut fout, image::ImageFormat::Png).expect("failed to write icon png");
                }
                Err(_) => {
                    eprintln!("Warning: failed to read image {}", path.display());
                }
            }
        }
    } else {
        eprintln!("Warning: ./icons directory not found; no icons will be installed into package.");
    }

    // Create control file
    // determine architecture string for Debian control file, respect forced arch env
    let eff = effective_arch();
    let arch = deb_arch_name(&eff).to_string();
    let control = format!(
        "Package: {pkg}\nVersion: {ver}\nSection: utils\nPriority: optional\nArchitecture: {arch}\nMaintainer: packager <packager@local>\nDescription: {pkg} packaged by slint-bundler\n",
        pkg = package_name,
        ver = version,
        arch = arch
    );
    fs::write(debian_dir.join("control"), control).expect("failed to write control file");

    // Build .deb ourselves (create data.tar.gz and control.tar.gz and then ar archive)
    let out_dir = Path::new("target").join("release").join("bundle").join("deb");
    fs::create_dir_all(&out_dir).expect("failed to create output bundle dir");
    let output_deb_path = out_dir.join(format!("{}_{}_{}.deb", package_name, version, arch));
    let control_tar_gz = tmp.path().join("control.tar.gz");
    let data_tar_gz = tmp.path().join("data.tar.gz");

    // Create control.tar.gz
    {
        let control_fd = fs::File::create(&control_tar_gz).expect("failed to create control.tar.gz");
        let enc = flate2::write::GzEncoder::new(control_fd, flate2::Compression::default());
        let mut tar = tar::Builder::new(enc);
        // add DEBIAN/control
        tar.append_path_with_name(debian_dir.join("control"), "control").expect("failed to append control");
        // finish and flush
        tar.finish().expect("failed to finish control tar");
    }

    // Create data.tar.gz (everything under pkg_root except DEBIAN)
    {
        let data_fd = fs::File::create(&data_tar_gz).expect("failed to create data.tar.gz");
        let enc = flate2::write::GzEncoder::new(data_fd, flate2::Compression::default());
        let mut tar = tar::Builder::new(enc);
        // append usr/ and other dirs
        let usr_dir = pkg_root.join("usr");
        tar.append_dir_all("./usr", &usr_dir).expect("failed to append usr dir");
        tar.finish().expect("failed to finish data tar");
    }

    // Write the debian-binary (must contain '2.0' and newline)
    let debian_binary = tmp.path().join("debian-binary");
    fs::write(&debian_binary, b"2.0\n").expect("failed to write debian-binary");

    // Create ar archive: debian-binary, control.tar.gz, data.tar.gz
    {
        let out = fs::File::create(&output_deb_path).expect("failed to create output deb");
        let mut ar_builder = ar::Builder::new(out);
        ar_builder.append_path(&debian_binary).expect("failed to add debian-binary to ar");
        ar_builder.append_path(&control_tar_gz).expect("failed to add control.tar.gz to ar");
        ar_builder.append_path(&data_tar_gz).expect("failed to add data.tar.gz to ar");
        // ar::Builder finishes when dropped
    }

    println!("Created {}", output_deb_path.display());
}