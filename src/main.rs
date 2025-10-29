mod bundle;
mod dev;
mod icon;
mod doctor;

use std::env;
use std::process;

const USAGE: &str = "Usage: slint-bundler <command> [options]

Commands:
  build     Build and bundle the project
            Options: --bundles <bundle-types>
  dev       Run the project in dev mode (auto-recompile on change)
  icon      Generate icons from PNG input
            Options: --input <png-file>
  doctor    Check environment setup
            Options: --fix
  android   Build Android package
  help      Print this message or help for specific command

Examples:
  slint-bundler build --bundles deb,msi
  slint-bundler icon --input icon.png
  slint-bundler help build";

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("{}", USAGE);
        process::exit(1);
    }

    match args[1].as_str() {
        "build" => {
            let bundles = parse_bundles(&args);
            bundle::handle_build(bundles);
        },
        "dev" => dev::handle_dev(),
        "icon" => {
            let input = parse_input(&args);
            if let Err(e) = icon::generate_pngs(&input) {
                eprintln!("Error generating icons: {}", e);
            }
        },
        "doctor" => {
            let fix = args.contains(&"--fix".to_string());
            doctor::doctor(fix);
        },
        "android" => bundle::handle_android(&args[2]),
        "help" => {
            if args.len() > 2 {
                print_command_help(&args[2]);
            } else {
                println!("{}", USAGE);
            }
        },
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            println!("\n{}", USAGE);
            process::exit(1);
        }
    }
}

fn parse_bundles(args: &[String]) -> Option<Vec<String>> {
    let bundle_index = args.iter().position(|arg| arg == "--bundles")?;
    if bundle_index + 1 >= args.len() {
        return None;
    }
    Some(args[bundle_index + 1]
        .split(',')
        .map(String::from)
        .collect())
}

fn parse_input(args: &[String]) -> String {
    args.iter()
        .position(|arg| arg == "--input")
        .and_then(|i| args.get(i + 1))
        .map(String::from)
        .unwrap_or_else(|| {
            eprintln!("Error: --input <png-file> is required for icon command");
            process::exit(1);
        })
}

fn print_command_help(command: &str) {
    match command {
        "build" => println!("Usage: slint-bundler build --bundles <bundle-types>\n\nCreate installation bundles for the project.\nBundle types: deb,msi,rpm,etc\nExample: slint-bundler build --bundles deb,msi"),
        "dev" => println!("Usage: slint-bundler dev\n\nRun the project in development mode with auto-recompile on file changes."),
        "icon" => println!("Usage: slint-bundler icon --input <png-file>\n\nGenerate application icons from a 1024x1024 PNG file."),
        "doctor" => println!("Usage: slint-bundler doctor [--fix]\n\nCheck the development environment setup.\nUse --fix to attempt automatic fixes."),
        "android" => println!("Usage: slint-bundler android\n\nBuild Android package for the project."),
        _ => println!("{}", USAGE),
    }
}