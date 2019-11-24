use rusty_usn::liveusn::live::WindowsLiveNtfs;
use std::process::exit;

fn main() {
    let live_ntfs = match WindowsLiveNtfs::from_volume_path(r"\\.\C:") {
        Ok(ntfs) => ntfs,
        Err(error) => {
            eprintln!("Error creating WindowsLiveNtfs: {:?}", error);
            exit(-1);
        }
    };

    println!("max entry: {}", live_ntfs.get_max_entry());
    let entry_iterator = live_ntfs.get_entry_iterator();
    for entry_result in entry_iterator {
        match entry_result {
            Ok(entry) => {
                // We only want directories
                if !entry.is_dir() {
                    continue;
                }

                // Get the best name attribute or <NA>
                let name = match entry.find_best_name_attribute() {
                    Some(fn_attr) => {
                        fn_attr.name
                    },
                    None => {
                        format!("<NA> [base: {}]", entry.header.base_reference.entry)
                    }
                };
                
                println!("record: {} -> {}", entry.header.record_number, name)
            },
            Err(error) => {
                eprintln!("{:?}", error);
            }
        }
    }
}