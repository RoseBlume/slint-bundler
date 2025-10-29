mod desktop;

mod deb_bundle;
mod rpm_bundle;
mod tar_xz;
mod tar_zst;
mod appimage;

pub use desktop::write_desktop_file;

pub use deb_bundle::bundle_deb;
pub use rpm_bundle::bundle_rpm;
pub use tar_xz::bundle_tar_xz;
pub use tar_zst::bundle_tar_zst;
pub use appimage::bundle_standalone;

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
pub fn filename_arch_name(eff: &str) -> &str {
    match eff {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        "armhf" => "armhfp",
        "i386" => "i386",
        "riscv64" => "riscv64",
        _ => "noarch",
    }
}




// Return the effective arch: if SLINT_BUNDLER_FORCE_ARCH is set, use that,
// otherwise use the normalized host arch.
pub fn effective_arch() -> String {
    if let Ok(v) = std::env::var("SLINT_BUNDLER_FORCE_ARCH") {
        return v;
    }
    normalize_host_arch().to_string()
}


// Map effective arch to names used in filenames (keeps most mapping identical,
// but maps "armhf" -> "armhfp" where a previous convention required it).






// This will bundle as an AppImage

