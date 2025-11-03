use std::path::Path;
use std::fs;

pub fn find_build_tools() -> String {
    //C:\Users\James\AppData\Local\Android\Sdk\build-tools\36.0.0
    let username = whoami::username();
    let path = format!("C:\\Users\\{}\\AppData\\Local\\Android\\Sdk\\build-tools", username);
    
        //let mut dir: DirEntry;
        // for entry in fs::read_dir(".")? {
        //    let dir = entry?;
        //     println!("{:?}", dir.path());
        // }
    let mut paths = fs::read_dir(&path).unwrap();
    let relative_path = paths.next().unwrap().expect("Failed to unwrap path").path();
    let final_path = Path::new(&path).join(relative_path).display().to_string();
    final_path
}