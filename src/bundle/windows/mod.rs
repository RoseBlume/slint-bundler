
use std::fs;
mod msix_bundle;
pub use msix_bundle::bundle_msix;
mod msi;
pub use msi::bundle_msi;
mod nsis;
pub use nsis::bundle_nsis;

/// Helper: Make package name look nicer by replacing hyphens with spaces and capitalizing words
pub fn prettify_package_name(name: &str) -> String {
    name.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}
/// Helper: Extract name and version from Cargo.toml
pub fn read_package_metadata() -> (String, String) {
    let manifest = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");
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

    (package_name, version)
}



