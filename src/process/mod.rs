use std::{
    path::PathBuf,
    sync::{mpsc::Sender, Arc},
};

use crossbeam_queue::SegQueue;

mod p_7z;
mod p_xz;

pub enum Format {
    C7z,
    Cxz,
    // Czip,
}

impl Format {
    pub fn extension(&self) -> String {
        match self {
            Self::C7z => String::from(".7z"),
            Self::Cxz => String::from(".tar.xz"),
        } 
    }
    
    pub fn from(format_str: &str) -> Self{
        match format_str{
            "7z" => Format::C7z,
            "xz" => Format::Cxz,
            _ => panic!("wrong format string!"),
        }
    }
}

impl Clone for Format {
    fn clone(&self) -> Self {
        match self {
            Self::C7z => Format::C7z,
            Self::Cxz => Format::Cxz,
        }
    }
}

impl ToString for Format {
    fn to_string(&self) -> String {
        match self {
            Format::C7z => String::from("7z"),
            Format::Cxz => String::from("xz"),
        }
    }
}

pub fn get_compressor(
    comp_t: Format,
) -> fn(queue: Arc<SegQueue<PathBuf>>, dest: &PathBuf, sender: Option<Sender<String>>) {
    return match comp_t {
        Format::C7z => p_7z::process,
        Format::Cxz => p_xz::process,
    };
}
