extern crate rusty_usn;

#[cfg(feature = "windows")]
#[test]
fn win_error_code_test() {
    use rusty_usn::listener::error::format_win_error;
    
    let error_str = format_win_error(
        Some(2)
    );

    assert_eq!(error_str, "The system cannot find the file specified.\r\n");
}