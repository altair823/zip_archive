use std::env::consts::OS;
use std::error::Error;
use std::io;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

pub fn send_message(sender: &Sender<String>, message: &str) {
    match sender.send(String::from(message)) {
        Ok(_) => (),
        Err(e) => println!("Message passing error!: {}", e),
    }
}

pub fn try_send_message(sender: &Option<Sender<String>>, message: &str){
    match sender{
        Some(s) => send_message(s, message),
        None => (),
    }
}

/// Get all directories list in the rood directory. Not recursive.
pub fn get_dir_list<O: AsRef<Path>>(root: O) -> io::Result<Vec<PathBuf>> {
    let cur_list: Vec<PathBuf> = root
        .as_ref()
        .read_dir()?
        .map(|entry| entry.unwrap().path())
        .collect();
    let dir_list = cur_list
        .iter()
        .filter(|p| p.is_dir())
        .map(|p| PathBuf::from(p.to_path_buf()))
        .collect::<Vec<_>>();

    Ok(dir_list)
}

pub fn get_7z_executable_path() -> Result<PathBuf, Box<dyn Error>> {
    match OS {
        "macos" => Ok(PathBuf::from("./7zz")),
        "windows" => Ok(PathBuf::from("7z.exe")),
        "linux" => Ok(PathBuf::from("./7zzs")),
        e => {
            println!("Doesn't support {} currently!", e);
            return Err(Box::new(io::Error::new(
                ErrorKind::NotFound,
                "Cannot find the 7z executable!",
            )));
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn get_7z_executable_path_test() {
        // The test will be passed if there is a 7z executable file in the root directory of the current project.
        assert!(get_7z_executable_path().unwrap().is_file());
    }
}
