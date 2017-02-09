extern crate rustyusn;
use rustyusn::usnpkg::timestamp;
#[test]
fn test_timestamp() {
    let raw_timestamp: &[u8] = &[0x53,0xC7,0x8B,0x18,0xC5,0xCC,0xCE,0x01];

    let time_stamp: timestamp::WinTimestamp = match timestamp::raw_to_wintimestamp(raw_timestamp){
        Ok(time_stamp) => time_stamp,
        Err(error) => panic!(error)
    };
    assert_eq!(format!("{}",time_stamp),"2013-10-19 12:16:53.276040");
    assert_eq!(format!("{:?}",time_stamp),"2013-10-19 12:16:53.276040");
    assert_eq!(time_stamp.0,130266586132760403);
}
