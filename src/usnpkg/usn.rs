use std::fs::File;
use std::io::Read;

pub struct UsnRecordV2 {
    record_length: u32,
    major_version: u16,
    minor_version: u16,
    file_reference_number: u64,
    parent_file_reference_number: u64,
    usn: u64,
    timestamp: u64,
    reason: u32,
    source_info: u32,
    security_id: u32,
    file_attributes: u32,
    file_name_length: u16,
    file_name_offset: u16,
    // 60 bytes for header //
    file_name: String
}

pub struct UsnConnection {
    filehandle: File,
    _offset: u64
}

impl UsnConnection{
    pub fn new(fh:File) -> UsnConnection {
        UsnConnection {filehandle:fh,_offset:0}
    }
    pub fn get_record(&mut self){
        println!("function: get_record()");

        // we need to read from the filehandle and return proper record

        // set _offset
    }
}

pub fn open_file(filename: &str)->UsnConnection{
    println!("function: open_file({})",filename);
    let usn_fh = match File::open(filename) {
        Ok(usn_fh) => usn_fh,
       // Handle error here
       Err(error) => panic!("Error: {}",error)
    };

    let usn_connection = UsnConnection{
        filehandle: usn_fh,
        _offset: 0
    };

    return usn_connection;
}
