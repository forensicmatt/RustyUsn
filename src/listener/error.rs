use std::ptr;
use std::string;
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
    Utf16Error,
    FromUtf16Error,
    WindowsError,
    InvalidUsnJournalData
}
#[derive(Debug)]
pub struct Error {
    /// Formated error message
    pub message: String,
    /// The type of error
    pub kind: ErrorKind
}
impl Error{
    #[allow(dead_code)]
    pub fn from_windows_error_code(err_code: u32)->Self{
        let err_str = format_win_error(Some(err_code));
        Error {
            message: err_str,
            kind: ErrorKind::WindowsError
        }
    }

    #[allow(dead_code)]
    pub fn from_windows_last_error()->Self{
        let err_str = format_win_error(None);
        Error {
            message: err_str,
            kind: ErrorKind::WindowsError
        }
    }

    #[allow(dead_code)]
    pub fn invalid_usn_journal_data(size: usize)->Self{
        let err_str = format!("Unknown size for UsnJournalData structure: {}", size);

        Error {
            message: err_str,
            kind: ErrorKind::InvalidUsnJournalData
        }
    }
}
impl From<string::FromUtf16Error> for Error {
    fn from(err: string::FromUtf16Error) -> Self {
        Error {
            message: format!("{}",err),
            kind: ErrorKind::FromUtf16Error
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