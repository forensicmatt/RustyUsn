use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
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
    _offset: u64,
    _size: u64
}

// implement UsnConnection Functionality
impl UsnConnection{
    // function for getting a record
    pub fn get_next_record(&mut self)->Option<UsnRecordV2>{
        loop {
            // check we are not at the end of file
            if self._offset == self._size {
                return None;
            }

            println!("function: get_record() at offset: {}", self._offset);

            // Seek to offset
            let option = self.filehandle.seek(SeekFrom::Start(self._offset));

            // init record struct
            let mut record: UsnRecordV2 = unsafe { mem::zeroed() };

            // set the size we want to copy into the struct
            // in this case I only want the first 60 bytes of v2 record
            let r_size = 60;

            let mut record_slice;
            unsafe {
                // slice our record into a byte array to read into
                record_slice = slice::from_raw_parts_mut(
                    &mut record as *mut _ as *mut u8,
                    r_size
                );

                // read into our sliced record
                self.filehandle.read(record_slice).unwrap();
            }

            // Do some record checks first
            if record.record_length == 0{
                self._offset += 8;
                continue;
            }

            // Create a vector to store the byte buffer
            let mut buff_name = Vec::<u8>::with_capacity((record.file_name_length) as usize);
            unsafe {
                // set size of byte buffer
                buff_name.set_len(record.file_name_length as usize);
            }
            // read into byte buffer
            let bytes_read = self.filehandle.read(&mut buff_name[..]);

            // create a utf-16 buffer from the byte buffer
            let title: &[u16] = unsafe {
                // slice into 2 byte pieces
                slice::from_raw_parts(
                    buff_name.as_ptr() as *const u16,
                    buff_name.len() / 2
                )
            };
            // set record file_name
            record.file_name = String::from_utf16(title).unwrap();

            // set new offset
            self._offset += record.record_length as u64;

            // return record
            return Some(record);
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
