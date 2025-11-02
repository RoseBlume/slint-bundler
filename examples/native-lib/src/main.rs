
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

fn main() {
    run().expect("Failed to run on desktop");
}