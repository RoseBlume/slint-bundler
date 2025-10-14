use std::io::{self};
use std::path::Path;
use std::fs;


/// Converts an executable name like "my_app" to "My App"
pub(crate) fn prettify_name(exe_name: &str) -> String {
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
    let pretty_name = prettify_name(bin_name);
    let desktop_content = format!(
        "[Desktop Entry]\nType=Application\nName={name}\nExec={bin}\nIcon={bin}\nTerminal=false\nCategories=Utility;\n",
        name = pretty_name,
        bin = bin_name
    );
    let desktop_file_path = output_dir.join(format!("{}.desktop", bin_name));
    fs::write(desktop_file_path, desktop_content.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_prettify_name() {
        assert_eq!(prettify_name("my_app"), "My App");
        assert_eq!(prettify_name("slint_bundler"), "Slint Bundler");
        assert_eq!(prettify_name("foo"), "Foo");
    }

    #[test]
    fn test_generate_desktop_file() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("Cargo.toml");
        std::fs::write(&manifest_path, r#"[package]
name = "test_app"
version = "0.1.0"
"#).unwrap();

        generate_desktop_file(&manifest_path, dir.path()).unwrap();
        let desktop_file = dir.path().join("test_app.desktop");
        let content = std::fs::read_to_string(desktop_file).unwrap();
        assert!(content.contains("Name=Test App"));
        assert!(content.contains("Exec=test_app"));
        assert!(content.contains("Icon=test_app"));
    }
}