mod desktop;
mod bundle;
mod dev;
mod icon;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "slint-bundler")]
#[command(about = "Build and bundle Rust projects for multiple platforms", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build and bundle the project
    Build {
        /// List of bundles to create (e.g. deb msi rpm)
        #[arg(long)]
        bundles: Option<Vec<String>>,
    /// Try to create bundles for other architectures (presence-only flag).
    /// When present, the bundler will attempt to produce bundles for all
    /// known architectures that are not the host architecture
    /// (i386, x86_64, aarch64, armhf, riscv64).
    #[arg(long = "cross-arch", action = clap::ArgAction::SetTrue)]
    cross_arch: bool,
        /// Cross compilation method: 'docker' or 'qemu'
        #[arg(long = "cross-method")]
        cross_method: Option<String>,
    },
    /// Run the project in dev mode (auto-recompile on change)
    Dev,
    Icon {
        /// Input PNG file (1024x1024)
        #[arg(long)]
        input: String,
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
    Commands::Build { bundles, cross_arch, cross_method } => bundle::handle_build(bundles, cross_arch, cross_method),
        Commands::Dev => dev::handle_dev(),
        Commands::Icon { input } => {
            if let Err(e) = icon::generate_pngs(&input) {
                eprintln!("Error generating icons: {}", e);
            }
        }
    }
}
