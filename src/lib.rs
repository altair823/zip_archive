//! # Zip archive
//!
//! `zip_archive` is a library that select 7z executable that are depending on the operating system,
//! and run it with multithread.
//!
//! # Examples
//!
//! ```
//! use std::path::PathBuf;
//! use zip_archive::archive_root_dir;
//!
//! let origin = PathBuf::from("./origin");
//! let dest = PathBuf::from("./dest");
//! let thread_count = 4;
//!
//! match archive_root_dir(origin, dest, thread_count){
//!     Ok(_) => (),
//!     Err(e) => println!("Cannot archive the directory! {}", e),
//! };
//! ```
//!
//! # Requirements
//!
//! ## Windows 10
//!
//! 1. Install [7-Zip](https://www.7-zip.org/).
//! 2. Find 7z.exe file in installed program folder and add it to path.
//! Or place it in project root folder.
//!
//! ## macOS
//!
//! 1. Download [7-Zip console version executable](https://www.7-zip.org/download.html) for macOS.
//! 2. Place 7zz executable to home directory.

use crossbeam_queue::SegQueue;
use extra::{get_dir_list, try_send_message};
use processes::{process, process_with_sender};
use std::error::Error;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;
use std::{path::PathBuf};

mod core;
mod extra;
mod processes;

pub struct Compressor {
    dest: Option<PathBuf>,
    thread_count: u32,
    sender: Option<Sender<String>>,
    queue: Option<SegQueue<PathBuf>>,
}

impl Compressor {

    pub fn new() -> Self{
        Compressor { 
            dest: None, 
            thread_count: 1, 
            sender: None, 
            queue: None 
        }
    }

    pub fn set_origin(&mut self, origin: PathBuf){
        let dir_vec = get_dir_list(origin).unwrap();
        let queue = SegQueue::new();
        for dir in dir_vec{
            queue.push(dir);
        }
        self.queue = Some(queue);
    }

    pub fn set_destination(&mut self, dest: PathBuf){
        self.dest = Some(dest);
    }

    pub fn set_thread_count(&mut self, thread_count: u32){
        self.thread_count = thread_count;
    }

    pub fn set_sender(&mut self, sender: Sender<String>){
        self.sender = Some(sender);
    }

    pub fn add_queue(&mut self, queue: SegQueue<PathBuf>){
        match &self.queue {
            None => self.queue = Some(queue),
            Some(q) => {
                while !queue.is_empty() {
                    q.push(queue.pop().unwrap());
                }
            }
        }
    }

    /// Compress the given directory with multithread.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use zip_archive::archive_root_dir;
    ///
    /// let origin = PathBuf::from("./origin");
    /// let dest = PathBuf::from("./dest");
    /// let thread_count = 4;
    ///
    /// match archive_root_dir(origin, dest, thread_count){
    ///     Ok(_) => (),
    ///     Err(e) => println!("Cannot archive the directory! {}", e),
    /// };
    /// ```
    ///
    pub fn archive_root_dir(&self) -> Result<(), Box<dyn Error>> {


        match &self.queue {
            Some(q) => {
                try_send_message(&self.sender, 
                    &format!(
                    "Total archive directory count: {}",
                    q.len()
                ));
            },
            None => {
                try_send_message(&self.sender, "There are no files to archive.");
                return Ok(());
            },
        }
        
        let queue =  Arc::new(Compressor::copy_queue(self.queue.as_ref().unwrap()));
        let dest = Arc::new(self.dest.clone().unwrap());
        
        let mut handles = Vec::new();
        for _ in 0..self.thread_count {
            let arc_queue = Arc::clone(&queue);
            let arc_dest = Arc::clone(&dest);
            let handle;
            match self.sender{
                Some(ref s) => {
                    let new_sender = s.clone();
                    handle = thread::spawn(move || {
                        process_with_sender(arc_queue, &arc_dest, new_sender);
                    });
                },
                None => {
                    handle = thread::spawn(move || {
                        process(arc_queue, &arc_dest);
                    });
                },
            }
            handles.push(handle);
        }
        for h in handles {
            h.join().unwrap();
        }

        try_send_message(&self.sender, "Archiving Complete!");

        Ok(())
    }

    fn copy_queue<T>(queue: &SegQueue<T>) -> SegQueue<T>{
        let new_queue = SegQueue::new();
        while !queue.is_empty() {
            new_queue.push(queue.pop().unwrap());
        }
        new_queue
    }
    
}


#[cfg(test)]
mod tests {

    use super::*;
    use std::sync::mpsc;
    use crate::core::test_util::setup;

    #[test]
    fn archive_root_dir_test() {
        let (origin, dest) = setup();

        let mut compressor = Compressor::new();
        compressor.set_origin(origin);
        compressor.set_destination(dest);

        compressor.archive_root_dir().unwrap();
    }

    #[test]
    fn archive_root_dir_with_sender_test() {
        let (origin, dest) = setup();

        let (tx, tr) = mpsc::channel();
        let mut compressor = Compressor::new();
        compressor.set_origin(origin);
        compressor.set_destination(dest);
        compressor.set_sender(tx);
        compressor.archive_root_dir().unwrap();
        for re in tr {
            println!("{}", re);
            if re == String::from("Archiving Complete!"){
                break;
            }
        }
    }

    #[test]
    fn copy_queue_test(){
        let queue1 = SegQueue::new();
        queue1.push("value1");
        queue1.push("value2");
        queue1.push("value3");
        queue1.push("value4");
        let queue2 = Compressor::copy_queue(&queue1);
        while !queue2.is_empty() {
            println!("{}", queue2.pop().unwrap());
        }
    }
}
