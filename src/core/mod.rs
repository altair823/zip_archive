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

    pub struct Dir {
        pub origin: PathBuf,
        pub dest: PathBuf,
    }

    pub fn setup(test_name: &str) -> Dir {
        let test_origin = PathBuf::from(format!("test_origin_{}", test_name));
        let test_dest = PathBuf::from(format!("test_dest_{}", test_name));

        if test_origin.is_dir() {
            fs::remove_dir_all(&test_origin).unwrap();
        }
        if test_dest.is_dir() {
            fs::remove_dir_all(&test_dest).unwrap();
        }

        fs::create_dir_all(&test_origin).unwrap();
        fs::create_dir_all(&test_dest).unwrap();

        let dir_list = get_dir_list("original_images").unwrap();
        let option = CopyOptions::new();
        for i in dir_list {
            dir::copy(i, &test_origin, &option).unwrap();
        }

        Dir {
            origin: test_origin,
            dest: test_dest,
        }
    }

    pub fn cleanup(test_name: &str) {
        let test_origin = PathBuf::from(format!("test_origin_{}", test_name));
        let test_dest = PathBuf::from(format!("test_dest_{}", test_name));

        if test_origin.is_dir() {
            fs::remove_dir_all(&test_origin).unwrap();
        }
        if test_dest.is_dir() {
            fs::remove_dir_all(&test_dest).unwrap();
        }
    }
}
