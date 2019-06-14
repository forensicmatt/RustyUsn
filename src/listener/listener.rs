use std::fs::File;
use std::process::exit;
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
    sender: Sender<UsnEntry>
}

impl UsnVolumeListener {
    pub fn new(source: String, sender: Sender<UsnEntry>) -> UsnVolumeListener {
        UsnVolumeListener{
            source,
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

        let mut next_start_usn: u64 = 0;
        loop {
            let mut buffer = vec![0u8; 4096];

            let read_data = ReadUsnJournalData::from_usn_journal_data(
                usn_journal_data.clone()
            ).with_start_usn(next_start_usn);

            match read_usn_journal(&file_handle, read_data, &mut buffer) {
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

                    for usn_entry in record_iterator {
                        match self.sender.send(usn_entry) {
                            Ok(_) => {},
                            Err(error) => {
                                eprintln!("CRAP! {}", error);
                            }
                        }
                    }
                },
                Err(error) => {
                    println!("{:#?}", error);
                    break;
                }
            }
        }
    }
}