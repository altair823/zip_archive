use std::io;
use std::{
    fs::File,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use tar::Builder;

use super::Compress;

pub struct CompressTar;

impl Compress for CompressTar {
    fn compress<T: AsRef<Path>, O: AsRef<Path>>(origin: T, dest: O) -> Result<PathBuf, io::Error> {
        let mut tar_path = dest.as_ref().join(&match origin.as_ref().file_name() {
            Some(p) => p,
            None => origin.as_ref().as_os_str(),
        });
        tar_path.set_extension("tar");

        if tar_path.is_file() {
            return Err(io::Error::new(
                ErrorKind::AlreadyExists,
                "The tar file already exists!",
            ));
        }

        let tar_file = File::create(&tar_path)?;
        let mut tar_builder = Builder::new(tar_file);
        tar_builder.append_dir_all(origin.as_ref().file_name().unwrap(), origin.as_ref())?;

        return Ok(tar_path);
    }
}

#[cfg(test)]
mod tests {
    use function_name::named;

    use crate::core::test_util::{cleanup, setup, Dir};

    use super::*;
    #[test]
    #[named]
    fn make_tar_test() {
        let Dir { mut origin, dest } = setup(function_name!());
        CompressTar::compress(origin.as_path(), dest.as_path()).unwrap();
        origin.set_extension("tar");
        assert!(dest.join(origin).is_file());
        cleanup(function_name!());
    }
}
