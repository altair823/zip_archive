use crossbeam_queue::SegQueue;
use std::sync::mpsc::Sender;
use std::sync::Arc;

use crate::core::c_7z::Compress7z;
use crate::core::Compress;
use crate::extra::try_send_message;

use super::Process;

pub struct Process7z;

impl Process for Process7z {
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
            match Compress7z::compress(&dir, &dest) {
                Ok(p) => try_send_message(
                    &sender,
                    format!("7z archiving complete: {}", p.to_str().unwrap()),
                ),
                Err(e) => try_send_message(&sender, format!("7z archiving error occured!: {}", e)),
            };
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
    use std::sync::mpsc;
    use std::thread;

    #[test]
    #[named]
    fn process_7z_test() {
        let Dir { origin, dest } = setup(function_name!());
        let raw_vec = get_dir_list(origin).unwrap();
        let queue = SegQueue::new();
        for i in raw_vec {
            queue.push(i);
        }
        let (tx, tr) = mpsc::channel();

        let dest = Arc::new(dest);
        thread::spawn(move || {
            Process7z::process(Arc::new(queue), dest, Some(tx));
        });

        let mut message = vec![];
        for re in tr {
            message.push(re);
        }

        let mut expected_message = vec![
            "7z archiving complete: test_dest_process_7z_test/dir2.7z",
            "7z archiving complete: test_dest_process_7z_test/dir3.7z",
            "7z archiving complete: test_dest_process_7z_test/dir1.7z",
        ];

        message.sort();
        expected_message.sort();

        assert_eq!(message, expected_message);
        cleanup(function_name!());
    }
}
