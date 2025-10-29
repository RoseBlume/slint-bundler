use std::io::{self};
use std::path::Path;
use std::fs;


/// Converts an executable name like "my_app" to "My App"
fn prettify_name(exe_name: &str) -> String {
    exe_name
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}


/// Write a desktop file for a binary name into the given output directory.
pub fn write_desktop_file(bin_name: &str, output_dir: &Path) -> io::Result<()> {
    let desktop_content = format!(
        "[Desktop Entry]\nType=Application\nName={name}\nExec={bin}\nIcon={bin}\nTerminal=false\nCategories=Utility;\n",
        name = prettify_name(bin_name),
        bin = bin_name
    );
    let desktop_file_path = output_dir.join(format!("{}.desktop", bin_name));
    fs::write(desktop_file_path, desktop_content.as_bytes())?;
    Ok(())
}

