use std::thread;
use std::fs::File;
use std::process::exit;
use std::time::Duration;
use std::sync::mpsc::Sender;
use serde_json::value::Value;
use byteorder::{ByteOrder, LittleEndian};
use crate::flags;
use crate::record::EntryMeta;
use crate::liveusn::winfuncs::{
    query_usn_journal,
    read_usn_journal,
};
use crate::usn::IterRecordsByIndex;
use crate::liveusn::error::UsnLiveError;
use crate::liveusn::live::WindowsLiveNtfs;
use crate::liveusn::ntfs::ReadUsnJournalData;


pub struct UsnVolumeListener {
    source: String,
    sleep_ms: u64,
    historical_flag: bool,
    sender: Sender<Value>
}

impl UsnVolumeListener {
    pub fn new(source: String, historical_flag: bool, sender: Sender<Value>) -> Self {
        let sleep_ms = 100;

        UsnVolumeListener {
            source,
            sleep_ms,
            historical_flag,
            sender
        }
    }

    pub fn listen_to_volume(self) -> Result<(), UsnLiveError> {
        let live_volume = WindowsLiveNtfs::from_volume_path(
            &self.source
        )?;

        let mut mapping = live_volume.get_folder_mapping();

        let file_handle = match File::open(self.source.clone()) {
            Ok(handle) => handle,
            Err(error) => {
                eprintln!("{}", error);
                exit(-1);
            }
        };

        let usn_journal_data = match query_usn_journal(&file_handle) {
            Ok(journal_info) => {
                debug!("{:#?}", journal_info);
                journal_info
            },
            Err(error) => {
                eprintln!("{:?}", error);
                exit(-1);
            }
        };

        let mut next_start_usn: u64 = usn_journal_data.get_next_usn();
        let catch_up_usn = next_start_usn;

        if self.historical_flag {
            next_start_usn = 0;
        }

        loop {
            let mut buffer = vec![0u8; 4096];

            let read_data = ReadUsnJournalData::from_usn_journal_data(
                usn_journal_data.clone()
            ).with_start_usn(next_start_usn);

            let count: u64 = match read_usn_journal(&file_handle, read_data, &mut buffer) {
                Ok(buffer) => {
                    // The first 8 bytes are the usn of the next record NOT in the buffer,
                    // use this value as the next_start_usn
                    next_start_usn = LittleEndian::read_u64(
                        &buffer[0..8]
                    );

                    let entry_meta = EntryMeta::new(
                        &self.source, 0
                    );

                    let record_iterator = IterRecordsByIndex::new(
                        entry_meta,
                        buffer[8..].to_vec()
                    );

                    let mut record_count: u64 = 0;
                    for usn_entry in record_iterator {
                        let entry_usn = usn_entry.record.get_usn();
                        let file_name = usn_entry.record.get_file_name();
                        let file_ref = usn_entry.record.get_file_reference();
                        let reason_code = usn_entry.record.get_reason_code();
                        let parent_ref = usn_entry.record.get_parent_reference();
                        let file_attributes = usn_entry.record.get_file_attributes();

                        if file_attributes.contains(flags::FileAttributes::FILE_ATTRIBUTE_DIRECTORY){
                            if reason_code.contains(flags::Reason::USN_REASON_RENAME_OLD_NAME) {
                                // We can remove old names from the mapping because we no longer need these.
                                // On new names, we add the name to the mapping.
                                mapping.remove_mapping(
                                    file_ref
                                );
                            }
                            else if reason_code.contains(flags::Reason::USN_REASON_FILE_DELETE) {
                                // If we are starting from historical entries, we need to add deleted
                                // entries to the map until we catch up to the current system, then we can 
                                // start removing deleted entries. This is because our mapping cannot
                                // get unallocated entries from the MFT via the Windows API.
                                if self.historical_flag && entry_usn < catch_up_usn {
                                    mapping.add_mapping(
                                        file_ref, 
                                        file_name.clone(), 
                                        parent_ref
                                    )
                                } else {
                                    mapping.remove_mapping(
                                        file_ref
                                    );
                                }
                            } else if reason_code.contains(flags::Reason::USN_REASON_RENAME_NEW_NAME) ||
                                reason_code.contains(flags::Reason::USN_REASON_FILE_CREATE) {
                                // If its a new name or creation, we need to updated the mapping
                                mapping.add_mapping(
                                    file_ref, 
                                    file_name.clone(), 
                                    parent_ref
                                )
                            }
                        }

                        // Enumerate the path of this record from the FolderMapping
                        let full_path = match mapping.enumerate_path(
                            parent_ref.entry,
                            parent_ref.sequence
                        ){
                            Some(path) => path,
                            None => "[<unknown>]".to_string()
                        };

                        let mut entry_value = match usn_entry.to_json_value(){
                            Ok(value) => value,
                            Err(e) => {
                                eprintln!("Error serializing entry to json value {:?}: {:?}", usn_entry, e);
                                continue;
                            }
                        };

                        let full_file_name = format!("{}/{}", &full_path, &file_name);

                        let map = entry_value.as_object_mut().unwrap();
                        map.insert(
                            "full_path".to_string(), 
                            Value::String(full_file_name)
                        );

                        match self.sender.send(entry_value) {
                            Ok(_) => {
                                record_count += 1;
                            },
                            Err(error) => {
                                eprintln!("error sending usn entry: {:?}", error);
                            }
                        }
                    }

                    record_count
                },
                Err(error) => {
                    println!("{:#?}", error);
                    break
                }
            };

            // need to sleep to minimize resources
            if count == 0 {
                thread::sleep(
                    Duration::from_millis(
                        self.sleep_ms
                    )
                );
            }
        }

        Ok(())
    }
}