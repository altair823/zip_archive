use crossbeam_queue::SegQueue;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::Arc;

use crate::core::c_7z::compress_7z;
use crate::extra::try_send_message;

pub fn process(queue: Arc<SegQueue<PathBuf>>, dest: &PathBuf, sender: Option<Sender<String>>) {
    while !queue.is_empty() {
        let dir = match queue.pop() {
            None => break,
            Some(d) => d,
        };
        match compress_7z(dir.as_path(), &dest) {
            Ok(p) => try_send_message(
                &sender,
                format!("7z archiving complete: {}", p.to_str().unwrap()),
            ),
            Err(e) => try_send_message(&sender, format!("7z archiving error occured!: {}", e)),
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
    fn process_7z_test() {
        let (origin, dest) = setup();
        let raw_vec = get_dir_list(origin).unwrap();
        let queue = SegQueue::new();
        for i in raw_vec {
            queue.push(i);
        }
        let (tx, tr) = mpsc::channel();

        thread::spawn(move || {
            process(Arc::new(queue), &dest, Some(tx));
        });

        for re in tr {
            println!("{}", re);
        }
    }
}
