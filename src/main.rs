use clap_lib::{
    App,
    Command,
    Arg
};
use crate::android::{
    begin_build,
    unpack_gradle_jar,
    create_jni_dirs,
    initialize_android_project,
    handle_android_dev,
    generate_key,
    sign_bundle
};

mod bundle;
mod dev;
mod icon;
mod doctor;
mod android;
mod new;
pub mod utils;



fn main(){
    #[cfg(target_os = "linux")]
    let bundles_arg = Arg::option("bundles")
        .desc("Comma-separated bundle types (deb,rpm,tar.zst,tar.xz,standalone)")
        .default("deb,rpm,tar.zst,tar.xz,standalone");
    #[cfg(target_os = "windows")]
    let bundles_arg = Arg::option("bundles")
        .desc("Comma-separated bundle types (msi,nsis,msix)")
        .default("msi,nsis,msix");

    App::new("slint-bundler")
        .desc("Build and bundle Slint applications")
        .subcommand(
            Command::new("new")
                .run(|_| {
                    new::handle_new()
                })
        )
        .subcommand(
            Command::new("build")
                .desc("Build and bundle the project")
                .arg(bundles_arg)
                .run(|ctx| {
                    let bundles = parse_bundles(ctx.value("bundles"));
                    bundle::handle_build(bundles);
                    
                })
        )
        .subcommand(
            Command::new("dev")
                .desc("Run the project in dev mode (auto-recompile on change)")
                .run(|_| {
                    dev::handle_dev()
                })
        )
        .subcommand(
            Command::new("icon")
                .desc("Generate icons from PNG input")
                .arg(
                    Arg::option("input")
                    .desc("Usage: --input <icon>.png")    
                    .required(true)
                )
                .run(| ctx | {
                    let input: &str = ctx.value("input");
                    icon::generate_pngs(input).expect("Failed to generate pngs");
                })
        )
        .subcommand(
            Command::new("doctor")
                .desc("Check environment setup")
                .arg(
                    Arg::flag("fix")
                        .desc("Defines whether to automatically add to the path")
                )
                .run(| ctx | {
                    let fix = ctx.flag("fix");
                    doctor::doctor(fix);
                })
        )
        .subcommand(
            Command::new("android")
                .desc("Asists in building, signing and devloping Android packages")
                .subcommand(
                    Command::new("init")
                        .desc("")
                        .run(|_| {
                            initialize_android_project().expect("Failed to initialize android project");
                            create_jni_dirs();
                            unpack_gradle_jar();
                        })
                )
                .subcommand(
                    Command::new("build")
                        .desc("")
                        .arg(
                            Arg::flag("debug")
                        )
                        .run(| ctx | {
                            let debug_flag = if ctx.flag("debug") { "" } else { "--release" };
                            begin_build(debug_flag);
                        })
                )
                .subcommand(
                    Command::new("dev")
                        .desc("")
                        .run(|_| {
                            handle_android_dev();
                        })
                )
                .subcommand(
                    Command::new("key")
                        .desc("Sign a bundle with a password")
                        .arg(
                            Arg::positional("action")
                                .desc("Action to perform: generate or sign")
                                .required(true)
                        )
                        .arg(
                            Arg::positional("password")
                                .desc("Password used for signing")
                                .required(true)
                        )
                        .run(|ctx| {
                            let action = ctx.value("action");
                            let password = ctx.value("password");
                            match action {
                                "generate" => generate_key(password),
                                "sign" => sign_bundle(password.to_string()),
                                _ => println!("Invalid action: {}\n Valid actions are generate, or sign", action),
                            }
                        })
                )
        )
        .run();
}



fn parse_bundles(arg: &str) -> Vec<String> {
    arg.split(',')
        .map(|s| s.trim().to_string())
        .collect()
}



