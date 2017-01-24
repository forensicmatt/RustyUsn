use usnpkg::chrono::*;                                  //Datetime Handling
use usnpkg::byteorder::{ReadBytesExt, LittleEndian};    //Reading little endian data structs
use std::fs::File;                                      //File handle
use std::io::{Error, ErrorKind};                        //for error handling
use std::io::Read;                                      //for Reading our File
use std::io::Seek;                                      //for Seeking our File
use std::io::SeekFrom;                                  //for Seeking our File
use std::slice;                                         //for going from binary to structures
use std::mem;                                           //for initializing our USN struct

#[derive(Debug)] //So we can print our structure
// Structure reference:
// https://msdn.microsoft.com/en-us/library/windows/desktop/aa365722(v=vs.85).aspx
pub struct UsnRecordV2 {
    // 0
    record_length: u32,
    major_version: u16,
    minor_version: u16,
    file_reference_number: u64,
    parent_file_reference_number: u64,
    usn: u64,
    // 32
    timestamp: NaiveDateTime, // Holds our datetime
    // 40
    reason: u32,
    source_info: u32,
    security_id: u32,
    file_attributes: u32,
    file_name_length: u16,
    file_name_offset: u16,
    // 60
    file_name: String // For unicode
}

// maintain UsnConnection info
pub struct UsnConnection {
    filehandle: File, // The filehandle
    _offset: u64,     // maintain where we are in the file
    _size: u64        // the size of the file
}

// implement UsnConnection Functionality
impl UsnConnection{
    // function for getting a record
    pub fn get_next_record(&mut self)->Result<UsnRecordV2,Error>{
        loop {
            // Check that our offset is not past the end of file
            if self._offset >= self._size{
                return Err(Error::new(ErrorKind::Other, "End of File"))
            }

            println!("function: get_next_record() at offset: {}", self._offset);

            // Seek to offset
            let soffset = match self.filehandle.seek(SeekFrom::Start(self._offset)){
                Ok(soffset) => soffset,
                Err(error) => return Err(error)
            };

            if soffset > self._size{
                return Err(Error::new(ErrorKind::Other, "End of File"))
            }

            // init record struct
            let mut record: UsnRecordV2 = unsafe {
                mem::zeroed()
            };

            ///////////////////////
            // Read structure
            ///////////////////////
            record.record_length = self.filehandle.read_u32::<LittleEndian>().unwrap();

            // Do some record checks first
            if record.record_length == 0{
                self._offset += 8;
                continue;
            }
            // TODO: Add additional checks here

            // Parse next 28 bytes
            record.major_version = self.filehandle.read_u16::<LittleEndian>().unwrap();
            record.minor_version = self.filehandle.read_u16::<LittleEndian>().unwrap();
            record.file_reference_number = self.filehandle.read_u64::<LittleEndian>().unwrap();
            record.parent_file_reference_number = self.filehandle.read_u64::<LittleEndian>().unwrap();
            record.usn = self.filehandle.read_u64::<LittleEndian>().unwrap();

            // Create datetime epoch (Windows epoch is 1601-01-01)
            record.timestamp = NaiveDate::from_ymd(1601, 1, 1).and_hms_nano(0, 0, 0, 0);
            // Get nanoseconds (100-nanosecond intervals)
            let t_nano = self.filehandle.read_i64::<LittleEndian>().unwrap();
            // Convert to microseconds
            let t_micro = t_nano / 10;
            // Add microseconds to timestamp via Duration
            record.timestamp = record.timestamp + duration::Duration::microseconds(t_micro);

            // Parse next 20 bytes
            record.reason = self.filehandle.read_u32::<LittleEndian>().unwrap();
            record.source_info = self.filehandle.read_u32::<LittleEndian>().unwrap();
            record.security_id = self.filehandle.read_u32::<LittleEndian>().unwrap();
            record.file_attributes = self.filehandle.read_u32::<LittleEndian>().unwrap();
            record.file_name_length = self.filehandle.read_u16::<LittleEndian>().unwrap();
            record.file_name_offset = self.filehandle.read_u16::<LittleEndian>().unwrap();

            // Create a vector to store the byte buffer
            let mut buff_name = Vec::<u8>::with_capacity((record.file_name_length) as usize);
            unsafe {
                // set size of byte buffer
                buff_name.set_len(record.file_name_length as usize);
            }

            // read into byte buffer
            match self.filehandle.read(&mut buff_name[..]){
                Ok(bytes_read) => bytes_read,
                Err(error) => return Err(error)
            };

            // create a utf-16 buffer from the byte buffer
            let wchar_buff: &[u16] = unsafe {
                // slice into 2 byte pieces
                slice::from_raw_parts(
                    buff_name.as_ptr() as *const u16,
                    buff_name.len() / 2
                )
            };

            // set record file_name
            record.file_name = String::from_utf16(wchar_buff).unwrap();

            // set new offset
            self._offset += record.record_length as u64;

            // return record
            return Ok(record);
        }
    }
}

// Get UsnConnection from filename
pub fn open_file(filename: &str)->UsnConnection{
    // Open a filehandle to the file
    let mut usn_fh = match File::open(filename) {
        Ok(usn_fh) => usn_fh,
        // Handle error here
        Err(error) => panic!("Error: {}",error)
    };

    // get file size
    let size = match usn_fh.seek(SeekFrom::End(0)){
        Err(e) => panic!("Error: {}",e),
        Ok(size) => size
    };

    // Create UsnConnection
    let usn_connection = UsnConnection{
        filehandle: usn_fh,
        _offset: 0,
        _size: size
    };

    // return our connection
    return usn_connection;
}

// Get Record from Buffer
pub fn parse_record(buffer: &[u8])->Result<UsnRecordV2,Error>{
    // Create record and initialize it
    let mut record: UsnRecordV2 = unsafe {
        mem::zeroed()
    };

    // record.record_length = unsafe {
    //     mem::transmute(buffer[4])
    // };

    Ok(record)
}
