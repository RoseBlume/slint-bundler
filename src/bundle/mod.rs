use std::process::Command;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use crate::bundle::linux::{bundle_deb, bundle_rpm, bundle_tar_zst, bundle_tar_xz, bundle_standalone};


// Windows-only imports
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use crate::bundle::windows::{bundle_msi, bundle_nsis, bundle_msix};




pub fn handle_build(bundles: Option<Vec<String>>) {
    // 1. Build the project in release mode
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--bin")
        .status()
        .expect("Failed to run cargo build");
    if !status.success() {
        eprintln!("Build failed");
        std::process::exit(1);
    }

    // 2. Determine bundles to create
    let os = std::env::consts::OS;
    let all_bundles = match os {
        "windows" => vec!["msi", "nsis", "msix"],
        "linux" => vec!["deb", "rpm", "tar.zst", "tar.xz", "standalone"],
        _ => vec!["standalone"],
    };
    let bundles = bundles.unwrap_or_else(|| all_bundles.iter().map(|s| s.to_string()).collect());

    // 3. Bundle for each requested type. If cross_arch flag is set, attempt to
    // create bundles for several non-host architectures by setting the
    // SLINT_BUNDLER_FORCE_ARCH environment variable per attempt.
    // let archs_to_try = vec!["i386", "x86_64", "aarch64", "armhf", "riscv64"];
    

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
            "msix" => bundle_msix(),
            _ => eprintln!("Unknown bundle type: {}", bundle),
        }
    }
}

