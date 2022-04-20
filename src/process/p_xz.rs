use std::{
    fs,
    path::PathBuf,
    sync::{mpsc::Sender, Arc},
};

use crossbeam_queue::SegQueue;

use crate::{
    core::{c_tar::make_tar, c_xz::compress_xz},
    extra::try_send_message,
};

pub fn process(queue: Arc<SegQueue<PathBuf>>, dest: &PathBuf, sender: Option<Sender<String>>) {
    while !queue.is_empty() {
        let dir = match queue.pop() {
            None => break,
            Some(d) => d,
        };
        let tar_path = match make_tar(&dir, &dest) {
            Ok(p) => p,
            Err(e) => {
                try_send_message(&sender, format!("Cannot create tarball!: {}", e));
                return;
            }
        };
        match compress_xz(&tar_path, &dest) {
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

        thread::spawn(move || {
            process(Arc::new(queue), &dest, Some(tx));
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
