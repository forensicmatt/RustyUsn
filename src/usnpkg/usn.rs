use std::fs::File;
use std::io::Read;
use std::slice;
use std::mem;

#[derive(Debug)]
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

// maintain UsnConnection info
pub struct UsnConnection {
    filehandle: File,
    _offset: u64
}

// implement UsnConnection Functionality
impl UsnConnection{
    // function for getting a record
    pub fn get_record(&mut self)->UsnRecordV2{
        println!("function: get_record()");

        // init record struct
        let mut record: UsnRecordV2 = unsafe { mem::zeroed() };

        // set the size we want to copy into the struct
        // in this case I only want the first 60 bytes of v2 record
        let r_size = 60;

        unsafe {
            // slice our record into a byte array of r_size??
            let record_slice = slice::from_raw_parts_mut(
                &mut record as *mut _ as *mut u8,
                r_size
            );
            // read into our sliced record
            self.filehandle.read_exact(record_slice).unwrap();
        }

        // return record
        return record;
    }
}

// Get UsnConnection from filename
pub fn open_file(filename: &str)->UsnConnection{
    // Open a filehandle to the file
    let usn_fh = match File::open(filename) {
        Ok(usn_fh) => usn_fh,
       // Handle error here
       Err(error) => panic!("Error: {}",error)
    };

    // Create UsnConnection
    let usn_connection = UsnConnection{
        filehandle: usn_fh,
        _offset: 0
    };

    // return our connection
    return usn_connection;
}
