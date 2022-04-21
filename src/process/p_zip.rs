use crate::{
    core::{c_zip::CompressZip, Compress},
    extra::try_send_message,
};

use super::Process;

pub struct ProcessZip;

impl Process for ProcessZip {
    fn process<T: AsRef<std::path::Path>, O: AsRef<std::path::Path>>(
        queue: std::sync::Arc<crossbeam_queue::SegQueue<T>>,
        dest: std::sync::Arc<O>,
        sender: Option<std::sync::mpsc::Sender<String>>,
    ) {
        let dest = &*dest;
        while !queue.is_empty() {
            let dir = match queue.pop() {
                None => break,
                Some(d) => d,
            };

            match CompressZip::compress(dir, dest) {
                Ok(p) => try_send_message(
                    &sender,
                    format!("zip archiving complete: {}", p.to_str().unwrap()),
                ),
                Err(e) => try_send_message(&sender, format!("zip archiving error occured!: {}", e)),
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::core::test_util::{cleanup, setup, Dir};
    use crate::extra::get_dir_list;
    use crossbeam_queue::SegQueue;
    use function_name::named;
    use std::sync::{mpsc, Arc};
    use std::thread;

    #[test]
    #[named]
    fn process_zip_test() {
        let Dir { origin, dest } = setup(function_name!());
        let raw_vec = get_dir_list(origin).unwrap();
        let queue = SegQueue::new();
        for i in raw_vec {
            queue.push(i);
        }
        let (tx, tr) = mpsc::channel();

        let dest = Arc::new(dest);
        thread::spawn(move || {
            ProcessZip::process(Arc::new(queue), dest, Some(tx));
        });

        let mut message = vec![];
        for re in tr {
            message.push(re);
        }

        let mut expected_message = vec![
            "zip archiving complete: test_dest_process_zip_test/dir1.zip",
            "zip archiving complete: test_dest_process_zip_test/dir2.zip",
            "zip archiving complete: test_dest_process_zip_test/dir3.zip",
        ];

        message.sort();
        expected_message.sort();

        assert_eq!(message, expected_message);
        cleanup(function_name!());
    }
}
