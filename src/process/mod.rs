use std::{
    path::Path,
    sync::{mpsc::Sender, Arc},
};

use crossbeam_queue::SegQueue;

mod p_7z;
mod p_xz;
mod p_zip;

pub enum Format {
    _7z,
    xz,
    zip,
}

impl Format {
    pub fn extension(&self) -> String {
        match self {
            Format::_7z => String::from(".7z"),
            Format::xz => String::from(".tar.xz"),
            Format::zip => String::from(".zip"),
        }
    }

    pub fn from(format_str: &str) -> Self {
        match format_str {
            "7z" => Format::_7z,
            "xz" => Format::xz,
            "zip" => Format::zip,
            _ => panic!("wrong format string!"),
        }
    }
}

impl Clone for Format {
    fn clone(&self) -> Self {
        match self {
            Format::_7z => Format::_7z,
            Format::xz => Format::xz,
            Format::zip => Format::zip,
        }
    }
}

impl ToString for Format {
    fn to_string(&self) -> String {
        match self {
            Format::_7z => String::from("7z"),
            Format::xz => String::from("xz"),
            Format::zip => String::from("zip"),
        }
    }
}

pub trait Process {
    fn process<T: AsRef<Path>, O: AsRef<Path>>(
        queue: Arc<SegQueue<T>>,
        dest: Arc<O>,
        sender: Option<Sender<String>>,
    );
}

pub fn get_compressor<T: AsRef<Path>, O: AsRef<Path>>(
    comp_t: Format,
) -> fn(queue: Arc<SegQueue<T>>, dest: Arc<O>, sender: Option<Sender<String>>) {
    return match comp_t {
        Format::_7z => p_7z::Process7z::process,
        Format::xz => p_xz::ProcessXz::process,
        Format::zip => p_zip::ProcessZip::process,
    };
}
