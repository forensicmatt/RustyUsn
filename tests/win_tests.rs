extern crate rusty_usn;

#[cfg(feature = "windows")]
#[test]
fn query_test() {
    use std::fs::File;
    
    use rusty_usn::liveusn::winfuncs::{
        query_usn_journal
    };

    let file_handle = match File::open("\\\\.\\C:") {
        Ok(handle) => handle,
        Err(error) => panic!(error)
    };

    match query_usn_journal(&file_handle) {
        Ok(journal_info) => {
            println!("{:#?}", journal_info);
        },
        Err(error) => panic!(error)
    }
}