use std::io;
use regex::bytes;
use std::fs::File;
use std::io::SeekFrom;
use crate::ReadSeek;
use crate::record::UsnEntry;


pub struct UsnParser<T: ReadSeek> {
    _inner_handle: T,
    _size: u64,
}

impl UsnParser<File> {
    pub fn from_path(filename: &str) -> Result<Self, io::Error> {
        let file_handle = File::open(filename)?;

        Self::from_read_seek(
            file_handle
        )
    }
}

impl <T: ReadSeek> UsnParser <T> {
    pub fn from_read_seek(mut inner_handle: T) -> Result<Self, io::Error> {
        // We need to get the end offset to determine the size
        let end_offset = inner_handle.seek(SeekFrom::End(0))?;

        // Seek back to the beginning
        inner_handle.seek(SeekFrom::Start(0))?;

        Ok(Self {
            _inner_handle: inner_handle,
            _size: end_offset,
        })
    }

    pub fn get_chunk_iterator(&mut self) -> IterFileChunks<T> {
        IterFileChunks{
            parser: self,
            chunk_size: 4096,
            current_offset: 0,
        }
    }
}

pub struct IterFileChunks<'c, T: ReadSeek> {
    parser: &'c mut UsnParser<T>,
    chunk_size: usize,
    current_offset: u64,
}

impl <'c, T: ReadSeek> Iterator for IterFileChunks <'c, T> {
    type Item = DataChunk;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        while self.current_offset < self.parser._size {
            let current_offset = self.current_offset;
            let mut buffer = vec![0u8; self.chunk_size];

            let _bytes_read = match self.parser._inner_handle.read(
                buffer.as_mut_slice()
            ){
                Ok(bytes_read) => bytes_read,
                Err(error) => {
                    error!("{}", error);
                    return None
                }
            };

            self.current_offset += _bytes_read as u64;

            return Some(
                DataChunk{
                    offset: current_offset,
                    data: buffer
                }
            );
        }
        
        None
    }
}

#[derive(Debug)]
pub struct DataChunk {
    offset: u64,
    data: Vec<u8>
}

impl DataChunk {
    pub fn get_records(self) -> Vec<UsnEntry> {
        debug!("Getting DataChunk at offset: {}", self.offset);
        let record_iterator = IterRecords::new(
            self.data, 
            self.offset
        );

        let records: Vec<UsnEntry> = record_iterator.collect();

        return records;
    }
}

pub struct IterRecords {
    block: Vec<u8>,
    start_offset: u64,
    match_offsets: Vec<u64>,
}

impl IterRecords {
    pub fn new(block: Vec<u8>, start_offset: u64) -> IterRecords {
        let usn_regex = bytes::Regex::new("..\x00\x00\x02\x00\x00\x00").unwrap();

        let mut match_offsets: Vec<u64> = Vec::new();
        for hit in usn_regex.find_iter(&block) {
            match_offsets.push(hit.start() as u64);
        }

        IterRecords {
            block,
            start_offset,
            match_offsets
        }
    }
}

impl Iterator for IterRecords {
    type Item = UsnEntry;

    fn next(&mut self) -> Option<UsnEntry> {
        loop {
            let start_of_hit = match self.match_offsets.pop(){
                Some(offset) => offset,
                None => break
            };

            // parse record
            let usn_record = match UsnEntry::new(
                self.start_offset + start_of_hit, 2, 
                &self.block[start_of_hit as usize ..]
            ){
                Ok(record) => record,
                Err(error) => {
                    debug!("error: {}", error);
                    continue;
                }
            };

            return Some(usn_record);
        }

        None
    }
}
