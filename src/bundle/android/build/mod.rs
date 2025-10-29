mod compile;
mod jni;
mod bundle;
use compile::build_android_targets;
use jni::create_symbolic_links;
use bundle::begin_gradle_build;
pub fn begin_build(mode: &str) {
    build_android_targets(mode).expect("Failed to build android targets");
    create_symbolic_links(mode);
    begin_gradle_build();
}