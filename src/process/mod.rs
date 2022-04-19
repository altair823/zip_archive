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

impl Clone for Format {
    fn clone(&self) -> Self {
        match self {
            Self::C7z => Self::C7z,
            Self::Cxz => Self::Cxz,
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
