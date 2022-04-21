use std::io;
use std::{
    ffi::{OsStr, OsString},
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use xz2::write::XzEncoder;

use super::Compress;

fn append_ext(ext: impl AsRef<OsStr>, path: PathBuf) -> PathBuf {
    let mut os_string: OsString = path.into();
    os_string.push(".");
    os_string.push(ext.as_ref());
    os_string.into()
}

pub struct CompressXz;

impl Compress for CompressXz {
    fn compress<T: AsRef<Path>, O: AsRef<Path>>(origin: T, dest: O) -> Result<PathBuf, io::Error> {
        if !origin.as_ref().is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "The origin is not a file!",
            ));
        }
        let mut origin_file = File::open(&origin)?;
        let mut dest = dest.as_ref().join(&origin.as_ref().file_name().unwrap());

        dest = append_ext("xz", dest);
        let dest_file = File::create(&dest)?;

        let mut encoder = XzEncoder::new(dest_file, 9);
        let mut content = Vec::new();
        origin_file.read_to_end(&mut content)?;
        encoder.write_all(&content)?;
        encoder.finish()?;
        return Ok(dest);
    }
}

#[cfg(test)]
mod tests {
    use function_name::named;

    use super::super::c_tar::CompressTar;
    use super::*;
    use crate::core::test_util::{cleanup, setup, Dir};

    #[test]
    #[named]
    fn compress_xz_test() {
        let Dir { origin, dest } = setup(function_name!());
        let tar_path = CompressTar::compress(&origin, &dest).unwrap();
        CompressXz::compress(&tar_path, dest).unwrap();

        assert!(tar_path.is_file());
        assert!(Path::new(&format!("{}.xz", &tar_path.to_str().unwrap())).is_file());
        cleanup(function_name!());
    }
}
