fn main() {
    slint_build::compile("ui/app-window.slint").expect("Slint build failed");
    if std::env::var("PROFILE").as_deref() == Ok("release") {
        #[cfg(target_os = "windows")]
        {
            // Use cargo-wix to generate an MSI installer
            // Requires: cargo install cargo-wix
            std::process::Command::new("cargo")
                .args(&["wix", "init"])
                .status()
                .expect("Failed to initialize wix");
            std::process::Command::new("cargo")
                .args(&["wix", "build"])
                .status()
                .expect("Failed to build MSI installer");
        }

        #[cfg(target_os = "linux")]
        {
            // Use cargo-deb for .deb, cargo-generate-rpm for .rpm, and cargo-arch for Arch package
            // Requires: cargo install cargo-deb cargo-generate-rpm cargo-arch
            std::process::Command::new("cargo")
                .args(&["deb"])
                .status()
                .expect("Failed to build .deb package");
            std::process::Command::new("cargo")
                .args(&["generate-rpm"])
                .status()
                .expect("Failed to build .rpm package");
            std::process::Command::new("cargo")
                .args(&["arch"])
                .status()
                .expect("Failed to build Arch Linux package");
        }
    }
}
