use std::process::Command;


/* targets
armv7-linux-androideabi
aarch64-linux-android
i686-linux-android
x86_644-linux-android
*/

pub fn build_android_targets(mode: &str) -> Result<(), std::io::Error> {
    let targets = vec![
        "armv7-linux-androideabi",
        "aarch64-linux-android",
        "i686-linux-android",
        "x86_64-linux-android",
    ];
    for target in targets {
        if mode == "--release" {
            println!("Building for target: {}", target);
            let build_args = ["build", "--lib", mode, "--target", target];
            let output = Command::new("cargo")
                .args(build_args)
                .status();
        }
        else {
            println!("Building for target: {}", target);
            let build_args = ["build", "--lib", "--target", target, "--crate-type", "cdylib"];
            let output = Command::new("cargo")
                .args(build_args)
                .status();
        }
    }

    Ok(())
}