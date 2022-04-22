use crossbeam_queue::SegQueue;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::sync::Arc;

use crate::core::Compress;
use crate::extra::try_send_message;
use crate::{core::c_7z::Compress7z, Format};

use super::{Message, Process};

pub struct Process7z {
    message: Message,
}

impl Default for Process7z {
    fn default() -> Self {
        Self {
            message: Message::new(Format::_7z),
        }
    }
}

impl<T: AsRef<Path>, O: AsRef<Path>> Process<T, O> for Process7z {
    fn process(&self, queue: Arc<SegQueue<T>>, dest: Arc<O>, sender: Option<Sender<String>>) {
        let dest = &*dest;
        while !queue.is_empty() {
            let dir = match queue.pop() {
                None => break,
                Some(d) => d,
            };
            match Compress7z::compress(&dir, &dest) {
                Ok(p) => try_send_message(&sender, self.message.completion_message(p)),
                Err(e) => try_send_message(&sender, self.message.error_message(e)),
            };
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::core::test_util::{cleanup, setup, Dir};
    use crate::extra::get_dir_list;
    use crate::process::message_test;
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

        let arc_dest = Arc::new(dest.clone());
        thread::spawn(move || {
            let processor = Process7z::default();
            processor.process(Arc::new(queue), arc_dest, Some(tx));
        });

        let mut message = vec![];
        for re in tr {
            message.push(re);
        }

        message_test::assert_messages(dest, Format::_7z, message);
        cleanup(function_name!());
    }
}
