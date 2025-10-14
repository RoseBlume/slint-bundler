use std::process::Command;
use std::path::Path;
use std::fs;

#[test]
fn bundler_runs_on_example_and_produces_packages() {
    // Use the provided example project
    let example_dir = Path::new("examples/slint-rust-template");
    assert!(example_dir.exists(), "example dir must exist for the test");

    // Run the bundler in that directory: cargo run -- build --bundles deb rpm
    let status = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("build")
        .arg("--bundles")
        .arg("deb")
        .arg("rpm")
        .current_dir(example_dir)
        .status()
        .expect("failed to run bundler");
    assert!(status.success(), "bundler did not exit successfully");

    // Check for produced artifacts in the example directory or tmp (the bundler may write into CWD)
    // For .deb we expect a .deb file with the example package prefix
    let mut found_deb = false;
    let mut found_rpm = false;
    for entry in fs::read_dir(example_dir).unwrap() {
        let p = entry.unwrap().path();
        if let Some(ext) = p.extension() {
            if ext == "deb" {
                found_deb = true;
            }
            if ext == "rpm" {
                found_rpm = true;
            }
        }
    }

    assert!(found_deb || found_rpm, "expected at least a .deb or .rpm to be produced");
}
