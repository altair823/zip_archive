use std::path::Path;

use crate::{
    core::{c_zip::CompressZip, Compress},
    extra::try_send_message,
    Format,
};

use super::{Message, Process};

pub struct ProcessZip {
    message: Message,
}

impl Default for ProcessZip {
    fn default() -> Self {
        Self {
            message: Message::new(Format::Zip),
        }
    }
}

impl<T: AsRef<Path>, O: AsRef<Path>> Process<T, O> for ProcessZip {
    fn process(
        &self,
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
                Ok(p) => try_send_message(&sender, self.message.completion_message(p)),
                Err(e) => try_send_message(&sender, self.message.error_message(e)),
            }
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
    use std::sync::{mpsc, Arc};
    use std::thread;

    #[test]
    #[named]
    fn process_zip_test() {
        let Dir { origin, dest } = setup(function_name!());
        
        let raw_vec = get_dir_list(origin).unwrap();
        let queue = SegQueue::new();
        for i in raw_vec.clone() {
            queue.push(i);
        }
        let (tx, tr) = mpsc::channel();
        
        let arc_dest = Arc::new(dest.clone());
        thread::spawn(move || {
            let processor = ProcessZip::default();
            processor.process(Arc::new(queue), arc_dest, Some(tx));
        });
        
        let mut message = vec![];
        for re in tr {
            message.push(re);
        }

        message_test::assert_messages(dest, Format::Zip, message);
        cleanup(function_name!());
    }
}
