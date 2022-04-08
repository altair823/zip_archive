use crossbeam_queue::SegQueue;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::Arc;

use crate::core::compress_a_dir_to_7z;
use crate::extra::send_message;

pub fn process(queue: Arc<SegQueue<PathBuf>>, dest: &PathBuf) {
    while !queue.is_empty() {
        let dir = match queue.pop() {
            None => break,
            Some(d) => d,
        };
        match compress_a_dir_to_7z(dir.as_path(), &dest) {
            Ok(_) => {}
            Err(e) => println!("Error occurred! : {}", e),
        }
    }
}

pub fn process_with_sender(queue: Arc<SegQueue<PathBuf>>, dest: &PathBuf, sender: Sender<String>) {
    while !queue.is_empty() {
        let dir = match queue.pop() {
            None => break,
            Some(d) => d,
        };
        match compress_a_dir_to_7z(dir.as_path(), &dest) {
            Ok(p) => send_message(
                &sender,
                format!("7z archiving complete: {}", p.to_str().unwrap()),
            ),
            Err(e) => send_message(&sender, format!("7z archiving error occured!: {}", e)),
        };
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::core::test_util::setup;
    use crate::extra::get_dir_list;
    use crossbeam_queue::SegQueue;
    use std::sync::mpsc;
    use std::thread;

    #[test]
    fn process_test() {
        let (origin, dest) = setup();
        let raw_vec = get_dir_list(origin).unwrap();
        let queue = SegQueue::new();
        for i in raw_vec {
            queue.push(i);
        }
        process(Arc::new(queue), &dest)
    }

    #[test]
    fn process_with_sender_test() {
        let (origin, dest) = setup();
        let raw_vec = get_dir_list(origin).unwrap();
        let queue = SegQueue::new();
        for i in raw_vec {
            queue.push(i);
        }
        let (tx, tr) = mpsc::channel();

        thread::spawn(move || {
            process_with_sender(Arc::new(queue), &dest, tx);
        });

        for re in tr {
            println!("{}", re);
        }
    }
}
