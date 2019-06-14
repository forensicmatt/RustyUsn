use std::thread;
use std::fs::File;
use std::process::exit;
use std::time::Duration;
use std::sync::mpsc::Sender;
use crate::record::UsnEntry;
use crate::usn::IterRecords;
use crate::listener::winfuncs::{
    query_usn_journal,
    read_usn_journal,
    ReadUsnJournalData
};
use byteorder::{ByteOrder, LittleEndian};


pub struct UsnVolumeListener {
    source: String,
    sleep_ms: u64,
    historical_flag: bool,
    sender: Sender<UsnEntry>
}

impl UsnVolumeListener {
    pub fn new(source: String, historical_flag: bool, sender: Sender<UsnEntry>) -> UsnVolumeListener {
        let sleep_ms = 100;

        UsnVolumeListener{
            source,
            sleep_ms,
            historical_flag,
            sender
        }
    }

    pub fn listen_to_volume(self) {
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
                    next_start_usn = LittleEndian::read_u64(&buffer[0..8]);

                    let record_iterator = IterRecords::new(
                        format!("{}", self.source),
                        buffer.to_vec(), 
                        0, 
                        buffer.len()
                    );

                    let mut record_count: u64 = 0;
                    for usn_entry in record_iterator {
                        match self.sender.send(usn_entry) {
                            Ok(_) => {
                                record_count += 1;
                            },
                            Err(error) => {
                                eprintln!("error sending usn entry: {}", error);
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