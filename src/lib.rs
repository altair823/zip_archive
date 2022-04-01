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

use std::path::{Path, PathBuf};
use std::env::consts::OS;
use std::error::Error;
use std::io;
use std::io::ErrorKind;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use subprocess::Exec;
use crossbeam_queue::SegQueue;
use std::thread;
use image_compressor::crawler::get_dir_list;

fn get_7z_executable_path() -> Result<PathBuf, Box<dyn Error>>{
    match OS {
        "macos" => Ok(PathBuf::from("./7zz")),
        "windows" => Ok(PathBuf::from("7z.exe")),
        "linux" => Ok(PathBuf::from("./7zzs")),
        e => {
            println!("Doesn't support {} currently!", e);
            return Err(Box::new(io::Error::new(ErrorKind::NotFound, "Cannot find the 7z executable!")));
        }
    }
}

fn compress_a_dir_to_7z(origin: &Path, dest: &Path, root: &Path) ->Result<PathBuf, Box<dyn Error>>{

    let compressor_path = get_7z_executable_path()?;

    let mut zip_path = dest.join(&match origin.strip_prefix(root){
        Ok(p) => p,
        Err(_) => origin,
    });
    zip_path.set_extension("7z");

    if zip_path.is_file(){
        return Err(Box::new(io::Error::new(ErrorKind::AlreadyExists, "The 7z archive file already exists!")));
    }

    let exec = Exec::cmd(compressor_path)
        .args(&vec!["a", "-mx=9", "-t7z", zip_path.to_str().unwrap(), match origin.to_str(){
            None => return Err(Box::new(io::Error::new(ErrorKind::NotFound, "Cannot get the destination directory path!"))),
            Some(s) => s,
        }]);
    exec.join()?;
    return Ok(zip_path);
}

fn process(queue: Arc<SegQueue<PathBuf>>,
           root: &PathBuf,
           dest: &PathBuf){
    while !queue.is_empty() {
        let dir = match queue.pop() {
            None => break,
            Some(d) => d,
        };
        match compress_a_dir_to_7z(dir.as_path(), &dest, &root){
            Ok(_) => {}
            Err(e) => println!("Error occurred! : {}", e),
        }
    }
}

fn process_with_sender(queue: Arc<SegQueue<PathBuf>>,
                       root: &PathBuf,
                       dest: &PathBuf,
                       sender: Sender<String>){
    while !queue.is_empty() {
        let dir = match queue.pop() {
            None => break,
            Some(d) => d,
        };
        match compress_a_dir_to_7z(dir.as_path(), &dest, &root){
            Ok(p) => {
                match sender.send(format!("7z archiving complete: {}", p.to_str().unwrap())){
                    Ok(_) => {},
                    Err(e) => println!("Message passing error!: {}", e),
                }
            }
            Err(e) => {
                match sender.send(format!("7z archiving error occured!: {}", e)) {
                    Ok(_) => {},
                    Err(e) => println!("Message passing error!: {}", e),
                }
            },
        };
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
