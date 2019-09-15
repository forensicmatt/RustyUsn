use std::io;
use std::ptr;
use mft::err::Error as MftError;
use winapi::shared::ntdef::WCHAR;
use winapi::um::winbase::{
    FormatMessageW, 
    FORMAT_MESSAGE_FROM_SYSTEM, 
    FORMAT_MESSAGE_IGNORE_INSERTS,
};
use winapi::um::errhandlingapi::GetLastError;


#[derive(Debug)]
pub enum ErrorKind {
    IoError,
    MftError,
    InvalidUsnJournalData,
    MftAttributeError,
    WindowsError
}

#[derive(Debug)]
pub struct UsnLiveError {
    pub message: String,
    pub kind: ErrorKind,
}

impl UsnLiveError {
    #[allow(dead_code)]
    pub fn unable_to_get_name_attr(message: &str) -> Self{
        UsnLiveError {
            message: message.to_owned(),
            kind: ErrorKind::MftAttributeError
        }
    }

    #[allow(dead_code)]
    pub fn from_windows_error_code(err_code: u32) -> Self{
        let err_str = format_win_error(
            Some(err_code)
        );

        UsnLiveError {
            message: err_str,
            kind: ErrorKind::WindowsError
        }
    }

    #[allow(dead_code)]
    pub fn from_windows_last_error() -> Self{
        let err_str = format_win_error(None);
        UsnLiveError {
            message: err_str,
            kind: ErrorKind::WindowsError
        }
    }

    #[allow(dead_code)]
    pub fn invalid_usn_journal_data(size: usize)->Self{
        let err_str = format!("Unknown size for UsnJournalData structure: {}", size);

        UsnLiveError {
            message: err_str,
            kind: ErrorKind::InvalidUsnJournalData
        }
    }

    #[allow(dead_code)]
    pub fn invalid_thing(message: &str)->Self{
        UsnLiveError {
            message: message.to_owned(),
            kind: ErrorKind::WindowsError
        }
    }
}

impl From<MftError> for UsnLiveError {
    fn from(err: MftError) -> Self {
        UsnLiveError {
            message: format!("{}", err),
            kind: ErrorKind::MftError
        }
    }
}

impl From<io::Error> for UsnLiveError {
    fn from(err: io::Error) -> Self {
        UsnLiveError {
            message: format!("{}", err),
            kind: ErrorKind::IoError,
        }
    }
}


pub fn format_win_error(error_code: Option<u32>) -> String {
    let mut message_buffer = [0 as WCHAR; 2048];
    let error_num: u32 = match error_code {
        Some(code) => code,
        None => unsafe { GetLastError() }
    };

    let message_size = unsafe { 
        FormatMessageW(
            FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
            ptr::null_mut(),
            error_num,
            0,
            message_buffer.as_mut_ptr(),
            message_buffer.len() as u32,
            ptr::null_mut(),
        )
    };

    if message_size == 0 {
        return format_win_error(None);
    } else {
        let err_msg = String::from_utf16(
            &message_buffer[..message_size as usize]
        ).unwrap();
        return err_msg;
    }
}
