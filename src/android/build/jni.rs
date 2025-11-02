use std::fs;
use std::path::Path;
use std::env;
use toml::Value;



// target/aarch64-linux-android/

fn get_lib_name(mut mode: &str) -> Option<String> {
    // Try to find an actual .so built in any of the Android target release folders first
    let project_root = env::current_dir().ok()?;
    let triples = [
        "aarch64-linux-android",
        "armv7-linux-androideabi",
        "i686-linux-android",
        "x86_64-linux-android",
    ];
    if mode == "--release" {
        mode = "release";
    }
    else {
        mode = "debug";
    }

    for t in &triples {
        let dir = project_root.join("target").join(t).join(mode);
        if dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("so") {
                        if let Some(fname) = path.file_name().and_then(|s| s.to_str()) {
                            return Some(fname.to_string());
                        }
                    }
                }
            }
        }
    }

    // Fallback: derive from Cargo.toml (use lib{name}.so which is what Rust produces for cdylib)
    let cargo_toml = fs::read_to_string("Cargo.toml").ok()?;
    let value = cargo_toml.parse::<Value>().ok()?;
    let pkg_name = value.get("package")?.get("name")?.as_str()?;
    Some(format!("lib{}.so", pkg_name))
}

// arm64-v8a: Will link to targets/aarch64-linux-android/release/{libname}
// armeabi-v7a: Will link to targets/armv7-linux-androideabi/release/{libname}
// x86: Will link to targets/i686-linux-android/release/{libname}
// x86_64: Will link to targets/x86_64-linux-android/release/{libname}

// Links will be put in android/app/src/main/jniLibs
pub fn create_symbolic_links(mut mode: &str) {
    let lib_name = match get_lib_name(mode) {
        Some(n) => n,
        None => {
            eprintln!("Could not determine library name from Cargo.toml");
            return;
        }
    };

    let mappings = [
        ("arm64-v8a", "aarch64-linux-android"),
        ("armeabi-v7a", "armv7-linux-androideabi"),
        ("x86", "i686-linux-android"),
        ("x86_64", "x86_64-linux-android"),
    ];

    let project_root = env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let jni_base = Path::new("android").join("app").join("src").join("main").join("jniLibs");
    if mode == "--release" {
        mode = "release";
    }
    else {
        mode = "debug";
    }
    for (abi_dir, target_triple) in &mappings {
        let target_path = project_root
            .join("target")
            .join(target_triple)
            .join(mode)
            .join(&lib_name);

        let dest_dir = jni_base.join(abi_dir);
        if let Err(e) = fs::create_dir_all(&dest_dir) {
            eprintln!("Failed to create {:?}: {}", dest_dir, e);
            continue;
        }

        let dest_link = dest_dir.join(&lib_name);

        if dest_link.exists() {
            if let Err(e) = fs::remove_file(&dest_link) {
                eprintln!("Failed to remove existing {:?}: {}", dest_link, e);
                // continue attempting to create the new link anyway
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Err(e) = std::os::unix::fs::symlink(&target_path, &dest_link) {
                eprintln!("Failed to create symlink {:?} -> {:?}: {}", dest_link, target_path, e);
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Use file symlink on Windows
            if let Err(e) = std::os::windows::fs::symlink_file(&target_path, &dest_link) {
                eprintln!("Failed to create symlink {:?} -> {:?}: {}", dest_link, target_path, e);
            }
        }
    }
}