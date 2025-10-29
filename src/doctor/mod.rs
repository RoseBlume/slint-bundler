use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::env;
use whoami::username;


#[derive(Clone, Debug)]
pub struct Doctor {
    groups: Vec<Group>,
}

impl Default for Doctor {
    fn default() -> Self {
        Self {
            groups: vec![
                Group {
                    name: "clang/llvm toolchain",
                    checks: vec![
                        Check::new("clang", Some(VersionCheck::new("--version", 0, 2))),
                        Check::new("clang++", Some(VersionCheck::new("--version", 0, 2))),
                        Check::new("llvm-ar", None),
                        Check::new("llvm-lib", None),
                        Check::new("llvm-readobj", Some(VersionCheck::new("--version", 1, 4))),
                        Check::new("lld", Some(VersionCheck::new("-flavor ld --version", 0, 1))),
                        Check::new("lld-link", Some(VersionCheck::new("--version", 0, 1))),
                        Check::new("lldb", Some(VersionCheck::new("--version", 0, 2))),
                        Check::new("lldb-server", None),
                    ],
                },
                Group {
                    name: "rust",
                    checks: vec![
                        Check::new("rustup", Some(VersionCheck::new("--version", 0, 1))),
                        Check::new("cargo", Some(VersionCheck::new("--version", 0, 1))),
                    ],
                },
                Group {
                    name: "android",
                    checks: vec![
                        Check::new("adb", Some(VersionCheck::new("--version", 0, 4))),
                        Check::new("javac", Some(VersionCheck::new("--version", 0, 1))),
                        Check::new("java", Some(VersionCheck::new("--version", 0, 1))),
                        Check::new("kotlin", Some(VersionCheck::new("-version", 0, 2))),
                        Check::new("gradle", Some(VersionCheck::new("--version", 2, 1))),
                    ],
                },
                Group {
                    name: "ios",
                    checks: vec![
                        Check::new("idevice_id", Some(VersionCheck::new("-v", 0, 1))),
                        Check::new("ideviceinfo", Some(VersionCheck::new("-v", 0, 1))),
                        Check::new("ideviceinstaller", Some(VersionCheck::new("-v", 0, 1))),
                        Check::new("ideviceimagemounter", Some(VersionCheck::new("-v", 0, 1))),
                        Check::new("idevicedebug", Some(VersionCheck::new("-v", 0, 1))),
                        Check::new(
                            "idevicedebugserverproxy",
                            Some(VersionCheck::new("-v", 0, 1)),
                        ),
                    ],
                },
                Group {
                    name: "linux",
                    checks: vec![
                        Check::new("mksquashfs", Some(VersionCheck::new("-version", 0, 2)))],
                },
                Group {
                    name: "NSIS",
                    checks: vec![
                        Check::new("makensis", Some(VersionCheck::new("/VERSION", 0, 3)),)]
                },
                Group {
                    name: "WiX Toolset",
                    checks: vec![Check::new(
                        "wix",
                        Some(VersionCheck::new("--version", 0, 6))
                    )
                    ]
                }
            ],
        }
    }
}
impl Doctor {
    fn fix(&self, search_dirs: Vec<String>) {
        let mut found_dirs: Vec<PathBuf> = Vec::new();

        for group in &self.groups {
            for check in &group.checks {
                // Skip if the executable already exists
                if check.path().is_ok() {
                    continue;
                }

                let exe_name = check.name();

                // Try to find the missing executable in the provided directories
                for dir in &search_dirs {
                    let dir_path = Path::new(dir.as_str());

                    if !dir_path.exists() {
                        continue;
                    }
                    //let message = for
                    let executable = exe_name.to_string();
                    let diret = dir_path.to_str().expect("Failure");
                    
                    println!("Found {executable} in {diret}");
                    if !found_dirs.contains(&dir_path.to_path_buf()) {
                        found_dirs.push(dir_path.to_path_buf());
                    }
            
                }
            }
        }
        // Add found directories to the PATH
    
        let mut paths = env::split_paths(&env::var_os("PATH").unwrap_or_default()).collect::<Vec<_>>();
        for dir in found_dirs.iter() {
            if !paths.contains(&dir) {
                println!("🔧 Adding '{}' to PATH", dir.display());
                paths.push(dir.to_path_buf());
            }
        }

        // ✅ Fix: unwrap the Result<OsString, JoinPathsError>
        let new_path = env::join_paths(paths).expect("Failed to join PATH entries");
        env::set_var("PATH", new_path);
    }
}

impl std::fmt::Display for Doctor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for group in &self.groups {
            write!(f, "{group}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Group {
    name: &'static str,
    checks: Vec<Check>,
}

impl std::fmt::Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{:-^1$}", self.name, 60)?;
        for check in &self.checks {
            write!(f, "{:20} ", check.name())?;
            if let Ok(path) = check.path() {
                let version = if let Ok(Some(version)) = check.version() {
                    version
                } else {
                    "unknown".into()
                };
                write!(f, "{version:20}")?;
                write!(f, "{}", path.display())?;
            } else {
                write!(f, "not found")?;
            }
            writeln!(f)?;
        }
        writeln!(f)
    }
}

#[derive(Clone, Copy, Debug)]
struct Check {
    name: &'static str,
    version: Option<VersionCheck>,
}

impl Check {
    pub const fn new(name: &'static str, version: Option<VersionCheck>) -> Self {
        Self { name, version }
    }
}

#[derive(Clone, Copy, Debug)]
struct VersionCheck {
    arg: &'static str,
    row: u8,
    col: u8,
}

impl VersionCheck {
    pub const fn new(arg: &'static str, row: u8, col: u8) -> Self {
        Self { arg, row, col }
    }
}

impl Check {
    fn name(self) -> &'static str {
        self.name
    }

    fn path(self) -> Result<PathBuf> {
        Ok(which::which(self.name)?)
    }

    fn version(self) -> Result<Option<String>> {
        if let Some(version) = self.version {
            let output = Command::new(self.name)
                .args(version.arg.split(' '))
                .output()?;
            anyhow::ensure!(output.status.success(), "failed to run {}", self.name);
            let output = std::str::from_utf8(&output.stdout)?;
            if let Some(line) = output.lines().nth(version.row as _) {
                let mut col = version.col as usize;
                if line.starts_with("Apple ") || line.starts_with("Homebrew ") {
                    col += 1;
                }
                if let Some(col) = line.split(' ').nth(col) {
                    return Ok(Some(col.to_string()));
                }
            }
            anyhow::bail!("failed to parse version: {:?}", output);
        } else {
            Ok(None)
        }
    }
}

pub fn doctor(fix: bool) {
    let doctor = Doctor::default();
    print!("{doctor}");
    let user = username();
    #[cfg(target_os = "windows")]
    let search_dirs = vec![
        format!(r"C:\Users\{}\AppData\Local\Android\Sdk\build-tools\35.0.0", user),
        format!(r"C:\Users\{}\AppData\Local\Android\Sdk\platform-tools", user),
        format!(r"C:\Users\{}\AppData\Local\Android\Sdk\cmdline-tools\latest\bin", user),
        format!(r"C:\Users\{}\AppData\Local\Android\Sdk\emulator", user),
        format!(r"C:\Users\{}\AppData\Local\Android\Sdk\tools\bin", user),
        format!(r"C:\Users\{}\AppData\Local\Android\Sdk\ndk\26.2.11394342", user),
        format!(r"C:\Users\{}\AppData\Local\Android\Sdk\ndk\25.1.8937393", user),
        format!(r"C:\Users\{}\AppData\Local\Android\Sdk\ndk\27.0.12077973", user),
        format!(r"C:\Users\{}\AppData\Local\Android\Sdk\ndk\24.0.8215888", user),
        format!(r"C:\Users\{}\AppData\Local\Android\Sdk\ndk\23.2.8568313", user),
        format!(r"C:\Users\{}\AppData\Local\Programs\Microsoft VS Code\bin", user),
        format!(r"C:\Users\{}\AppData\Roaming\Cargo\bin", user),
        format!(r"C:\Users\{}\AppData\Local\dotnet", user),
        format!(r"C:\Users\{}\AppData\Local\Android\Sdk\tools\bin", user),
        format!(r"C:\Users\{}\AppData\Local\Android\Sdk\emulator", user),
        format!(r"C:\Users\{}\AppData\Local\Android\Sdk\platform-tools", user),
        format!(r"C:\Users\{}\AppData\Local\Android\Sdk\cmdline-tools\latest\bin", user),
        format!(r"C:\Users\{}\AppData\Local\dotnet\tools", user),
        format!(r"C:\Users\{}\AppData\Roaming\Cargo\bin", user),
        format!(r"C:\Users\{}\AppData\Local\dotnet", user),
        r"C:\Program Files\Microsoft\jdk-21.0.8.9-hotspot\bin".to_string(),
        r"C:\Program Files\Android\Android Studio\plugins\Kotlin\kotlinc\bin".to_string(),
        r"C:\Program Files (x86)\NSIS".to_string(),
        r"C:\Program Files\LLVM\bin".to_string(),
        r"C:\Program Files\Android\Android Studio\plugins\gradle\lib".to_string(),
    ];

    #[cfg(target_os = "linux")]
    let search_dirs = vec![
        format!("/home/{}/.local/share/Android/Sdk/build-tools/35.0.0", user),
        format!("/home/{}/.local/share/Android/Sdk/platform-tools", user),
        format!("/home/{}/.local/share/Android/Sdk/cmdline-tools/latest/bin", user),
        format!("/home/{}/.local/share/Android/Sdk/emulator", user),
        format!("/home/{}/.local/share/Android/Sdk/tools/bin", user),
        format!("/home/{}/.local/share/Android/Sdk/ndk/26.2.11394342", user),
        format!("/home/{}/.local/share/Android/Sdk/ndk/25.1.8937393", user),
        format!("/home/{}/.local/share/Android/Sdk/ndk/27.0.12077973", user),
        format!("/home/{}/.cargo/bin", user),
        format!("/home/{}/.local/bin", user),
        "/usr/lib/llvm/bin".to_string(),
        "/usr/lib/android-studio/plugins/Kotlin/kotlinc/bin".to_string(),
        "/usr/lib/android-studio/plugins/gradle/lib".to_string(),
        "/usr/lib/jvm/default/bin".to_string(),
        "/usr/bin".to_string(),
        "/usr/local/bin".to_string(),
    ];
    if fix {
        println!("Attempting to find tools and add to path");
        Doctor::fix(&doctor, search_dirs);
    }
}