
use std::path::{Path, PathBuf};
use std::error::Error;
use std::io;
use std::io::ErrorKind;
use subprocess::Exec;

use crate::extra::get_7z_executable_path;



pub fn compress_a_dir_to_7z(origin: &Path, dest: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let compressor_path = get_7z_executable_path()?;

    let mut zip_path = dest.join(&match origin.file_name() {
        Some(p) => p,
        _ => origin.as_os_str(),
    });
    zip_path.set_extension("7z");

    if zip_path.is_file() {
        return Err(Box::new(io::Error::new(
            ErrorKind::AlreadyExists,
            "The 7z archive file already exists!",
        )));
    }

    let exec = Exec::cmd(compressor_path).args(&vec![
        "a",
        "-mx=9",
        "-t7z",
        zip_path.to_str().unwrap(),
        match origin.to_str() {
            None => {
                return Err(Box::new(io::Error::new(
                    ErrorKind::NotFound,
                    "Cannot get the destination directory path!",
                )))
            }
            Some(s) => s,
        },
    ]);
    exec.join()?;
    return Ok(zip_path);
}


#[cfg(test)]
mod test{
    use super::*;
    use std::fs;
    use fs_extra::dir;
    use fs_extra::dir::CopyOptions;

    fn setup() -> (PathBuf, PathBuf){
        let mut i = 1;
        let mut test_origin = PathBuf::from(format!("test_origin{}", i));
        while test_origin.is_dir() {
            i += 1;
            test_origin = PathBuf::from(format!("test_origin{}", i));
        }
        let test_dest = PathBuf::from(format!("test_dest{}", i));
        
        fs::create_dir_all(&test_origin).unwrap();
        fs::create_dir_all(&test_dest).unwrap();

        let option = CopyOptions::new();
        dir::copy("original_images", &test_origin, &option).unwrap();

        (test_origin, test_dest)
    }

    #[test]
    fn compress_a_dir_to_7z_test(){
        let (origin, dest) = setup();
        compress_a_dir_to_7z(origin.join("original_images").as_path(), dest.as_path()).unwrap();
        assert!(dest.join("original_images.7z").is_file());
    }
}