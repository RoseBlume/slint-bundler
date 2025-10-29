// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::env;

slint::include_modules!();
#[unsafe(no_mangle)]
pub fn android_main(app: slint::android::AndroidApp) -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("SLINT_FULLSCREEN", "true");
    slint::android::init(app).unwrap();
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
// Test Test