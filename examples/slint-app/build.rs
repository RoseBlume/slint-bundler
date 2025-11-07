fn main() {
    slint_build::compile("ui/app-window.slint").expect("Slint build failed");
    // slint_build::compile("ui/list-area.slint").expect("Slint build failed");
}