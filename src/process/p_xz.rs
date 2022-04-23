use std::{
    fs,
    path::Path,
    sync::{mpsc::Sender, Arc},
};

use crossbeam_queue::SegQueue;

use crate::{
    core::{c_tar::CompressTar, c_xz::CompressXz, Compress},
    extra::try_send_message,
    Format,
};

use super::{Message, Process};

pub struct ProcessXz {
    message: Message,
}

impl Default for ProcessXz {
    fn default() -> Self {
        Self {
            message: Message::new(Format::Xz),
        }
    }
}

impl<T: AsRef<Path>, O: AsRef<Path>> Process<T, O> for ProcessXz {
    fn process(&self, queue: Arc<SegQueue<T>>, dest: Arc<O>, sender: Option<Sender<String>>) {
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
                    try_send_message(&sender, self.message.completion_message(p));
                }
                Err(e) => try_send_message(&sender, self.message.error_message(e)),
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
        get_dir_list, process::message_test,
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

        let arc_dest = Arc::new(dest.clone());
        thread::spawn(move || {
            let processor = ProcessXz::default();
            processor.process(Arc::new(queue), arc_dest, Some(tx));
        });

        let mut message = vec![];
        for re in tr {
            message.push(re);
        }
        
        message_test::assert_messages(dest, Format::Xz, message);
        cleanup(function_name!());
    }
}
