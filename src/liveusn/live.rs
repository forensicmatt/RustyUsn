use std::fs::File;
use std::io::Read;
use mft::MftEntry;
use byteorder::{ReadBytesExt, LittleEndian};
use crate::liveusn::winfuncs;
use crate::liveusn::error::UsnLiveError;
use crate::liveusn::ntfs::NtfsVolumeData;

#[derive(Debug)]
pub struct MftOutputBuffer {
    file_reference_number: u64,
    file_record_length: u32,
    file_record_buffer: Vec<u8>
}

impl MftOutputBuffer {
    pub fn from_buffer<T: Read>(mut raw_buffer: T) -> Result<Self, UsnLiveError> {
        let file_reference_number = raw_buffer.read_u64::<LittleEndian>()?;
        let file_record_length = raw_buffer.read_u32::<LittleEndian>()?;
        let mut file_record_buffer = vec![0; file_record_length as usize];
        
        raw_buffer.read_exact(&mut file_record_buffer)?;

        Ok(
            MftOutputBuffer {
                file_reference_number,
                file_record_length,
                file_record_buffer
            }
        )
    }

    pub fn buffer_as_hex(&self) -> String {
        hex::encode(&self.file_record_buffer)
    }

    pub fn as_entry(&self) -> Result<MftEntry, UsnLiveError> {
        Ok(MftEntry::from_buffer_skip_fixup(
            self.file_record_buffer.clone(),
            self.file_reference_number
        )?)
    }
}


#[derive(Debug)]
pub struct WindowsLiveNtfs {
    volume_path: String,
    volume_handle: File,
    ntfs_volume_data: NtfsVolumeData
}
impl WindowsLiveNtfs {
    pub fn from_volume_path(volume_path: &str) -> Result<Self, UsnLiveError> {
        let file_handle = File::open(&volume_path)?;
        let ntfs_volume_data = winfuncs::get_ntfs_volume_data(
            &file_handle
        )?;

        Ok(
            WindowsLiveNtfs {
                volume_path: volume_path.to_string(),
                volume_handle: file_handle,
                ntfs_volume_data: ntfs_volume_data
            }
        )
    }

    fn get_entry_buffer(&mut self, entry: i64) -> Result<MftOutputBuffer, UsnLiveError> {
        let raw_buffer = winfuncs::query_file_record(
            &self.volume_handle,
            entry,
            self.ntfs_volume_data.bytes_per_file_record_segment
        )?;

        MftOutputBuffer::from_buffer(
            &raw_buffer[..]
        )
    }

    pub fn get_entry(&mut self, entry: i64) -> Result<MftEntry, UsnLiveError> {
        let mft_buffer = self.get_entry_buffer(entry)?;
        mft_buffer.as_entry()
    }
}