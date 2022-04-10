use std::env::consts::OS;
use std::error::Error;
use std::io;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

pub fn send_message<T: ToString>(sender: &Sender<T>, message: T){
    match sender.send(message) {
        Ok(_) => (),
        Err(e) => println!("Message passing error!: {}", e),
    }
}

pub fn try_send_message<T: ToString>(sender: &Option<Sender<T>>, message: T){
    match sender{
        Some(s) => send_message(s, message),
        None => (),
    }
}

/// Get list of all subdirectories in the rood directory. Not recursive.
/// 
/// # Examples
/// ```
/// use zip_archive::get_dir_list;
/// use std::path::PathBuf;
/// use std::fs::create_dir_all;
/// 
/// let dir = PathBuf::from("dir_test/dir1"); 
/// create_dir_all(&dir).unwrap();
/// assert_eq!(get_dir_list("dir_test").unwrap(), vec![PathBuf::from("dir_test/dir1")]);
/// ```
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
/// Get a list of directories at a specific depth among all subdirectories of the rood directory.
/// 
/// # Examples
/// ```
/// use zip_archive::get_dir_list_with_depth;
/// use std::path::PathBuf;
/// use std::fs::create_dir_all;
/// 
/// let dir = PathBuf::from("dir_test/dir1/dir2/dir3"); 
/// create_dir_all(&dir).unwrap();
/// assert_eq!(get_dir_list_with_depth("dir_test", 0).unwrap(), vec![PathBuf::from("dir_test")]);
/// assert_eq!(get_dir_list_with_depth("dir_test", 1).unwrap(), vec![PathBuf::from("dir_test/dir1")]);
/// assert_eq!(get_dir_list_with_depth("dir_test", 2).unwrap(), vec![PathBuf::from("dir_test/dir1/dir2")]);
/// assert_eq!(get_dir_list_with_depth("dir_test", 3).unwrap(), vec![PathBuf::from("dir_test/dir1/dir2/dir3")]);
/// ```
pub fn get_dir_list_with_depth<O: AsRef<Path>>(root: O, depth: u32) -> io::Result<Vec<PathBuf>> {
    if depth == 0 {
        return Ok(vec![root.as_ref().to_path_buf()])
    }
    let depth = depth - 1;

    let mut result = Vec::new();
    let cur_list = root.as_ref()
        .read_dir()?
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    let dir_list = cur_list
        .iter()
        .filter(|p| p.is_dir())
        .map(|p| p.to_path_buf())
        .collect::<Vec<_>>();
    
    if depth == 0{
        for dir in dir_list{
            result.push(dir);
        }
    }
    else {
        for dir in dir_list{
            result.append(&mut get_dir_list_with_depth(dir, depth)?);
        }
    }
    return Ok(result)

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

    #[test]
    fn get_dir_list_with_depth_test(){
        println!("{:?}", get_dir_list_with_depth("dir_test", 4).unwrap());
    }
}
