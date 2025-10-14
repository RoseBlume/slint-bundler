use std::process::Command;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use crate::bundle::linux::{bundle_deb, bundle_rpm, bundle_tar_zst, bundle_tar_xz, bundle_standalone};


// Windows-only imports
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use crate::bundle::windows::{bundle_msi, bundle_nsis, bundle_standalone};



pub fn handle_build(bundles: Option<Vec<String>>, cross_arch: bool, _cross_method: Option<String>) {
    // 1. Build the project in release mode
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .status()
        .expect("Failed to run cargo build");
    if !status.success() {
        eprintln!("Build failed");
        std::process::exit(1);
    }

    // 2. Determine bundles to create
    let os = std::env::consts::OS;
    let all_bundles = match os {
        "windows" => vec!["msi", "nsis"],
        "linux" => vec!["deb", "rpm", "tar.zst", "tar.xz", "standalone"],
        _ => vec!["standalone"],
    };
    let bundles = bundles.unwrap_or_else(|| all_bundles.iter().map(|s| s.to_string()).collect());

    // 3. Bundle for each requested type. If cross_arch flag is set, attempt to
    // create bundles for several non-host architectures by setting the
    // SLINT_BUNDLER_FORCE_ARCH environment variable per attempt.
    // let archs_to_try = vec!["i386", "x86_64", "aarch64", "armhf", "riscv64"];
    if cross_arch {
        println!("TODO: Unfinished and disabled for now.");
        /*
        // determine host normalized arch
        let host_arch = normalize_host_arch();
        for arch in archs_to_try.into_iter().filter(|a| *a != host_arch) {
            println!("Attempting bundles for target architecture: {}", arch);
            std::env::set_var("SLINT_BUNDLER_FORCE_ARCH", arch);
            // perform a build inside a VM/container for this arch (docker/qemu)
            if let Some(method) = _cross_method.as_ref() {
                if let Err(e) = run_build_in_vm(arch, method) {
                    eprintln!("Warning: VM build for arch {} failed: {}\nFalling back to attempting packaging without fresh build.", arch, e);
                }
            } else {
                // default to docker-based emulation if available
                if let Err(e) = run_build_in_vm(arch, "docker") {
                    eprintln!("Warning: VM build (docker) for arch {} failed: {}\nFalling back to attempting packaging without fresh build.", arch, e);
                }
            }
            for bundle in &bundles {
                match bundle.as_str() {
                    "msi" => bundle_msi(),
                    "nsis" => bundle_nsis(),
                    "deb" => bundle_deb(),
                    "rpm" => bundle_rpm(),
                    "tar.zst" => bundle_tar_zst(),
                    "tar.xz" => bundle_tar_xz(),
                    "standalone" => bundle_standalone(),
                    _ => eprintln!("Unknown bundle type: {}", bundle),
                }
            }
            std::env::remove_var("SLINT_BUNDLER_FORCE_ARCH");
        }
        // Also produce host bundles as a final step
        println!("Creating bundles for host architecture");
        */
    }
    

    // produce host bundles (or only bundles if cross_arch was false)
    
    #[cfg(target_os = "linux")]
    for bundle in bundles {
        match bundle.as_str() {
            "deb" => bundle_deb(),
            "rpm" => bundle_rpm(),
            "tar.zst" => bundle_tar_zst(),
            "tar.xz" => bundle_tar_xz(),
            "standalone" => bundle_standalone(),
            _ => eprintln!("Unknown bundle type: {}", bundle),
        }
    }

    #[cfg(target_os = "windows")]
    for bundle in bundles {
        match bundle.as_str() {
            "msi" => bundle_msi(),
            "nsis" => bundle_nsis(),
            "standalone" => bundle_standalone(),
            _ => eprintln!("Unknown bundle type: {}", bundle),
        }
    }
}

