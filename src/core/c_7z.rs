use std::io;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use subprocess::Exec;

use crate::extra::get_7z_executable_path;

use super::Compress;

pub struct Compress7z;

impl Compress for Compress7z {
    fn compress<T: AsRef<Path>, O: AsRef<Path>>(origin: T, dest: O) -> Result<PathBuf, io::Error> {
        let compressor_path = get_7z_executable_path()?;

        let mut zip_path = dest.as_ref().join(&match origin.as_ref().file_name() {
            Some(p) => p,
            _ => origin.as_ref().as_os_str(),
        });
        zip_path.set_extension("7z");

        if zip_path.is_file() {
            return Err(io::Error::new(
                ErrorKind::AlreadyExists,
                "The 7z archive file already exists!",
            ));
        }

        let exec = Exec::cmd(compressor_path).args(&vec![
            "a",
            "-mx=9",
            "-t7z",
            zip_path.to_str().unwrap(),
            match PathBuf::from("./").join(origin).to_str() {
                None => {
                    return Err(io::Error::new(
                        ErrorKind::NotFound,
                        "Cannot get the destination directory path!",
                    ))
                }
                Some(s) => s,
            },
        ]);
        match exec.join() {
            Ok(_) => (),
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    format!("Cannot execute subprocess!: {}", e),
                ))
            }
        };
        return Ok(zip_path);
    }
}

#[cfg(test)]
mod test {
    use function_name::named;

    use crate::core::test_util::{cleanup, setup, Dir};

    use super::*;

    #[test]
    #[named]
    fn compress_to_7z_test() {
        let Dir { mut origin, dest } = setup(function_name!());
        Compress7z::compress(origin.as_path(), dest.as_path()).unwrap();
        origin.set_extension("7z");
        assert!(dest.join(origin).is_file());
        cleanup(function_name!());
    }
}
