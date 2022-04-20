use std::io;
use std::{
    error::Error,
    fs::File,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use tar::Builder;

pub fn make_tar(origin: &Path, dest: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let mut tar_path = dest.join(&match origin.file_name() {
        Some(p) => p,
        None => origin.as_os_str(),
    });
    tar_path.set_extension("tar");

    if tar_path.is_file() {
        return Err(Box::new(io::Error::new(
            ErrorKind::AlreadyExists,
            "The tar file already exists!",
        )));
    }

    let tar_file = File::create(&tar_path)?;
    let mut tar_builder = Builder::new(tar_file);
    tar_builder.append_dir_all(origin.file_name().unwrap(), origin)?;

    return Ok(tar_path);
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
        make_tar(origin.as_path(), dest.as_path()).unwrap();
        origin.set_extension("tar");
        assert!(dest.join(origin).is_file());
        cleanup(function_name!());
    }
}
