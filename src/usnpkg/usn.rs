use byteorder::{ReadBytesExt, LittleEndian};    //Reading little endian data structs
use seek_bufread::BufReader;
use std::fs::File;                                      //File handle
use std::io::{Error, ErrorKind};                        //for error handling
use std::io::Read;                                      //for Reading our File
use std::io::Seek;                                      //for Seeking our File
use std::io::SeekFrom;                                  //for Seeking our File
use std::slice;                                         //for going from binary to structures
use std::mem;                                           //for initializing our USN struct
use usnpkg::usn_errors;
use rwinstructs::timestamp::{WinTimestamp};
use rwinstructs::reference::{MftReference};
use usnpkg::flags;

// Structure reference:
// https://msdn.microsoft.com/en-us/library/windows/desktop/aa365722(v=vs.85).aspx
#[derive(Serialize, Debug)]
pub struct UsnRecordV2 {
    // 0
    pub record_length: u32,
    pub major_version: u16,
    pub minor_version: u16,
    pub file_reference_number: MftReference,
    pub parent_file_reference_number: MftReference,
    pub usn: u64,
    pub timestamp: WinTimestamp,
    pub reason: flags::Reason,
    pub source_info: flags::SourceInfo,
    pub security_id: u32,
    pub file_attributes: u32,
    pub file_name_length: u16,
    pub file_name_offset: u16,
    // 60
    pub file_name: String // For unicode
}

// maintain UsnConnection info
pub struct UsnConnection {
    filehandle: BufReader<File>, // File, // The filehandle
    _verbose: bool,   // verbose output
    _offset: u64,     // maintain where we are in the file
    _size: u64        // the size of the file
}

pub struct UsnResult (
    pub UsnRecordV2, // The record
    pub u64          // The offset
);

// implement UsnConnection Functionality
impl UsnConnection {
    // function for getting a record
    pub fn get_next_record(&mut self)->Result<UsnResult,Error>{
        loop {
            // Check that our offset is not past the end of file
            if self._offset >= self._size {
                return Err(Error::new(ErrorKind::Other, "End of File"));
            }

            // Seek to offset
            let soffset = match self.filehandle.seek(SeekFrom::Start(self._offset)){
                Ok(soffset) => soffset,
                Err(error) => return Err(error)
            };

            if soffset > self._size {
                return Err(Error::new(ErrorKind::Other, "End of File"));
            }

            let record: UsnRecordV2 = match read_record(self.filehandle.by_ref()) {
                Ok(record) => record,
                Err(error) => {
                    if self._verbose {
                        println!("{:?}",error);
                    }
                    self._offset = self._offset + 8;
                    continue;
                }
            };

            self._offset = self._offset + (record.record_length as u64);

            return Ok(UsnResult(record,soffset as u64));
        }
    }
}

// Get UsnConnection from filename
pub fn open_file(filename: &str, verbose: bool)->UsnConnection{
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
        filehandle: BufReader::with_capacity(4096,usn_fh),
        _verbose: verbose,
        _offset: 0,
        _size: size
    };

    // return our connection
    return usn_connection;
}

// Get Record from Buffer
pub fn read_record<R: Read>(mut buffer: R)->Result<UsnRecordV2,usn_errors::UsnError>{
    // init record struct
    let mut record: UsnRecordV2 = unsafe {
        mem::zeroed()
    };

    record.record_length = buffer.read_u32::<LittleEndian>()?;

    // Do some record checks first
    if record.record_length == 0 {
        return Err(
            usn_errors::UsnError::invalid_v2_record(
                format!("Record length is 0.")
            )
        );
    }
    if record.record_length > 1024 {
        return Err(
            usn_errors::UsnError::invalid_v2_record(
                format!("Record length is over 1024.")
            )
        );
    }

    record.major_version = buffer.read_u16::<LittleEndian>()?;
    if record.major_version != 2 {
        return Err(
            usn_errors::UsnError::invalid_v2_record(
                format!("Major version is not 2")
            )
        );
    }
    record.minor_version = buffer.read_u16::<LittleEndian>()?;
    if record.minor_version != 0 {
        return Err(
            usn_errors::UsnError::invalid_v2_record(
                format!("Minor version is not 0")
            )
        );
    }
    record.file_reference_number = MftReference(buffer.read_u64::<LittleEndian>()?);
    record.parent_file_reference_number = MftReference(buffer.read_u64::<LittleEndian>()?);
    record.usn = buffer.read_u64::<LittleEndian>()?;
    record.timestamp = WinTimestamp(buffer.read_u64::<LittleEndian>()?);

    record.reason = flags::Reason::from_bits_truncate(buffer.read_u32::<LittleEndian>()?);
    record.source_info = flags::SourceInfo::from_bits_truncate(buffer.read_u32::<LittleEndian>()?);

    record.security_id = buffer.read_u32::<LittleEndian>()?;
    record.file_attributes = buffer.read_u32::<LittleEndian>()?;
    record.file_name_length = buffer.read_u16::<LittleEndian>()?;
    record.file_name_offset = buffer.read_u16::<LittleEndian>()?;

    if record.file_name_offset != 60 {
        return Err(
            usn_errors::UsnError::invalid_v2_record(
                format!("file_name_offset is not 60")
            )
        );
    }

    // Create a vector to store the byte buffer
    let mut buff_name = vec![0u8; record.file_name_length as usize];

    // read into byte buffer
    let bytes_read = match buffer.read(&mut buff_name[..]){
        Ok(bytes_read) => bytes_read,
        Err(error) => return Err(
            usn_errors::UsnError::io_error(
                format!("IO Error: {:?}",error)
            )
        )
    };

    if bytes_read < record.file_name_length as usize {
        return Err(
            usn_errors::UsnError::io_error(
                format!("Not enough bytes for filename buffer.")
            )
        )
    }

    // create a utf-16 buffer from the byte buffer
    let wchar_buff: &[u16] = unsafe {
        // slice into 2 byte pieces
        slice::from_raw_parts(
            buff_name.as_ptr() as *const u16,
            buff_name.len() / 2
        )
    };

    // set record file_name
    record.file_name = match String::from_utf16(wchar_buff) {
        Ok(file_name) => file_name,
        Err(error) => return Err(
            usn_errors::UsnError::io_error(
                // Better Error needed here - Not really IO Error
                format!("IO Error: {:?}",error)
            )
        )
    };

    Ok(record)
}
