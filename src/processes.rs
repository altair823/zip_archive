
use crossbeam_queue::SegQueue;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::path::PathBuf;

use crate::core::compress_a_dir_to_7z;


pub fn process(queue: Arc<SegQueue<PathBuf>>, root: &PathBuf, dest: &PathBuf) {
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

pub fn process_with_sender(
    queue: Arc<SegQueue<PathBuf>>,
    root: &PathBuf,
    dest: &PathBuf,
    sender: Sender<String>,
) {
    while !queue.is_empty() {
        let dir = match queue.pop() {
            None => break,
            Some(d) => d,
        };
        match compress_a_dir_to_7z(dir.as_path(), &dest) {
            Ok(p) => match sender.send(format!("7z archiving complete: {}", p.to_str().unwrap())) {
                Ok(_) => {}
                Err(e) => println!("Message passing error!: {}", e),
            },
            Err(e) => match sender.send(format!("7z archiving error occured!: {}", e)) {
                Ok(_) => {}
                Err(e) => println!("Message passing error!: {}", e),
            },
        };
    }
}
