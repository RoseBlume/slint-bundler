use std::fs::{File, self};
use std::path::{Path, PathBuf};
use regex::Regex;
use std::env;
use std::io::{self, Write};
use crate::icon::generate_pngs;

const ICON: &[u8; 145951] = include_bytes!("./icon.png");
fn unpack_icon(package_name: &str) {
    let output_path = format!("{}\\icons\\icon.png", package_name);
    fs::create_dir_all(format!("{}\\icons", package_name)).expect("Failed to create icons directory");
    std::fs::write(output_path, ICON).expect("Failed to write icon.png");
}

// {
//     "app_id": "io.github.RoseBlume.native_lib",
//     "project_name": "Native Lib",
//     "version": "1.0.0",
//     "package_name": "native_lib" 
// }

// let (app_id, project_name, version, package_name) = read_package_metadata();
struct TemplateFile {
    path: PathBuf,
    content: String,
}
fn is_snake_case(s: &str) -> bool {
    let re = Regex::new(r"^[a-z0-9]+(?:_[a-z0-9]+)*$").unwrap();
    re.is_match(s)
}

fn get_package_name() -> String {
    let mut package_name: String = "slint_lib".to_string();
    print!("Enter your package name(slint_lib):");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut package_name).expect("Failed to read line");
    package_name.trim().to_string()
} 
fn get_project_name() -> String {
    let mut project_name: String = "slint-project".to_string();
    print!("Enter your project name(slint-project):");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut project_name).expect("Failed to read line");
    project_name.trim().to_string()
}
fn get_app_id() -> String {
    let mut app_id: String = "io.github.slint.project".to_string();
    print!("Enter your app id(default: io.github.slint.project):");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut app_id).expect("Failed to read line");
    app_id.trim().to_string()
}
pub fn handle_new() {
    let package_name = get_package_name();
    let app_id = get_app_id();
    let project_name = get_project_name();
    let version = "0.1.0";
    let template_files = generate_template_files(&package_name, &version, &app_id, &project_name);
    let target_dir = Path::new(&package_name);
    
    for template in template_files {
        let full_path = target_dir.join(&template.path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).expect("Failed to write template files");
        }
        fs::write(full_path, template.content).expect("Failed to write template files");
    }
    unpack_icon(&package_name);
    
}
fn generate_template_files(package_name: &str, version: &str, app_id: &str, project_name: &str) -> Vec<TemplateFile> {
    let mut files = Vec::new();
    files.push(TemplateFile {
        path: PathBuf::from("slint-app.json"),
        content: format!(r#"{{
    "app_id": "{app_id}",
    "project_name": "{project_name}",
    "version": "{version}",
    "package_name": "{package_name}" 
    }}"#, package_name = package_name, version=version, app_id = app_id, project_name = project_name)
    });
    files.push(TemplateFile {
        path: PathBuf::from("Cargo.toml"),
        content: format!(r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "{package_name}"

[lib]
path = "src/lib.rs"
crate-type = ["cdylib"]


[target."cfg(target_os = \"android\")".dependencies]
slint = {{version = "1.14.1", features = ["std", "backend-android-activity-06"]}}
# i-slint-backend-android-activity = "1.14.1"

[target."cfg(not(any(target_os = \"android\", target_os = \"ios\")))".build-dependencies]
slint-build = {{version = "1.14.1"}}

[target."cfg(not(any(target_os = \"android\", target_os = \"ios\")))".dependencies]
slint = "1.14.1"

[profile.release]
opt-level = 3
strip = "debuginfo"
"#, package_name = package_name)
    });
    files.push(TemplateFile {
        path: PathBuf::from("ui/app-window.slint"),
        content: r#"import { Button, VerticalBox } from "std-widgets.slint";

export component AppWindow inherits Window {
    in-out property <int> counter: 42;
    callback request-increase-value();
    callback request-decrease-value();
    VerticalBox {
        Button {
            text: "Increase value";
            clicked => {
                root.request-increase-value();
            }
        }
        Text {
            text: "Counter: \{root.counter}";
        }
        Button {
            text: "Decrease value";
            clicked => {
                root.request-decrease-value();
            }
        }
        Text {
            text: "This is a simple test!";
        }
        width: parent.width;
        height: parent.height;


    }
}
"#.to_string()
    });
    files.push(TemplateFile {
        path: PathBuf::from("build.rs"),
        content: r#"fn main() {
    slint_build::compile("ui/app-window.slint").expect("Slint build failed");
}"#.to_string()
    });

    files.push(TemplateFile {
        path: PathBuf::from("src/lib.rs"),
        content: r#"
use std::env;

slint::include_modules!();
#[unsafe(no_mangle)]
pub fn android_main(app: slint::android::AndroidApp) -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("SLINT_FULLSCREEN", "true");
    slint::android::init(app).unwrap();
    run()
}
// Test Test


fn run() -> Result<(), Box<dyn std::error::Error>> {
    let ui = AppWindow::new()?;
    ui.on_request_increase_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() + 1);
        }
    });
    ui.on_request_decrease_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() - 1);
        }
    });

    ui.run()?;

    Ok(())
}"#.to_string()
    });

    files.push(TemplateFile {
        path: PathBuf::from("src/main.rs"),
        content: r#"#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let ui = AppWindow::new()?;
    ui.on_request_increase_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() + 1);
        }
    });
    ui.on_request_decrease_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() - 1);
        }
    });

    ui.run()?;

    Ok(())
}

fn main() {
    run().expect("Failed to run on desktop");
}"#.to_string()
    });
    files.push(TemplateFile {
        path: PathBuf::from(".gitignore"),
        content: r#".cargo
target
build
gradle
*.jks
*.keystore
*.apk
*.idsig"#.to_string()
    });


    files
}