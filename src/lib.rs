//! # Zip archive
//!
//! `zip_archive` is a library that select 7z executable that are depending on the operating system,
//! and run it with multithread.
//!
//! # Examples
//!
//! Compress root directory with multithread.
//! ```
//! use std::path::PathBuf;
//! use zip_archive::Archiver;
//!
//! let origin = PathBuf::from("./origin");
//! let dest = PathBuf::from("./dest");
//! let thread_count = 4;
//!
//! let mut archiver = Archiver::new();
//! archiver.push(origin);
//! archiver.set_destination(dest);
//! archiver.set_thread_count(thread_count);
//! 
//! match archiver.archive(){
//!     Ok(_) => (),
//!     Err(e) => println!("Cannot archive the directory! {}", e),
//! };
//! ```
//! 
//! Compress each directory using the container's iterator.
//! ```
//! use std::path::PathBuf;
//! use zip_archive::Archiver;
//!
//! let origin = PathBuf::from("./origin");
//! let dest = PathBuf::from("./dest");
//!
//! let mut archiver = Archiver::new();
//! archiver.push_from_iter(vec!["./origin/dir1", "./origin/dir2", "./origin/dir3"].into_iter());
//! archiver.set_destination(dest);
//! match archiver.archive(){
//!     Ok(_) => (),
//!     Err(e) => println!("Cannot archive the directory! {}", e),
//! };
//! ```
//! 
//! Compress subdirectories with a depth of 1.
//! ```
//! use std::path::PathBuf;
//! use zip_archive::{Archiver, get_dir_list};
//!
//! let origin = PathBuf::from("./origin");  // Change to the wanted directory.
//! let dest = PathBuf::from("./dest");
//! 
//! let mut archiver = Archiver::new();
//! archiver.push_from_iter(get_dir_list(origin).unwrap().iter());
//! archiver.set_destination(dest);
//! match archiver.archive(){
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
use extra::try_send_message;
use processes::{process, process_with_sender};
use std::error::Error;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::{thread, io};
use std::path::{Path, PathBuf};
use std::fs::create_dir_all;

mod core;
mod extra;
mod processes;

pub use extra::{get_dir_list, get_dir_list_with_depth};

/// Archiver struct.
/// 
/// You can use this struct and its methods to compress directories or files.
/// For the detail example, see also [`archive`](Archiver::archive) function.  
pub struct Archiver {
    dest: Option<PathBuf>,
    thread_count: u32,
    sender: Option<Sender<String>>,
    queue: Option<SegQueue<PathBuf>>,
}

impl Archiver {

    /// Create a new Archiver instance. 
    /// The initial number of threads is 1. 
    pub fn new() -> Self{
        Archiver { 
            dest: None, 
            thread_count: 1, 
            sender: None, 
            queue: None 
        }
    }

    /// Set the destination of compressed files. 
    /// If the destination directory does not exist, 
    /// it will create a new directory when the `archive` function is called.
    pub fn set_destination<T: AsRef<Path>>(&mut self, dest: T){
        self.dest = Some(dest.as_ref().to_path_buf());
    }

    /// Set for the number of threads. 
    pub fn set_thread_count(&mut self, thread_count: u32){
        self.thread_count = thread_count;
    }

    /// Set the [`std::sync::mpsc::Sender`] to send messages whether compressing processes complete.
    pub fn set_sender(&mut self, sender: Sender<String>){
        self.sender = Some(sender);
    }

    /// Push all elements in givin iterator to the queue.
    /// It iterate through all elements and push it to the queue. 
    /// 
    /// # Examples
    /// ```
    /// use zip_archive::Archiver;
    /// 
    /// let mut archiver = Archiver::new();
    /// archiver.push_from_iter(vec!["origin/dir1", "origin/dir2", "origin/dir3"].into_iter());
    /// ```
    pub fn push_from_iter<I>(&mut self, iter: I)
    where 
        I: Iterator,
        I::Item: AsRef<Path>,
        {
        if let None = self.queue{
            self.queue = Some(SegQueue::new());
        }
        for i in iter{
            self.queue.as_ref().unwrap().push(i.as_ref().to_path_buf());
        }
    }

    /// Push a single directory to the queue. 
    ///
    /// # Examples 
    /// ```
    /// use zip_archive::Archiver;
    /// 
    /// let mut archiver = Archiver::new();
    /// archiver.push("origin/dir1");
    /// archiver.push("origin/dir2");
    /// archiver.push("origin/dir3");
    /// ```
    pub fn push<T: AsRef<Path>>(&mut self, path: T){
        if let None = self.queue {
            self.queue = Some(SegQueue::new());
        }
        self.queue.as_ref().unwrap().push(path.as_ref().to_path_buf());
    }

    
    /// Compress directories in the queue with multithread.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use zip_archive::Archiver;
    ///
    /// let origin = PathBuf::from("./origin");
    /// let dest = PathBuf::from("./dest");
    /// let thread_count = 4;
    ///
    /// let mut archiver = Archiver::new();
    /// archiver.push(origin);
    /// archiver.set_destination(dest);
    /// archiver.set_thread_count(thread_count);
    /// 
    /// match archiver.archive(){
    ///     Ok(_) => (),
    ///     Err(e) => println!("Cannot archive the directory! {}", e),
    /// };
    /// ```
    ///
    pub fn archive(&self) -> Result<(), Box<dyn Error>> {

        match &self.dest{
            Some(p) if !p.is_dir() => create_dir_all(p)?,
            None => return Err(Box::new(io::Error::new(io::ErrorKind::NotFound, "Destination directory is not set"))),
            _ => (),
        };

        match &self.queue {
            Some(q) => {
                try_send_message(&self.sender, 
                    format!(
                    "Total archive directory count: {}",
                    q.len()
                ));
            },
            None => {
                try_send_message(&self.sender, "There are no files to archive in queue.".to_string());
                return Ok(());
            },
        }
        
        let queue =  Arc::new(Archiver::copy_queue(self.queue.as_ref().unwrap()));
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

        try_send_message(&self.sender, "Archiving Complete!".to_string());

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
mod tests{ 

    use super::*;
    use std::sync::mpsc;
    use crate::core::test_util::setup;

    #[test]
    fn archive_root_dir_test() {
        let (origin, dest) = setup();

        let mut archiver = Archiver::new();
        archiver.push_from_iter(get_dir_list(origin).unwrap().iter());
        archiver.set_destination(dest);

        archiver.archive().unwrap();
    }

    #[test]
    fn archive_root_dir_with_sender_test() {
        let (origin, dest) = setup();

        let (tx, tr) = mpsc::channel();
        {
            let mut archiver = Archiver::new();
            archiver.push_from_iter(get_dir_list(origin).unwrap().iter());
            archiver.set_destination(dest);
            archiver.set_sender(tx);
            archiver.archive().unwrap();
        }
        for re in tr {
            println!("{}", re);
        }
    }

    #[test]
    fn copy_queue_test(){
        let queue1 = SegQueue::new();
        queue1.push("value1");
        queue1.push("value2");
        queue1.push("value3");
        queue1.push("value4");
        let queue2 = Archiver::copy_queue(&queue1);
        while !queue2.is_empty() {
            println!("{}", queue2.pop().unwrap());
        }
    }

    #[test]
    fn add_queue_test(){
        let (origin, dest) = setup();
        let queue = SegQueue::new();
        for dir in origin.read_dir().unwrap(){
            queue.push(dir.unwrap().path().to_path_buf());
        }
        let mut archiver = Archiver::new();
        archiver.push_from_iter(queue.into_iter());
        archiver.set_destination(dest.to_path_buf());
        archiver.archive().unwrap();
        assert!(dest.join("dir1.7z").is_file());
        assert!(dest.join("dir2.7z").is_file());
        assert!(dest.join("dir3.7z").is_file());

        let (origin, dest) = setup();
        let files = get_dir_list(origin).unwrap();
        let mut archiver = Archiver::new();
        archiver.push_from_iter(files.iter());
        archiver.set_destination(dest.to_path_buf());
        archiver.archive().unwrap();
        assert!(dest.join("dir1.7z").is_file());
        assert!(dest.join("dir2.7z").is_file());
        assert!(dest.join("dir3.7z").is_file());

        let (origin, dest) = setup();
        let mut archiver = Archiver::new();
        archiver.push_from_iter(vec![
            origin.join("dir1").to_str().unwrap(), 
            origin.join("dir2").to_str().unwrap(), 
            origin.join("dir3").to_str().unwrap()
            ].into_iter());
        archiver.set_destination(dest.to_path_buf());
        archiver.archive().unwrap();
        assert!(dest.join("dir1.7z").is_file());
        assert!(dest.join("dir2.7z").is_file());
        assert!(dest.join("dir3.7z").is_file());
    }

    #[test]
    fn push_test(){
        let (origin, dest) = setup(); 
        let mut archiver = Archiver::new();
        archiver.push(origin.join("dir1"));
        archiver.push(origin.join("dir2"));
        archiver.set_destination(dest);
        archiver.archive().unwrap();
    }
}
