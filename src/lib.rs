#[macro_use] extern crate log;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate lazy_static;

// Our modules
pub mod usn;
pub mod record;
pub mod error;
pub mod utils;
pub mod flags;
pub mod mapping;


use std::io;
use std::io::{Read, Seek, SeekFrom};

pub trait ReadSeek: Read + Seek {
    fn tell(&mut self) -> io::Result<u64> {
        self.seek(SeekFrom::Current(0))
    }
}

impl<T: Read + Seek> ReadSeek for T {}
