use std::fmt;
use std::fmt::Display;
use std::io;
use serde_json::error::Error as SjError;
use winstructs::err::Error as WinstructError;

#[derive(Debug)]
pub enum ErrorKind {
    InvalidUsnRecord,
    InvalidUsnV2Record,
    InvalidUsnV3Record,
    UnsupportedVersion,
    WinstructError,
    Utf16DecodeError,
    IoError,
    SerdeJsonError,
    ValueError,
}

/// USN Record Parsing Error
#[derive(Debug)]
pub struct UsnError {
    pub message: String,
    pub kind: ErrorKind,
}

impl UsnError{
    #[allow(dead_code)]
    pub fn json_value_error(msg: String) -> Self {
        UsnError {
            message: msg,
            kind: ErrorKind::ValueError,
        }
    }

    #[allow(dead_code)]
    pub fn utf16_decode_error(msg: String) -> Self {
        UsnError {
            message: msg,
            kind: ErrorKind::Utf16DecodeError,
        }
    }

    #[allow(dead_code)]
    pub fn unsupported_usn_version(msg: String) -> Self {
        UsnError {
            message: msg,
            kind: ErrorKind::UnsupportedVersion,
        }
    }

    #[allow(dead_code)]
    pub fn invalid_record(msg: String) -> Self {
        UsnError {
            message: msg,
            kind: ErrorKind::InvalidUsnRecord,
        }
    }

    #[allow(dead_code)]
    pub fn invalid_v2_record(msg: String) -> Self {
        UsnError {
            message: msg,
            kind: ErrorKind::InvalidUsnV3Record,
        }
    }

    #[allow(dead_code)]
    pub fn invalid_usn_record_length(msg: String) -> Self {
        UsnError {
            message: msg,
            kind: ErrorKind::InvalidUsnRecord,
        }
    }
}

impl From<io::Error> for UsnError {
    fn from(err: io::Error) -> Self {
        UsnError {
            message: format!("{}", err),
            kind: ErrorKind::IoError,
        }
    }
}

impl From<SjError> for UsnError {
    fn from(err: SjError) -> Self {
        UsnError {
            message: format!("{}", err),
            kind: ErrorKind::SerdeJsonError,
        }
    }
}

impl From<WinstructError> for UsnError {
    fn from(err: WinstructError) -> Self {
        UsnError {
            message: format!("{}", err),
            kind: ErrorKind::IoError,
        }
    }
}

impl Display for UsnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
        writeln!(f, "{}", self.message)
    }
}
