#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//use slint_lib::run;
#[cfg(target_os = "windows")]
use i_slint_backend_winit::Backend;

#[cfg(target_os = "linux")]
use i_slint_backend_linuxkms::Backend;


/*
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();
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
*/
fn main() {
    slint::platform::set_platform(Box::new(Backend::new().unwrap())).expect("Failed to select a backend");
    std::env::set_var("SLINT_MAXIMIZED", "true");
    slinterlib::run().expect("Failed to run on desktop");
}