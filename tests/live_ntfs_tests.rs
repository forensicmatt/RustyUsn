extern crate rusty_usn;


#[cfg(feature = "windows")]
#[test]
fn live_get_entry_test() {
    use rusty_usn::liveusn::live;

    let mut live_ntfs = live::WindowsLiveNtfs::from_volume_path(
        r"\\.\C:"
    ).unwrap();

    let mft_entry = live_ntfs.get_entry(0).unwrap();
    let mft_name = match mft_entry.find_best_name_attribute() {
        Some(attr) => attr.name,
        None => {
            eprintln!("Error getting mft entry name for record 0. {:?}", mft_entry);
            panic!("Error getting mft entry name for record 0. {:?}", mft_entry);
        }
    };
    assert_eq!(mft_name, r"$MFT");
}

#[cfg(feature = "windows")]
#[test]
fn live_volume_info_test() {
    use std::fs::File;
    use rusty_usn::liveusn::winfuncs;

    let file_handle = File::open(r"\\.\C:").unwrap();
    let _volume_data = winfuncs::get_ntfs_volume_data(
        &file_handle
    ).unwrap();
}

#[cfg(feature = "windows")]
#[test]
fn parse_live_volume_data_test() {
    use rusty_usn::liveusn::ntfs;

    let volume_buffer: &[u8] = &[
        0x23,0x6E,0x46,0x0A,0xA3,0x46,0x0A,0xA8,0xFF,0x77,0x7F,0x3B,0x00,0x00,0x00,0x00,
        0xFF,0xEE,0x6F,0x07,0x00,0x00,0x00,0x00,0xA3,0x64,0xA1,0x00,0x00,0x00,0x00,0x00,
        0x70,0x14,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x02,0x00,0x00,0x00,0x10,0x00,0x00,
        0x00,0x04,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0xCC,0x2A,0x00,0x00,0x00,0x00,
        0x00,0x00,0x0C,0x00,0x00,0x00,0x00,0x00,0x02,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
        0xE0,0x03,0x10,0x06,0x00,0x00,0x00,0x00,0x60,0x54,0x10,0x06,0x00,0x00,0x00,0x00,
        0x20,0x00,0x00,0x00,0x03,0x00,0x01,0x00,0x00,0x02,0x00,0x00,0x02,0x00,0x00,0x00,
        0x00,0x01,0x00,0x00,0xFF,0xFF,0xFF,0xFF,0x3E,0x00,0x00,0x00,0x00,0x00,0x00,0x40,
    ];

    let volume_data = ntfs::NtfsVolumeData::from_buffer(volume_buffer);

    let volume_data_json_str = serde_json::to_string(
        &volume_data
    ).unwrap();

    assert_eq!(&volume_data_json_str, r#"{"volume_serial_number":-6338175859504550365,"number_sectors":998209535,"total_clusters":124776191,"free_clusters":10577059,"total_reserved":5232,"bytes_per_sector":512,"bytes_per_cluster":4096,"bytes_per_file_record_segment":1024,"clusters_per_file_record_segment":0,"mft_valid_data_length":718012416,"mft_start_lcn":786432,"mft_2_start_lcn":2,"mft_zone_start":101712864,"mft_zone_end":101733472,"ntfs_extended_volume_data":{"byte_count":32,"major_version":3,"minor_version":1,"bytes_per_physical_sector":512,"lfs_major_version":2,"lfs_minor_version":0,"max_device_trim_extent_count":256,"max_device_trim_byte_count":4294967295,"max_volume_trim_extent_count":62,"max_volume_trim_byte_count":1073741824}}"#);

    assert_eq!(volume_data.volume_serial_number, -6338175859504550365);
    assert_eq!(volume_data.number_sectors, 998209535);
    assert_eq!(volume_data.total_clusters, 124776191);
    assert_eq!(volume_data.free_clusters, 10577059);
    assert_eq!(volume_data.total_reserved, 5232);
    assert_eq!(volume_data.bytes_per_sector, 512);
    assert_eq!(volume_data.bytes_per_cluster, 4096);
    assert_eq!(volume_data.bytes_per_file_record_segment, 1024);
    assert_eq!(volume_data.clusters_per_file_record_segment, 0);
    assert_eq!(volume_data.mft_valid_data_length, 718012416);
    assert_eq!(volume_data.mft_start_lcn, 786432);
    assert_eq!(volume_data.mft_2_start_lcn, 2);
    assert_eq!(volume_data.mft_zone_start, 101712864);
    assert_eq!(volume_data.mft_zone_end, 101733472);
}
