mod adb;

use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use std::process::{Command, Child};
use std::sync::mpsc::channel;
use std::time::Duration;
use crate::android::build::begin_build;
use adb::perform_streamed_install;
use crate::utils::{read_package_metadata};


pub fn handle_dev() {
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default()).expect("Failed to create watcher");
    let src_path = std::path::Path::new("src").canonicalize().expect("Failed to canonicalize src path");
    let ui_path = std::path::Path::new("ui").canonicalize().expect("Failed to canonicalize ui path");
    let cargo_path = std::path::Path::new("Cargo.toml").canonicalize().expect("Failed to canonicalize Cargo manifest Path");
    println!("Watching: {}", src_path.display());
    watcher.watch(&src_path, RecursiveMode::Recursive).expect("Failed to watch src");

    println!("Watching: {}", ui_path.display());
    watcher.watch(&ui_path, RecursiveMode::Recursive).expect("Failed to watch ui");

    println!("Watching {}", cargo_path.display());
    watcher.watch(&cargo_path, RecursiveMode::Recursive).expect("Failed to watch Cargo.toml");

    // Compile and run at the start
    println!("Initial build (dev profile)...");
    let mut child: Option<Child> = None;
    println!("Building Application Initially");
    begin_build("--dev");
    perform_streamed_install();

    loop {
        // println!("Waiting for file changes...");
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(event) => {
                if let Ok(event) = event {
                    if matches!(event.kind, EventKind::Modify(_)) {
                        if let Some(mut c) = child.take() {
                            let _ = c.kill();
                            let _ = c.wait();
                        }
                        println!("Rebuilding (dev profile)...");
                        begin_build("");
                        perform_streamed_install();
                    }
                }
            }
            Err(_) => {}
        }
    }
}
