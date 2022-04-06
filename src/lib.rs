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

use std::path::PathBuf;
use std::error::Error;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use crossbeam_queue::SegQueue;
use extra::get_dir_list;
use processes::{process, process_with_sender};
use std::thread;

mod core;
mod extra;
mod processes;

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
pub fn archive_root_dir(root: PathBuf,
                        dest: PathBuf,
                        thread_count: u32) -> Result<(), Box<dyn Error>>{
    let to_7z_file_list = get_dir_list(&root)?;

    let queue = Arc::new(SegQueue::new());
    for dir in to_7z_file_list{
        queue.push(dir);
    }

    let mut handles = Vec::new();
    let arc_root = Arc::new(root);
    let arc_dest = Arc::new(dest);
    for _ in 0..thread_count {
        let arc_queue = Arc::clone(&queue);
        let arc_root = Arc::clone(&arc_root);
        let arc_dest = Arc::clone(&arc_dest);
        let handle = thread::spawn(move || {
            process(arc_queue, &arc_root, &arc_dest)
        });
        handles.push(handle);
    }
    for h in handles{
        h.join().unwrap();
    }

    Ok(())
}

/// Compress the given directory with multithread and Sender. 
/// 
/// # Examples
/// 
/// ```
/// use std::path::PathBuf;
/// use std::sync::mpsc;
/// use zip_archive::archive_root_dir_with_sender;
/// 
/// let origin = PathBuf::from("./origin");
/// let dest = PathBuf::from("./dest");
/// let thread_count = 4;
/// let (tx, tr) = mpsc::channel();
/// 
/// match archive_root_dir_with_sender(origin, dest, thread_count, tx.clone()){
///     Ok(_) => (),
///     Err(e) => println!("Cannot archive the directory! {}", e),
/// };
/// ```
/// 
pub fn archive_root_dir_with_sender(root: PathBuf,
                                    dest: PathBuf,
                                    thread_count: u32,
                                    sender: Sender<String>) -> Result<(), Box<dyn Error>>{
    let to_7z_file_list = match get_dir_list(&root){
        Ok(s) => s,
        Err(e) => {
            println!("Cannot extract the list of directories in {} : {}", root.to_str().unwrap(), e);
            return Err(Box::new(e));
        }
    };

    match sender.send(format!("Total archive directory count: {}", to_7z_file_list.len())){
        Ok(_) => {},
        Err(e) => println!("Message passing error!: {}", e),
    }

    let queue = Arc::new(SegQueue::new());
    for dir in to_7z_file_list{
        queue.push(dir);
    }

    let mut handles = Vec::new();
    let arc_root = Arc::new(root);
    let arc_dest = Arc::new(dest);
    for _ in 0..thread_count {
        let arc_queue = Arc::clone(&queue);
        let arc_root = Arc::clone(&arc_root);
        let arc_dest = Arc::clone(&arc_dest);
        let new_sender = sender.clone();
        let handle = thread::spawn(move || {
            process_with_sender(arc_queue, &arc_root, &arc_dest, new_sender);
        });
        handles.push(handle);
    }

    for h in handles{
        h.join().unwrap();
    }

    match sender.send(String::from("Archiving Complete!")){
        Ok(_) => {},
        Err(e) => println!("Message passing error!: {}", e),
    }
    Ok(())
}
