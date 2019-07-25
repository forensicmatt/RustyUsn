#[macro_use] extern crate log;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate encoding;
extern crate byteorder;

// Our modules
pub mod usn;
pub mod flags;
pub mod record;
pub mod usn_err;
pub mod listener;
pub mod mapping;

use std::io;
use std::io::{Read, Seek, SeekFrom};

pub trait ReadSeek: Read + Seek {
    fn tell(&mut self) -> io::Result<u64> {
        self.seek(SeekFrom::Current(0))
    }
}

impl<T: Read + Seek> ReadSeek for T {}
