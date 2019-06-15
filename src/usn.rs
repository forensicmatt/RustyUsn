#[cfg(feature = "multithreading")]
use rayon;
use std::io;
use regex::bytes;
use std::cmp::max;
use std::fs::File;
use std::io::SeekFrom;
#[cfg(feature = "multithreading")]
use rayon::prelude::*;
use std::collections::VecDeque;
use byteorder::{ByteOrder, LittleEndian};
use crate::ReadSeek;
use crate::record::UsnEntry;

// This is the size of data chunks
const SIZE_CHUNK: usize = 17408;
// This is the size of the search within a chunk
// It is smaller than the chunk size to garentee that the last record found is complete
// It has been noticed that generally usn records are paged in 4096 byte pages. I have not
// observed usn records overlaping the 4096 offset and are zero padded to the 4096 mark.
const SIZE_SEARCH: usize = 16384;

lazy_static! {
    static ref RE_USN: bytes::Regex = bytes::Regex::new(
        "(?-u)..\x00\x00\x02\x00\x00\x00"
    ).expect("Regex Error");
}


pub struct UsnParserSettings{
    thread_count: usize
}

impl Default for UsnParserSettings {
    fn default() -> Self {
        UsnParserSettings {
            thread_count: 0
        }
    }
}

impl UsnParserSettings {
    pub fn new() -> UsnParserSettings {
        UsnParserSettings::default()
    }

    /// Sets the number of worker threads.
    /// `0` will let rayon decide.
    ///
    #[cfg(feature = "multithreading")]
    pub fn thread_count(mut self, thread_count: usize) -> Self {
        self.thread_count = if thread_count == 0 {
            rayon::current_num_threads()
        } else {
            thread_count
        };
        self
    }

    /// Does nothing and emits a warning when complied without multithreading.
    #[cfg(not(feature = "multithreading"))]
    pub fn thread_count(mut self, _thread_count: usize) -> Self {
        warn!("Setting num_threads has no effect when compiling without multithreading support.");
        self.thread_count = 1;
        self
    }
}


pub struct UsnParser<T: ReadSeek> {
    inner_handle: T,
    source: String,
    handle_size: u64,
    settings: UsnParserSettings
}

impl UsnParser<File> {
    pub fn from_path(filename: &str) -> Result<Self, io::Error> {
        let file_handle = File::open(filename)?;

        Self::from_read_seek(
            filename.to_string(),
            file_handle
        )
    }
}

impl <T: ReadSeek> UsnParser <T> {
    pub fn from_read_seek(source: String, mut inner_handle: T) -> Result<Self, io::Error> {
        // We need to get the end offset to determine the size
        let end_offset = inner_handle.seek(SeekFrom::End(0))?;

        // Seek back to the beginning
        inner_handle.seek(SeekFrom::Start(0))?;

        Ok( Self {
            inner_handle: inner_handle,
            source: source,
            handle_size: end_offset,
            settings: UsnParserSettings::default()
        })
    }

    pub fn with_configuration(mut self, configuration: UsnParserSettings) -> Self {
        self.settings = configuration;
        self
    }

    pub fn get_chunk_iterator(&mut self) -> IterFileChunks<T> {
        IterFileChunks{
            parser: self,
            chunk_size: SIZE_CHUNK,
            search_size: SIZE_SEARCH,
            chunk_start_offset: 0,
        }
    }

    pub fn into_chunk_iterator(self) -> IntoIterFileChunks<T> {
        IntoIterFileChunks {
            parser: self,
            chunk_size: SIZE_CHUNK,
            search_size: SIZE_SEARCH,
            chunk_start_offset: 0,
        }
    }

    pub fn records(&mut self) -> impl Iterator<Item = UsnEntry> + '_ {
        let num_threads = max(self.settings.thread_count, 1);

        let mut chunks = self.get_chunk_iterator();

        let records_per_chunk = std::iter::from_fn(move || 
            {
                // Allocate some chunks in advance, so they can be parsed in parallel.
                let mut list_of_chunks = Vec::with_capacity(num_threads);

                for _ in 0..num_threads {
                    if let Some(chunk) = chunks.next() {
                        list_of_chunks.push(chunk);
                    };
                }

                // We only stop once no chunks can be allocated.
                if list_of_chunks.is_empty() {
                    None
                } else {
                    #[cfg(feature = "multithreading")]
                    let chunk_iter = list_of_chunks.into_par_iter();
                    #[cfg(not(feature = "multithreading"))]
                    let chunk_iter = list_of_chunks.into_iter();

                    // Serialize the records in each chunk.
                    let iterators: Vec<Vec<UsnEntry>> = chunk_iter
                        .map(|data_chunk| data_chunk.get_records()
                        )
                        .collect();

                    Some(iterators.into_iter().flatten())
                }
            }
        );

        records_per_chunk.flatten()
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
        while self.chunk_start_offset < self.parser.handle_size {
            // Create buffer for our data chunk
            let mut buffer = vec![0u8; self.chunk_size];

            // Get the current offset
            let current_offset = self.chunk_start_offset;
            
            // Seek to where we start our chunk
            match self.parser.inner_handle.seek(
                SeekFrom::Start(current_offset)
            ) {
                Ok(_) => {},
                Err(error) => {
                    error!("{}", error);
                    break;
                }
            }

            // Read into buffer
            let _bytes_read = match self.parser.inner_handle.read(
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
                    source: self.parser.source.to_owned(),
                    offset: current_offset,
                    search_size: self.search_size,
                    data: buffer
                }
            );
        }
        
        None
    }
}


pub struct IntoIterFileChunks<T: ReadSeek> {
    parser: UsnParser<T>,
    chunk_size: usize,
    search_size: usize,
    chunk_start_offset: u64,
}

impl<T: ReadSeek> Iterator for IntoIterFileChunks<T> {
    type Item = DataChunk;
    
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        while self.chunk_start_offset < self.parser.handle_size {
            // Create buffer for our data chunk
            let mut buffer = vec![0u8; self.chunk_size];

            // Get the current offset
            let current_offset = self.chunk_start_offset;
            
            // Seek to where we start our chunk
            match self.parser.inner_handle.seek(
                SeekFrom::Start(current_offset)
            ) {
                Ok(_) => {},
                Err(error) => {
                    error!("{}", error);
                    break;
                }
            }

            // Read into buffer
            let _bytes_read = match self.parser.inner_handle.read(
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
                    source: self.parser.source.to_owned(),
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
    source: String,
    offset: u64,
    search_size: usize,
    data: Vec<u8>
}

impl DataChunk {
    pub fn get_records(self) -> Vec<UsnEntry> {
        trace!("Getting records for ChunkData at offset: {}", self.offset);

        let record_iterator = self.get_record_iterator();

        let records: Vec<UsnEntry> = record_iterator.collect();

        return records;
    }

    pub fn get_record_iterator(self) -> IterRecords {
        IterRecords::new(
            self.source,
            self.data, 
            self.offset,
            self.search_size
        )
    }
}

#[derive(Debug)]
pub struct IterRecords {
    source: String,
    block: Vec<u8>,
    start_offset: u64,
    match_offsets: VecDeque<u64>,
}

impl IterRecords {
    pub fn new(source: String, block: Vec<u8>, start_offset: u64, search_size: usize) -> IterRecords {
        let match_offsets: VecDeque<u64> = RE_USN.find_iter(&block[0..search_size])
            .map(|m| m.start() as u64)
            .collect();

        IterRecords {
            source,
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
            // start of hit
            let start_of_hit = match self.match_offsets.pop_front(){
                Some(offset) => offset,
                None => break
            };
            // index starts at start of hit offset
            let i = start_of_hit as usize;

            // the entries' absolute offset
            let entry_offset = self.start_offset + start_of_hit;

            // validate record length is 8 byte aligned
            let record_length = LittleEndian::read_u32(&self.block[i..i+4]);
            if record_length % 8 != 0 {
                debug!("not 8 byte aligned at offset {}", entry_offset);
                continue;
            }

            // Check versions
            let major = LittleEndian::read_u16(&self.block[i+4..i+6]);

            let usn_entry = match major {
                2 => {
                    let minor = LittleEndian::read_u16(&self.block[i+6..i+8]);

                    // validate minor version
                    if minor != 0 {
                        debug!("minor version does not match major at offset {}", entry_offset);
                        continue;
                    }

                    // validate name offset
                    let name_offset = LittleEndian::read_u16(&self.block[i+58..i+60]);
                    if name_offset != 60 {
                        debug!("name offset does not match 60 at offset {}", entry_offset);
                        continue;
                    }

                    // Parse entry
                    let entry = match UsnEntry::new(
                        self.source.clone(),
                        entry_offset, 
                        2,
                        &self.block[start_of_hit as usize ..]
                    ) {
                        Ok(entry) => entry,
                        Err(error) => {
                            debug!("error at offset {}: {}", entry_offset, error);
                            continue;
                        }
                    };

                    entry
                },
                other => {
                    debug!("Version not handled: {}; offset: {}", other, entry_offset);
                    continue;
                }
            };

            return Some(usn_entry);
        }

        None
    }
}