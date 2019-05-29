use std::io::Read;
use serde::Serialize;
use encoding::all::UTF_16LE;
use encoding::{DecoderTrap, Encoding};
use byteorder::{ReadBytesExt, LittleEndian};
use winstructs::ntfs::mft_reference::MftReference;
use crate::flags;
use crate::usn_err::UsnError;
use crate::timestamp::WinTimestamp;

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum UsnRecord {
    V2(UsnRecordV2)
}

impl UsnRecord {
    pub fn new<R: Read>(version: u16, mut reader: R)-> Result<UsnRecord, UsnError>{
        if version == 2{
            let usn_record_v2 = UsnRecordV2::new(&mut reader)?;
            Ok(UsnRecord::V2(usn_record_v2))
        } else {
            Err(UsnError::unsupported_version(
                format!("Unsupported USN version {}", version)
            ))
        }
    }
}


#[derive(Serialize, Debug)]
pub struct UsnEntry {
    #[serde(rename="_offset")]
    offset: u64,
    #[serde(flatten)]
    record: UsnRecord,
}
impl UsnEntry {
    pub fn new<R: Read>(offset: u64, version: u16, mut reader: R)-> Result<UsnEntry, UsnError>{
        let record = UsnRecord::new(version, &mut reader)?;

        Ok(UsnEntry {
            offset: offset,
            record: record,
        })
    }
}


// Structure reference:
// https://msdn.microsoft.com/en-us/library/windows/desktop/aa365722(v=vs.85).aspx
#[derive(Serialize, Debug)]
pub struct UsnRecordV2 {
    pub record_length: u32,
    pub major_version: u16,
    pub minor_version: u16,
    pub file_reference: MftReference,
    pub parent_reference: MftReference,
    pub usn: u64,
    pub timestamp: WinTimestamp,
    pub reason: flags::Reason,
    pub source_info: flags::SourceInfo,
    pub security_id: u32,
    pub file_attributes: u32,
    pub file_name_length: u16,
    pub file_name_offset: u16,
    pub file_name: String
}
impl UsnRecordV2 {
    /// A UsnRecordV2.
    ///
    /// # Example
    ///
    /// Parse a raw buffer.
    ///
    /// ```
    /// extern crate RustyUsn;
    /// use RustyUsn::record;
    /// use std::io::Cursor;
    /// # fn test_usn_record_v2() {
    ///     let record_buffer: &[u8] = &[
    ///         0x60,0x00,0x00,0x00,0x02,0x00,0x00,0x00,0x73,0x00,0x00,0x00,0x00,0x00,0x68,0x91,
    ///         0x3B,0x2A,0x02,0x00,0x00,0x00,0x07,0x00,0x00,0x00,0x80,0xBC,0x04,0x00,0x00,0x00,
    ///         0x53,0xC7,0x8B,0x18,0xC5,0xCC,0xCE,0x01,0x02,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
    ///         0x00,0x00,0x00,0x00,0x20,0x20,0x00,0x00,0x20,0x00,0x3C,0x00,0x42,0x00,0x54,0x00,
    ///         0x44,0x00,0x65,0x00,0x76,0x00,0x4D,0x00,0x61,0x00,0x6E,0x00,0x61,0x00,0x67,0x00,
    ///         0x65,0x00,0x72,0x00,0x2E,0x00,0x6C,0x00,0x6F,0x00,0x67,0x00,0x00,0x00,0x00,0x00
    ///     ];
    /// 
    ///     let record = match record::UsnRecordV2::new(&mut Cursor::new(record_buffer)) {
    ///         Ok(record) => record,
    ///         Err(error) => panic!(error)
    ///     };
    /// 
    ///     assert_eq!(record.record_length, 96);
    ///     assert_eq!(record.major_version, 2);
    ///     assert_eq!(record.minor_version, 0);
    ///     assert_eq!(record.file_reference.entry, 115);
    ///     assert_eq!(record.file_reference.sequence, 37224);
    ///     assert_eq!(record.parent_reference.entry, 141883);
    ///     assert_eq!(record.parent_reference.sequence, 7);
    ///     assert_eq!(record.usn, 20342374400);
    ///     assert_eq!(record.timestamp.0, 130266586132760403);
    ///     assert_eq!(format!("{}", record.timestamp), "2013-10-19 12:16:53.276040");
    ///     assert_eq!(record.reason.bits(), 2);
    ///     assert_eq!(record.source_info.bits(), 0);
    ///     assert_eq!(record.security_id, 0);
    ///     assert_eq!(record.file_attributes, 8224);
    ///     assert_eq!(record.file_name_length, 32);
    ///     assert_eq!(record.file_name_offset, 60);
    ///     assert_eq!(record.file_name, "BTDevManager.log");
    /// # }
    /// ```
    pub fn new<T: Read>(mut buffer: T) -> Result<UsnRecordV2, UsnError> {
        let record_length = buffer.read_u32::<LittleEndian>()?;

        // Do some length checks
        if record_length == 0 {
            return Err(
                UsnError::invalid_v2_record(
                    format!("Record length is 0.")
                )
            );
        }
        if record_length > 1024 {
            return Err(
                UsnError::invalid_v2_record(
                    format!("Record length is over 1024.")
                )
            );
        }

        let major_version = buffer.read_u16::<LittleEndian>()?;
        if major_version != 2 {
            return Err(
                UsnError::invalid_v2_record(
                    format!("Major version is not 2")
                )
            );
        }

        let minor_version = buffer.read_u16::<LittleEndian>()?;
        if minor_version != 0 {
            return Err(
                UsnError::invalid_v2_record(
                    format!("Minor version is not 0")
                )
            );
        }

        let file_reference = MftReference::from_reader(&mut buffer)?;
        let parent_reference = MftReference::from_reader(&mut buffer)?;
        let usn = buffer.read_u64::<LittleEndian>()?;
        let timestamp = WinTimestamp::new(buffer.read_u64::<LittleEndian>()?);
        let reason = flags::Reason::from_bits_truncate(buffer.read_u32::<LittleEndian>()?);
        let source_info = flags::SourceInfo::from_bits_truncate(buffer.read_u32::<LittleEndian>()?);
        let security_id = buffer.read_u32::<LittleEndian>()?;
        let file_attributes = buffer.read_u32::<LittleEndian>()?;
        let file_name_length = buffer.read_u16::<LittleEndian>()?;
        let file_name_offset = buffer.read_u16::<LittleEndian>()?;

        let mut name_buffer = vec![0; file_name_length as usize];
        buffer.read_exact(&mut name_buffer)?;

        let file_name = match UTF_16LE.decode(&name_buffer, DecoderTrap::Ignore) {
            Ok(s) => s,
            Err(_e) => {
                return Err(UsnError::io_error(
                    format!("decode Error: {:?}", _e)
                ));
            },
        };

        Ok(
            UsnRecordV2 {
                record_length,
                major_version,
                minor_version,
                file_reference,
                parent_reference,
                usn,
                timestamp,
                reason,
                source_info,
                security_id,
                file_attributes,
                file_name_length,
                file_name_offset,
                file_name
            }
        )
    }
}