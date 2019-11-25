use std::fs::File;
use std::io::Read;
use mft::MftEntry;
use byteorder::{ReadBytesExt, LittleEndian};
use crate::mapping::FolderMapping;
use crate::liveusn::winfuncs;
use crate::liveusn::error::UsnLiveError;
use crate::liveusn::ntfs::NtfsVolumeData;
use winstructs::ntfs::mft_reference::MftReference;


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


/// Struct for interacting with a live NTFS volume via Windows API
///
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

    pub fn get_folder_mapping(self) -> FolderMapping {
        // Create the folder mapping
        let mut folder_mapping = FolderMapping::new();

        // Iterate over live MFT entries
        let entry_iter = self.get_entry_iterator();
        for entry_result in entry_iter {
            match entry_result {
                Ok(entry) => {
                    // We only want directories
                    if !entry.is_dir() {
                        continue;
                    }

                    let mut l_entry = entry.header.record_number;
                    let mut l_sequence = entry.header.sequence;

                    // if entry is child, set entry and sequence to parent
                    if entry.header.base_reference.entry != 0 {
                        l_entry = entry.header.base_reference.entry;
                        l_sequence = entry.header.base_reference.sequence;
                    }

                    // Get the best name attribute or <NA>
                    let fn_attr = match entry.find_best_name_attribute() {
                        Some(fn_attr) => fn_attr,
                        None => continue
                    };

                    // Entry reference for our key
                    let entry_reference = MftReference::new(
                        l_entry,
                        l_sequence
                    );

                    // Add this entry to the folder mapping
                    folder_mapping.add_mapping(
                        entry_reference,
                        fn_attr.name,
                        fn_attr.parent
                    );
                },
                Err(error) => {
                    eprintln!("{:?}", error);
                }
            }
        }

        folder_mapping
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

    pub fn get_max_entry(&self) -> u64 {
        self.ntfs_volume_data.get_max_entry()
    }

    pub fn get_entry_iterator(self) -> LiveMftEntryIterator {
        let last_entry = self.get_max_entry();

        LiveMftEntryIterator {
            live_ntfs: self,
            current_entry: last_entry as i64 - 1
        }
    }
}


/// Iterator to iterate mft entries on a live NTFS volume. The iterator 
/// returns entries in reverse order (highest to lowest) which maximizes 
/// performance due to Windows API because FSCTL_GET_NTFS_FILE_RECORD
/// retrieves the first file record that is in use and is of a lesser than or equal 
/// ordinal value to the requested file reference number.
/// The current entry must start at the highest to lowest and be one less than
/// the max entry
/// 
pub struct LiveMftEntryIterator {
    live_ntfs: WindowsLiveNtfs,
    current_entry: i64
}
impl Iterator for LiveMftEntryIterator {
    type Item = Result<MftEntry, UsnLiveError>;

    // It is fastest to iterate file entries from highest to lowest becuase
    // the Windows API fetches the lowest allocated entry if an entry is queried
    // that is unallocated. This prevents us from having to iterate through blocks
    // of unallocated entries (in which case the same entry will be returned till the
    // next allocated) until we find the next allocated.
    fn next(&mut self) -> Option<Result<MftEntry, UsnLiveError>> {
        while self.current_entry >= 0 {
            // Get MFT entry for current entry
            let mft_entry = match self.live_ntfs.get_entry(
                self.current_entry as i64
            ) {
                Ok(entry) => entry,
                Err(error) => {
                    self.current_entry -= 1;
                    return Some(Err(error))
                }
            };

            // Deincrement the entry by 1
            self.current_entry = mft_entry.header.record_number as i64 - 1;

            return Some(Ok(mft_entry));
        }

        None
    }
}