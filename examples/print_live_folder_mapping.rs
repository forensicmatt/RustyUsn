use rusty_usn::liveusn::live::WindowsLiveNtfs;
use std::process::exit;
use serde_json;

fn main() {
    let live_ntfs = match WindowsLiveNtfs::from_volume_path(r"\\.\C:") {
        Ok(ntfs) => ntfs,
        Err(error) => {
            eprintln!("Error creating WindowsLiveNtfs: {:?}", error);
            exit(-1);
        }
    };

    eprintln!("creating live folder mapping...");
    let folder_mapping = live_ntfs.get_folder_mapping();
    let json_str = serde_json::to_string(
        &folder_mapping
    ).unwrap();

    println!("{}", json_str);
}