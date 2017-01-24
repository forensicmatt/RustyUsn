extern crate rustyusn;
use rustyusn::usnpkg;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

#[test]
fn usn_record_v2_test() {
    // Buffer for test
    let mut buffer = [0u8; 96];

    // Open Filehandle to test record
    let mut usn_fh = match File::open("testdata/record.usn") {
        Ok(usn_fh) => usn_fh,
        // Handle error here
        Err(error) => panic!("Error: {}",error)
    };

    let bytes_read = usn_fh.read(&mut buffer).unwrap();
    assert_eq!(bytes_read, 96);

    // parse raw buffer
    // let mut record = usnpkg::parse_record(buffer){
    //     Ok(record) => record,
    //     Err(error) => panic!("Error: {}",error)
    // };
    //
    // assert_eq!(record.record_length, 96);
}
