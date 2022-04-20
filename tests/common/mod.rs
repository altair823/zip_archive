use fs_extra::dir::{self, CopyOptions};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::mpsc::{channel, Receiver, Sender},
};
use zip_archive::{get_dir_list, get_dir_list_with_depth, Archiver, Format};

pub struct Dir {
    pub origin: PathBuf,
    pub dest: PathBuf,
    pub tx: Sender<String>,
    pub tr: Receiver<String>,
}

pub fn setup(test_name: &str) -> Dir {
    let origin_dir_name = format!("integration_test_origin_{}", test_name);
    let origin = PathBuf::from(origin_dir_name);
    let dest_dir_name = format!("integration_test_dest_{}", test_name);
    let dest = PathBuf::from(dest_dir_name);

    if origin.is_dir() {
        fs::remove_dir_all(&origin).unwrap();
    }
    if dest.is_dir() {
        fs::remove_dir_all(&dest).unwrap();
    }

    fs::create_dir_all(&origin).unwrap();
    fs::create_dir_all(&dest).unwrap();

    let dir_list = get_dir_list("original_images").unwrap();
    let option = CopyOptions::new();
    for i in dir_list {
        dir::copy(i, &origin, &option).unwrap();
    }

    let (tx, tr) = channel();

    Dir {
        origin: origin,
        dest: dest,
        tx: tx,
        tr: tr,
    }
}

pub fn get_archiver<T: AsRef<Path>, O: AsRef<Path>>(
    origin: T,
    dest: O,
    tx: Sender<String>,
    format: Format,
) -> Archiver {
    let mut archiver = Archiver::new();
    archiver.set_destination(&dest);
    archiver.set_thread_count(3);
    archiver.push_from_iter(get_dir_list_with_depth(&origin, 1).unwrap().iter());
    archiver.set_sender(tx);
    archiver.set_format(format);

    archiver
}

pub fn cleanup(test_name: &str) {
    let test_origin = PathBuf::from(format!("integration_test_origin_{}", test_name));
    let test_dest = PathBuf::from(format!("integration_test_dest_{}", test_name));

    if test_origin.is_dir() {
        fs::remove_dir_all(&test_origin).unwrap();
    }
    if test_dest.is_dir() {
        fs::remove_dir_all(&test_dest).unwrap();
    }
}
