use std::{
    fs,
    sync::{mpsc::Sender, Arc},
};

use crossbeam_queue::SegQueue;

use crate::{
    core::{c_tar::CompressTar, c_xz::CompressXz, Compress},
    extra::try_send_message,
};

use super::Process;

pub struct ProcessXz;

impl Process for ProcessXz {
    fn process<T: AsRef<std::path::Path>, O: AsRef<std::path::Path>>(
        queue: Arc<SegQueue<T>>,
        dest: Arc<O>,
        sender: Option<Sender<String>>,
    ) {
        let dest = &*dest;
        while !queue.is_empty() {
            let dir = match queue.pop() {
                None => break,
                Some(d) => d,
            };
            let tar_path = match CompressTar::compress(&dir, &dest) {
                Ok(p) => p,
                Err(e) => {
                    try_send_message(&sender, format!("Cannot create tarball!: {}", e));
                    return;
                }
            };
            match CompressXz::compress(&tar_path, &dest) {
                Ok(p) => {
                    match fs::remove_file(&tar_path) {
                        Ok(_) => (),
                        Err(_) => try_send_message(&sender, format!("Cannot delete tarball!")),
                    };
                    try_send_message(
                        &sender,
                        format!("xz archiving complete: {}", p.to_str().unwrap()),
                    );
                }
                Err(e) => try_send_message(&sender, format!("xz archiving error occured!: {}", e)),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::mpsc, thread};

    use function_name::named;

    use crate::{
        core::test_util::{cleanup, setup, Dir},
        get_dir_list,
    };

    use super::*;

    #[test]
    #[named]
    fn process_xz_test() {
        let Dir { origin, dest } = setup(function_name!());
        let raw_vec = get_dir_list(origin).unwrap();
        let queue = SegQueue::new();
        for i in raw_vec {
            queue.push(i);
        }
        let (tx, tr) = mpsc::channel();

        let dest = Arc::new(dest);
        thread::spawn(move || {
            ProcessXz::process(Arc::new(queue), dest, Some(tx));
        });

        let mut message = vec![];
        for re in tr {
            message.push(re);
        }
        println!("{:?}", message);
        let mut expected_message = vec![
            "xz archiving complete: test_dest_process_xz_test/dir2.tar.xz",
            "xz archiving complete: test_dest_process_xz_test/dir3.tar.xz",
            "xz archiving complete: test_dest_process_xz_test/dir1.tar.xz",
        ];

        message.sort();
        expected_message.sort();

        assert_eq!(message, expected_message);
        cleanup(function_name!());
    }
}
