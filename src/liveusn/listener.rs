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
use crate::liveusn::mapping::LiveMapping;
use crate::liveusn::ntfs::ReadUsnJournalData;


pub struct UsnVolumeListener {
    source: String,
    sleep_ms: u64,
    historical_flag: bool,
    sender: Sender<Value>
}

impl UsnVolumeListener {
    pub fn new(source: String, historical_flag: bool, sender: Sender<Value>) -> UsnVolumeListener {
        let sleep_ms = 100;

        UsnVolumeListener {
            source,
            sleep_ms,
            historical_flag,
            sender
        }
    }

    pub fn listen_to_volume(self) {
        let mut live_mapping = match LiveMapping::from_volume_path(
            &self.source
        ){
            Ok(mapping) => mapping,
            Err(e) => {
                eprintln!("Error creating LiveMapping: {:?}", e);
                return;
            }
        };

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
                        let file_name = usn_entry.record.get_file_name();
                        let reason_code = usn_entry.record.get_reason_code();
                        let parent_ref = usn_entry.record.get_parent_reference();
                        let file_attributes = usn_entry.record.get_file_attributes();


                        if file_attributes.contains(flags::FileAttributes::FILE_ATTRIBUTE_DIRECTORY){
                            if reason_code.contains(flags::Reason::USN_REASON_FILE_DELETE) ||
                                reason_code.contains(flags::Reason::USN_REASON_RENAME_OLD_NAME) {
                                // Remove from cache because these will no longer be valid.
                                live_mapping.remove_path_from_cache(
                                    parent_ref.entry as i64
                                );
                            }
                        }

                        let full_path = match live_mapping.get_full_path(
                            parent_ref.entry as i64
                        ) {
                            Ok(path) => path,
                            Err(e) => {
                                eprintln!("Error getting path for entry {}: {:?}", parent_ref.entry, e);
                                continue;
                            }
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
    }
}