use std::{
    error::Error,
    path::Path,
    sync::{mpsc::Sender, Arc},
};

use crossbeam_queue::SegQueue;

mod p_7z;
mod p_xz;
mod p_zip;

/// The enum of formats that currently supported.  
/// Using this enum, you can set the format of archiving method. 
#[derive(PartialEq)]
pub enum Format {

    /// .7z format. 
    /// Best compression level, longest time, need requirments.  
    /// If you want to use it, see requirments tab in README.  
    _7z, 


    /// .xz format.  
    /// Good compression level.  
    Xz,


    /// Deflate archive which has .zip extension.  
    /// shortest time.  
    Zip,
}

impl Format {

    /// Get extension string of [`Format`].
    pub fn extension(&self) -> String {
        match self {
            Format::_7z => String::from(".7z"),
            Format::Xz => String::from(".tar.xz"),
            Format::Zip => String::from(".zip"),
        }
    }

    /// Create a [`Format`] from the str. 
    pub fn from(format_str: &str) -> Self {
        match format_str {
            "7z" => Format::_7z,
            "xz" => Format::Xz,
            "zip" => Format::Zip,
            _ => panic!("wrong format string!"),
        }
    }
}

impl Clone for Format {
    fn clone(&self) -> Self {
        match self {
            Format::_7z => Format::_7z,
            Format::Xz => Format::Xz,
            Format::Zip => Format::Zip,
        }
    }
}

impl ToString for Format {
    fn to_string(&self) -> String {
        match self {
            Format::_7z => String::from("7z"),
            Format::Xz => String::from("xz"),
            Format::Zip => String::from("zip"),
        }
    }
}

impl Default for Format {
    fn default() -> Self {
        Format::Zip
    }
}

pub trait Process<T: AsRef<Path>, O: AsRef<Path>> {
    fn process(&self, queue: Arc<SegQueue<T>>, dest: Arc<O>, sender: Option<Sender<String>>);
}

pub struct Message {
    format: Format,
}

impl Message {
    pub fn new(format: Format) -> Self {
        Message { format: format }
    }

    pub fn completion_message<P: AsRef<Path>>(&self, target_path: P) -> String {
        format!(
            "{} archiving complete: {}",
            self.format.to_string(),
            match target_path.as_ref().to_str() {
                Some(s) => s,
                None => "",
            }
        )
    }

    pub fn error_message<E: Error>(&self, error: E) -> String {
        format!(
            "{} archiving error occured!: {}",
            self.format.to_string(),
            error
        )
    }
}

pub fn get_compressor<T: AsRef<Path>, O: AsRef<Path>>(comp_t: Format) -> Box<dyn Process<T, O>> {
    return match comp_t {
        Format::Xz => Box::new(p_xz::ProcessXz::default()),
        Format::_7z => Box::new(p_7z::Process7z::default()),
        Format::Zip => Box::new(p_zip::ProcessZip::default()),
    };
}

#[cfg(test)]
pub mod message_test{
    use std::path::Path;

    use crate::{Format, process::Message};

    pub fn assert_messages<T: AsRef<Path>>(dest: T,format: Format, mut message: Vec<String>){
        let format = format;
        let expected_message = Message::new(format.clone());
        let mut expected_messages = vec![
            expected_message.completion_message(format!("{}/dir1{}", dest.as_ref().to_str().unwrap(), format.extension())),
            expected_message.completion_message(format!("{}/dir2{}", dest.as_ref().to_str().unwrap(), format.extension())),
            expected_message.completion_message(format!("{}/dir3{}", dest.as_ref().to_str().unwrap(), format.extension())),
        ];

        message.sort();
        expected_messages.sort();

        assert_eq!(message, expected_messages);
    }
}