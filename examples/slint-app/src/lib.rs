use slint::{ToSharedString, SharedString, ModelRc, VecModel};
use std::env;
use std::fs;
use std::result::Result;
use std::rc::Rc;
mod player;
use std::path::PathBuf;
use player::play_song;
// mod list;


// fn create_component() -> ItMoss {
//     ItMoss::new().expect("Failed to create List item")
// }
slint::include_modules!();
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub fn android_main(app: slint::android::AndroidApp) -> Result<(), Box<dyn std::error::Error>> {
    //slint::platform::set_platform(Box::new(i_slint_backend_android_activity::Backend::new().unwrap()));
    std::env::set_var("SLINT_FULLSCREEN", "true");
    slint::android::init(app).unwrap();
    run()
}
// Test Test


pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let ui = AppWindow::new().expect("Failed to run AppWindow");
    ui.on_request_play({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            let file_path: String = ui.get_filepath().into();
            let path: PathBuf = PathBuf::from(file_path);
            if path.exists() {
                play_song(&path.display().to_string());
            }
        }
    });
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
    {
        let ui_handle = ui.as_weak();
        let ui = ui_handle.unwrap();
        ui.set_info(ModelRc::from(Rc::new(print_music_file_paths())));
            // list::run(ui.get_counter());
            // ui.set_info("Test".to_shared_string());
    }
    println!("{}", find_music_files());
    ui.run()?;
    Ok(())
}

fn find_music_files() -> &'static str {
    let path = TEST_PATHS;
    match std::fs::write(path, "123456") {
        Ok(_) => { "SUCCESS WRITING ID" } 
        Err(_) => { 
            println!("Failure to write");
            "Failure to write"

        }
    }
}

pub fn find_music_folder() -> String {
    #[cfg(target_os = "android")] {
        "/storage/emulated/0/Music".to_string()
    }

    #[cfg(not(target_os = "android"))]{
        let username = whoami::username();
        #[cfg(target_os = "windows")] {
            format!("C:\\Users\\{}\\Music", username)
        }
        #[cfg(target_os = "linux")] {
            format!("/home/{}/Music", username)
        }
    }
}


pub fn print_music_file_paths() -> VecModel<SharedString> {
    // let mut final_paths: Vec<&str> = vec![];
    let my_vec = get_music_file_paths();
    let new_vec: VecModel<SharedString> = VecModel::from(vec!["".to_shared_string()]);
    for i in my_vec {
         new_vec.push(i.to_shared_string());
         println!("{}", i);
     }
    new_vec

}

fn get_music_file_paths() -> Vec<String> {
    let mut my_string = String::from("");
    let paths = fs::read_dir(find_music_folder()).unwrap();

    // input.split(',').map(|s| s.trim().to_string()).collect()
    for path in paths {
        //let path_name = path.unwrap().path().display().to_string(); //
        my_string.push_str(&path.unwrap().path().display().to_string());
        my_string.push('\n');
        // final_paths.push(&(path.unwrap().path().display().to_string()));
        // println!("Name: {}", path.unwrap().path().display());
    }
    let my_vec: Vec<String> = my_string.split('\n').map(|s| s.trim().to_string()).collect();
    my_vec
}
const TEST_PATHS: &str = "/storage/emulated/0/Documents/id.txt";