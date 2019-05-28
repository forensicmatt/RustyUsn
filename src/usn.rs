use std::io;
use regex::bytes;
use std::fs::File;
use std::io::SeekFrom;
use crate::ReadSeek;
use crate::record::UsnEntry;

// This is the size of data chunks
const SIZE_CHUNK: usize = 5120;
// This is the size of the search within a chunk
// It is smaller than the chunk size to garentee that the last record found is complete
// It has been noticed that generally usn records are paged in 4096 byte pages. I have not
// observed usn records overlaping the 4096 offset and are zero padded to the 4096 mark.
const SIZE_SEARCH: usize = 4096;


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
            chunk_size: SIZE_CHUNK,
            search_size: SIZE_SEARCH,
            chunk_start_offset: 0,
        }
    }
}

pub struct IterFileChunks<'c, T: ReadSeek> {
    parser: &'c mut UsnParser<T>,
    // The chunk size is larger than the parse size to ensure complete end record
    chunk_size: usize,
    // The parse size is smaller than the chunk size to ensure there is overlap for complete end record
    search_size: usize,
    // This is the relative offset of this chunk
    chunk_start_offset: u64,
}

impl <'c, T: ReadSeek> Iterator for IterFileChunks <'c, T> {
    type Item = DataChunk;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        while self.chunk_start_offset < self.parser._size {
            // Create buffer for our data chunk
            let mut buffer = vec![0u8; self.chunk_size];

            // Get the current offset
            let current_offset = self.chunk_start_offset;
            
            // Seek to where we start our chunk
            match self.parser._inner_handle.seek(
                SeekFrom::Start(current_offset)
            ) {
                Ok(_) => {},
                Err(error) => {
                    error!("{}", error);
                    break;
                }
            }

            // Read into buffer
            let _bytes_read = match self.parser._inner_handle.read(
                buffer.as_mut_slice()
            ){
                Ok(bytes_read) => bytes_read,
                Err(error) => {
                    error!("{}", error);
                    return None
                }
            };

            // Set the next chunk's offset
            // Increment by search size and not chunk size
            self.chunk_start_offset += self.search_size as u64;

            // Return data chunk
            return Some(
                DataChunk{
                    offset: current_offset,
                    search_size: self.search_size,
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
    search_size: usize,
    data: Vec<u8>
}

impl DataChunk {
    pub fn get_records(self) -> Vec<UsnEntry> {
        trace!("Getting records for ChunkData at offset: {}", self.offset);

        let record_iterator = IterRecords::new(
            self.data, 
            self.offset,
            self.search_size
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
    pub fn new(block: Vec<u8>, start_offset: u64, search_size: usize) -> IterRecords {
        let usn_regex = bytes::Regex::new("(?-u)..\x00\x00\x02\x00\x00\x00").unwrap();

        let mut match_offsets: Vec<u64> = Vec::new();
        for hit in usn_regex.find_iter(&block[0..search_size]) {
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

            // the entries' absolute offset
            let entry_offset = self.start_offset + start_of_hit;

            // parse record
            let usn_record = match UsnEntry::new(
                entry_offset, 2, 
                &self.block[start_of_hit as usize ..]
            ){
                Ok(record) => record,
                Err(error) => {
                    debug!("error at offset {}: {}", entry_offset, error);
                    continue;
                }
            };

            return Some(usn_record);
        }

        None
    }
}
