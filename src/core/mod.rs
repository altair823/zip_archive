pub mod c_7z;
pub mod c_tar;
pub mod c_xz;

#[cfg(test)]
pub mod test_util {
    use crate::extra::get_dir_list;
    use fs_extra::dir;
    use fs_extra::dir::CopyOptions;

    use std::fs;
    use std::path::PathBuf;

    pub fn setup() -> (PathBuf, PathBuf) {
        let mut i = 1;
        let mut test_origin = PathBuf::from(format!("test_origin{}", i));
        while test_origin.is_dir() {
            i += 1;
            test_origin = PathBuf::from(format!("test_origin{}", i));
        }
        let test_dest = PathBuf::from(format!("test_dest{}", i));

        fs::create_dir_all(&test_origin).unwrap();
        fs::create_dir_all(&test_dest).unwrap();

        let dir_list = get_dir_list("original_images").unwrap();
        let option = CopyOptions::new();
        for i in dir_list {
            dir::copy(i, &test_origin, &option).unwrap();
        }

        (test_origin, test_dest)
    }
}
