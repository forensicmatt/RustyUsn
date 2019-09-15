extern crate rusty_usn;


#[cfg(feature = "windows")]
#[test]
fn live_mapping_test() {
    use rusty_usn::liveusn::mapping;

    let mut live_mapping = mapping::LiveMapping::from_volume_path(
        r"\\.\C:"
    ).unwrap();

    let path = live_mapping.get_full_path(32).unwrap();
    assert_eq!(path, r"$Extend\$RmMetadata\$TxfLog\$Tops");
}
