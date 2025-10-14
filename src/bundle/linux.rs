use std::process::Command;
use std::path::Path;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;
use image;
use crate::desktop::write_desktop_file;
use rpm::{PackageBuilder, FileOptions};

fn normalize_host_arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        "arm" | "armv7" | "armv7l" => "armhf",
        "x86" | "i386" => "i386",
        "riscv64" => "riscv64",
        _ => "noarch",
    }
}
fn filename_arch_name(eff: &str) -> &str {
    match eff {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        "armhf" => "armhfp",
        "i386" => "i386",
        "riscv64" => "riscv64",
        _ => "noarch",
    }
}

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


// Return the effective arch: if SLINT_BUNDLER_FORCE_ARCH is set, use that,
// otherwise use the normalized host arch.
fn effective_arch() -> String {
    if let Ok(v) = std::env::var("SLINT_BUNDLER_FORCE_ARCH") {
        return v;
    }
    normalize_host_arch().to_string()
}


// Map effective arch to names used in filenames (keeps most mapping identical,
// but maps "armhf" -> "armhfp" where a previous convention required it).

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

pub fn bundle_tar_xz() {
    println!("Creating .tar.xz package (Arch)...");

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

    // Stage files into tempdir similar to tar.zst
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
    let out_dir = Path::new("target").join("release").join("bundle").join("tar.xz");
    fs::create_dir_all(&out_dir).expect("failed to create output bundle dir");
    // derive arch similar to RPM
    let eff = effective_arch();
    let arch = filename_arch_name(&eff);
    let out_path = out_dir.join(format!("{}_{}_{}.tar.xz", package_name, version, arch));

    // Create tar and compress with xz
    let tar_fd = fs::File::create(&out_path).expect("failed to create output file");
    let enc = xz2::write::XzEncoder::new(tar_fd, 6);
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all("usr", &pkg_root.join("usr")).expect("failed to append usr dir");
    tar.finish().expect("failed to finish tar");

    println!("Created {}", out_path.display());
}
// This will bundle as an AppImage

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