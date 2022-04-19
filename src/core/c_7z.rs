use std::error::Error;
use std::io;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use subprocess::Exec;

use crate::extra::{get_7z_executable_path, get_file_list};

pub fn compress_7z(origin: &Path, dest: &Path) -> Result<PathBuf, Box<dyn Error>> {
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
        match PathBuf::from("./").join(origin).to_str() {
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
mod test {
    use crate::core::test_util::setup;

    use super::*;

    #[test]
    fn compress_to_7z_test() {
        let (mut origin, dest) = setup();
        compress_7z(origin.as_path(), dest.as_path()).unwrap();
        origin.set_extension("7z");
        assert!(dest.join(origin).is_file());
    }
}
