//! # Zip archive
//!
//! `zip_archive` is a library that archive a directory with a specific compression format.
//! Supports multi-threading.
//!
//! # Supported Formats
//!
//! | Formats | description |
//! | ------ | ------ |
//! | [xz](https://en.wikipedia.org/wiki/XZ) | Using [xz2] crate. |
//! | [7z](https://www.7-zip.org) | See [Requirements](#requirements-for-7z-format) section. |
//! | [zip] | Using [zip] crate. |
//!
//!
//! # Examples
//!
//! - Compress root directory with 4 threads.
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
//! - Compress each directory using the container's iterator.
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
//! - Compress directory with .xz format.
//! ```
//! use std::path::PathBuf;
//! use zip_archive::Format;
//! use zip_archive::{Archiver, get_dir_list_with_depth};
//!
//! let origin = PathBuf::from("./origin");  // Change to the wanted directory.
//! let dest = PathBuf::from("./dest");
//!
//! let mut archiver = Archiver::new();
//! archiver.push(origin);
//! archiver.set_destination(dest);
//! archiver.set_format(Format::Xz); // == `archiver.set_format_str("xz");`
//! match archiver.archive(){
//!     Ok(_) => (),
//!     Err(e) => println!("Cannot archive the directory! {}", e),
//! };
//! ```
//!
//! # Requirements for 7z format
//!
//! To use 7z archiving format, you need to install 7z or get the executable depending on the operating system.
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

mod core;
mod extra;
mod process;

use crossbeam_queue::SegQueue;
use extra::try_send_message;
use process::get_compressor;
use std::error::Error;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::{io, thread};

pub use extra::{get_dir_list, get_dir_list_with_depth};
pub use process::Format;

/// Archiver struct.
///
/// You can use this struct and its methods to compress directories or files.
/// For the detail example, see also [`archive`](Archiver::archive) function.  
pub struct Archiver {
    dest: Option<PathBuf>,
    thread_count: u32,
    sender: Option<Sender<String>>,
    queue: Option<SegQueue<PathBuf>>,
    format: Format,
}

impl Archiver {
    /// Create a new Archiver instance.
    /// The initial number of threads is 1.
    pub fn new() -> Self {
        Archiver {
            dest: None,
            thread_count: 1,
            sender: None,
            queue: None,
            format: Format::Zip,
        }
    }

    /// Set the destination of compressed files.
    /// If the destination directory does not exist,
    /// it will create a new directory when the `archive` function is called.
    pub fn set_destination<T: AsRef<Path>>(&mut self, dest: T) {
        self.dest = Some(dest.as_ref().to_path_buf());
    }

    /// Set for the number of threads.
    pub fn set_thread_count(&mut self, thread_count: u32) {
        self.thread_count = thread_count;
    }

    /// Set the [`std::sync::mpsc::Sender`] to send messages whether compressing processes complete.
    pub fn set_sender(&mut self, sender: Sender<String>) {
        self.sender = Some(sender);
    }

    /// Set the format of the file to be compressed with [Format].
    /// For more information, see [Format].
    /// ```
    /// use zip_archive::{Archiver, Format};
    /// let mut archiver = Archiver::new();
    /// archiver.set_format(Format::_7z);
    /// ```
    pub fn set_format(&mut self, comp_format: Format) {
        self.format = comp_format;
    }

    /// Set the format of the file to be compressed with a string. 
    /// ```
    /// use zip_archive::Archiver;
    /// let mut archiver = Archiver::new();
    /// archiver.set_format_str("7z");
    /// ```
    pub fn set_format_str<T: ToString>(&mut self, comp_format_str: T) {
        self.format = Format::from(&comp_format_str.to_string());
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
        if let None = self.queue {
            self.queue = Some(SegQueue::new());
        }
        for i in iter {
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
    pub fn push<T: AsRef<Path>>(&mut self, path: T) {
        if let None = self.queue {
            self.queue = Some(SegQueue::new());
        }
        self.queue
            .as_ref()
            .unwrap()
            .push(path.as_ref().to_path_buf());
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
        self.verify_dest()?;
        self.verigy_queue()?;

        let queue = Arc::new(Archiver::copy_queue(self.queue.as_ref().unwrap()));
        let dest = Arc::new(self.dest.clone().unwrap());

        let mut handles = Vec::new();
        for _ in 0..self.thread_count {
            let arc_queue = Arc::clone(&queue);
            let arc_dest = Arc::clone(&dest);
            let format = self.format.clone();
            let handle;
            match self.sender {
                Some(ref s) => {
                    let new_sender = s.clone();
                    handle = thread::spawn(move || {
                        let compressor = get_compressor(format);
                        compressor.process(arc_queue, arc_dest, Some(new_sender));
                    });
                }
                None => {
                    handle = thread::spawn(move || {
                        let compressor = get_compressor(format);
                        compressor.process(arc_queue, arc_dest, None);
                    });
                }
            }
            handles.push(handle);
        }
        for h in handles {
            h.join().unwrap();
        }

        try_send_message(&self.sender, "Archiving Complete!".to_string());

        Ok(())
    }

    fn verify_dest(&self) -> Result<(), Box<dyn Error>> {
        match &self.dest {
            Some(p) if !p.is_dir() => {
                create_dir_all(p)?;
                Ok(())
            }
            None => Err(Box::new(io::Error::new(
                io::ErrorKind::NotFound,
                "Destination directory is not set",
            ))),
            _ => Ok(()),
        }
    }

    fn verigy_queue(&self) -> Result<(), Box<dyn Error>> {
        match &self.queue {
            Some(q) => {
                if self.queue.as_ref().unwrap().is_empty() {
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::NotFound,
                        "The queue is empty",
                    )));
                }
                try_send_message(
                    &self.sender,
                    format!("Total archive directory count: {}", q.len()),
                );
                Ok(())
            }
            None => {
                try_send_message(
                    &self.sender,
                    "There are no files to archive in queue.".to_string(),
                );
                Err(Box::new(io::Error::new(
                    io::ErrorKind::NotFound,
                    "The queue is empty",
                )))
            }
        }
    }

    fn copy_queue<T>(queue: &SegQueue<T>) -> SegQueue<T> {
        let new_queue = SegQueue::new();
        while !queue.is_empty() {
            new_queue.push(queue.pop().unwrap());
        }
        new_queue
    }
}

#[cfg(test)]
mod tests {

    use function_name::named;

    use crate::core::test_util::{cleanup, setup, Dir};

    use super::*;
    use std::sync::mpsc;

    #[test]
    #[named]
    fn archive_root_dir_test() {
        let Dir { origin, dest } = setup(function_name!());

        let mut archiver = Archiver::new();
        archiver.push_from_iter(get_dir_list(origin).unwrap().iter());
        archiver.set_destination(&dest);

        archiver.archive().unwrap();

        assert!(dest.join("dir1.zip").is_file());
        assert!(dest.join("dir2.zip").is_file());
        assert!(dest.join("dir3.zip").is_file());

        cleanup(function_name!());
    }

    #[test]
    #[named]
    fn archive_root_dir_with_sender_test() {
        let Dir { origin, dest } = setup(function_name!());

        let (tx, tr) = mpsc::channel();
        {
            let mut archiver = Archiver::new();
            archiver.push_from_iter(get_dir_list(origin).unwrap().iter());
            archiver.set_destination(&dest);
            archiver.set_sender(tx);
            archiver.archive().unwrap();
        }
        let mut messages = vec![];
        for re in tr {
            messages.push(re);
        }
        let mut expected_messages = vec![
            "Total archive directory count: 3",
            "zip archiving complete: test_dest_archive_root_dir_with_sender_test/dir2.zip",
            "zip archiving complete: test_dest_archive_root_dir_with_sender_test/dir3.zip",
            "zip archiving complete: test_dest_archive_root_dir_with_sender_test/dir1.zip",
            "Archiving Complete!",
        ];

        messages.sort();
        expected_messages.sort();

        assert_eq!(expected_messages, messages);
        assert!(dest.join("dir1.zip").is_file());
        assert!(dest.join("dir2.zip").is_file());
        assert!(dest.join("dir3.zip").is_file());

        cleanup(function_name!());
    }

    #[test]
    fn copy_queue_test() {
        let queue1 = SegQueue::new();
        queue1.push("value1");
        queue1.push("value2");
        queue1.push("value3");
        queue1.push("value4");
        let queue2 = Archiver::copy_queue(&queue1);

        let mut queue_vec = Vec::new();
        while !queue2.is_empty() {
            queue_vec.push(queue2.pop().unwrap());
        }
        let mut expected_vec = vec!["value1", "value2", "value3", "value4"];

        queue_vec.sort();
        expected_vec.sort();

        assert_eq!(expected_vec, queue_vec);
    }

    #[test]
    #[named]
    fn add_queue_test() {
        let Dir { origin, dest } = setup(function_name!());
        let queue = SegQueue::new();
        for dir in origin.read_dir().unwrap() {
            queue.push(dir.unwrap().path().to_path_buf());
        }
        let mut archiver = Archiver::new();
        archiver.push_from_iter(queue.into_iter());
        archiver.set_destination(dest.to_path_buf());
        archiver.archive().unwrap();
        assert!(dest.join("dir1.zip").is_file());
        assert!(dest.join("dir2.zip").is_file());
        assert!(dest.join("dir3.zip").is_file());
        cleanup(function_name!());

        let Dir { origin, dest } = setup(function_name!());
        let files = get_dir_list(origin).unwrap();
        let mut archiver = Archiver::new();
        archiver.push_from_iter(files.iter());
        archiver.set_destination(dest.to_path_buf());
        archiver.archive().unwrap();
        assert!(dest.join("dir1.zip").is_file());
        assert!(dest.join("dir2.zip").is_file());
        assert!(dest.join("dir3.zip").is_file());
        cleanup(function_name!());

        let Dir { origin, dest } = setup(function_name!());
        let mut archiver = Archiver::new();
        archiver.push_from_iter(
            vec![
                origin.join("dir1").to_str().unwrap(),
                origin.join("dir2").to_str().unwrap(),
                origin.join("dir3").to_str().unwrap(),
            ]
            .into_iter(),
        );
        archiver.set_destination(dest.to_path_buf());
        archiver.archive().unwrap();
        assert!(dest.join("dir1.zip").is_file());
        assert!(dest.join("dir2.zip").is_file());
        assert!(dest.join("dir3.zip").is_file());
        cleanup(function_name!());
    }

    #[test]
    #[named]
    fn push_test() {
        let Dir { origin, dest } = setup(function_name!());
        let mut archiver = Archiver::new();
        archiver.push(origin.join("dir1"));
        archiver.push(origin.join("dir2"));
        archiver.set_destination(&dest);
        archiver.archive().unwrap();

        assert!(dest.join("dir1.zip").is_file());
        assert!(dest.join("dir2.zip").is_file());
        cleanup(function_name!());
    }

    #[test]
    #[named]
    fn format_test() {
        let Dir { origin, dest } = setup(function_name!());
        let mut archiver = Archiver::new();
        archiver.push_from_iter(get_dir_list(&origin).unwrap().iter());
        archiver.set_destination(&dest);
        archiver.set_format(Format::Xz);
        archiver.archive().unwrap();

        assert!(dest.join("dir1.tar.xz").is_file());
        assert!(dest.join("dir2.tar.xz").is_file());
        assert!(dest.join("dir3.tar.xz").is_file());

        archiver.push_from_iter(get_dir_list(&origin).unwrap().iter());
        archiver.set_format_str("7z");
        archiver.archive().unwrap();

        assert!(dest.join("dir1.7z").is_file());
        assert!(dest.join("dir2.7z").is_file());
        assert!(dest.join("dir3.7z").is_file());
        cleanup(function_name!());
    }
}
