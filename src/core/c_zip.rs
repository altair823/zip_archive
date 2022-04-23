use std::{
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use zip::{write::FileOptions, ZipWriter};

use crate::extra::get_file_list;

use super::Compress;

fn get_content_vec<T: AsRef<Path>>(path: T) -> Result<Vec<u8>, io::Error> {
    let mut file = File::open(path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    Ok(content)
}

pub struct CompressZip;

impl Compress for CompressZip {
    fn compress<T: AsRef<Path>, O: AsRef<Path>>(origin: T, dest: O) -> Result<PathBuf, io::Error> {
        let mut zip_file_name =
            PathBuf::from(dest.as_ref().join(&origin.as_ref().file_name().unwrap()));
        zip_file_name.set_extension("zip");
        let zip_file = File::create(&zip_file_name)?;

        let mut zip_writer = ZipWriter::new(zip_file);
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for file in get_file_list(&origin)? {
            let content = get_content_vec(&file)?;
            zip_writer.start_file(
                file.strip_prefix(&origin.as_ref().parent().unwrap())
                    .unwrap()
                    .to_str()
                    .unwrap(),
                options,
            )?;
            zip_writer.write_all(&content)?;
        }

        zip_writer.finish()?;

        return Ok(zip_file_name);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use function_name::named;
    use std::{
        fs::File,
        io::{Read, Write},
    };
    use zip::write::FileOptions;

    use crate::core::test_util::{cleanup, setup, Dir};

    #[named]
    fn lib_test() {
        let Dir { origin, dest } = setup(function_name!());

        // We use a buffer here, though you'd normally use a `File`
        let buf = File::create(dest.join("test.zip")).unwrap();
        let mut zip = zip::ZipWriter::new(buf);

        let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

        zip.start_file(
            origin.join("dir1").join("file3.png").to_str().unwrap(),
            options,
        )
        .unwrap();
        let mut content = Vec::new();
        let mut origin_file = File::open(origin.join("dir1").join("file3.png")).unwrap();
        origin_file.read_to_end(&mut content).unwrap();
        zip.write_all(&content).unwrap();

        zip.start_file(
            origin.join("dir1").join("file5.webp").to_str().unwrap(),
            options,
        )
        .unwrap();
        let mut content = Vec::new();
        let mut origin_file = File::open(origin.join("dir1").join("file5.webp")).unwrap();
        origin_file.read_to_end(&mut content).unwrap();
        zip.write_all(&content).unwrap();

        // Apply the changes you've made.
        // Dropping the `ZipWriter` will have the same effect, but may silently fail
        zip.finish().unwrap();
    }

    #[test]
    #[named]
    fn compress_zip_test() {
        let Dir { mut origin, dest } = setup(function_name!());
        CompressZip::compress(&origin, &dest).unwrap();
        origin.set_extension("zip");
        assert!(dest.join(origin).is_file());
        cleanup(function_name!())
    }
}
