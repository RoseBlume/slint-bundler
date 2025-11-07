

const HELP_TOP: &str = concat!(
    "build: Build and bundle the project\n",
    "dev: Run the project in dev mode (auto-recompile on change)\n",
    "icon: Generate icons from PNG input\n",
    "doctor: Check environment setup\n",
    "android: Run an android subcommand\n",
);

const HELP_BUILD: &str = "build: Build and bundle the project";
const HELP_DEV: &str = "dev: Run the project in dev mode (auto-recompile on change)";
const HELP_ICON: &str = "icon: Generate icons from PNG input\nUsage: slint-bundler icon /path/to/image.png (Must be 1024x1024px)";
const HELP_DOCTOR: &str = "doctor: Check environment setup";

const HELP_ANDROID_ROOT: &str = concat!(
    "android: Run an android subcommand",
    "\nUsage: slint-bundler android <subcommand>",
    "\n\nSubcommands:",
    "\n\tinit: Initialize an android project",
    "\n\tbuild: Build an android apk",
    "\n\tdev: Run the project in dev mode on an android device or emulator (auto-recompile on change)",
    "\n\tkey: Run an android key subcommand"
);

const HELP_ANDROID_INIT: &str = "init: Initialize an android project";
const HELP_ANDROID_BUILD: &str = "build: Build an android apk";
const HELP_ANDROID_DEV: &str = "dev: Run the project in dev mode on an android device or emulator (auto-recompile on change)";

const HELP_ANDROID_KEY_ROOT: &str = concat!(
    "key: Run an android key subcommand",
    "\nUsage: slint-bundler android key <subcommand>",
    "\n\nSubcommands:",
    "\n\tgenerate: Generate a keystore to sign",
    "\n\tsign: Sign an apk bundle"
);

const HELP_ANDROID_KEY_GENERATE: &str = "generate: Generate a keystore to sign";
const HELP_ANDROID_KEY_SIGN: &str = "sign: Sign an apk bundle";

/// Walk the provided args slice and return the most-specific help message available.
/// Expects `args` to be the CLI arguments (for example argv[1..]).
fn final_recognized_command(args: &[String]) -> &'static str {
    if args.is_empty() {
        return HELP_TOP;
    }

    // helper to get arg at index if present
    let get = |i: usize| args.get(i).map(|s| s.as_str());

    match get(0) {
        Some("build") => HELP_BUILD,
        Some("dev") => HELP_DEV,
        Some("icon") => HELP_ICON,
        Some("doctor") => HELP_DOCTOR,
        Some("android") => {
            match get(1) {
                Some("init") => HELP_ANDROID_INIT,
                Some("build") => HELP_ANDROID_BUILD,
                Some("dev") => HELP_ANDROID_DEV,
                Some("key") => {
                    match get(2) {
                        Some("generate") => HELP_ANDROID_KEY_GENERATE,
                        Some("sign") => HELP_ANDROID_KEY_SIGN,
                        _ => HELP_ANDROID_KEY_ROOT,
                    }
                }
                _ => HELP_ANDROID_ROOT,
            }
        }
        _ => HELP_TOP,
    }
}

/// Return a help message string appropriate for the provided args.
pub fn generate_help_message(args: &[String]) -> String {
    final_recognized_command(args).to_string()
}