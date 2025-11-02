
use std::env;

slint::include_modules!();
#[unsafe(no_mangle)]
pub fn android_main(app: slint::android::AndroidApp) -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("SLINT_FULLSCREEN", "true");
    slint::android::init(app).unwrap();
    run()
}
// Test Test


fn run() -> Result<(), Box<dyn std::error::Error>> {
    let ui = AppWindow::new()?;
    ui.on_request_increase_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() + 1);
        }
    });
    ui.on_request_decrease_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() - 1);
        }
    });

    ui.run()?;

    Ok(())
}