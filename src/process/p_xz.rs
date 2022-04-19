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
                try_send_message(&sender, format!("Cannot create tarball!"));
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

    use crate::{core::test_util::setup, get_dir_list};

    use super::*;

    #[test]
    fn process_xz_test() {
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
