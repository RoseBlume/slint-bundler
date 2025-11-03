
const GRADLE_JAR: &[u8; 45457] = include_bytes!("./gradle-wrapper.jar");
pub fn unpack_gradle_jar() {
    std::fs::write("android/gradle/wrapper/gradle-wrapper.jar", GRADLE_JAR).expect("Failed to write gradle wrapper");
}