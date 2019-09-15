extern crate rusty_usn;
extern crate serde_json;
use rusty_usn::record;
use byteorder::{ByteOrder, LittleEndian};


#[test]
fn reference_128() {
    let ref_buffer: &[u8] = &[
        0xC8,0x07,0x00,0x00,0x00,0x00,0x02,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00
    ];

    let ref128 = record::Ntfs128Reference(
        LittleEndian::read_u128(&ref_buffer[0..16])
    );

    assert_eq!(ref128.0, 562949953423304);
    let file_ref = ref128.as_mft_reference();
    assert_eq!(file_ref.entry, 1992);
    assert_eq!(file_ref.sequence, 2);

    let json_str = serde_json::to_string(
        &ref128
    ).unwrap();
    assert_eq!(json_str, r#"{"u128":"562949953423304","entry":1992,"sequence":2}"#);
}