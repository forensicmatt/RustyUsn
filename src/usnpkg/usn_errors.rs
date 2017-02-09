use std::fmt;
use std::fmt::Display;
use std::result::Result as StdResult;
use std::io;

pub type Result<T> = StdResult<T, UsnError>;

#[derive(Debug)]
pub enum ErrorKind {
    InvalidUsnV2Record,
    UnsupportedVersion,
    IoError,
}

/// USN Record Parsing Error
#[derive(Debug)]
pub struct UsnError {
    /// Formated error message
    pub message: String,
    /// The type of error
    pub kind: ErrorKind,
    /// Any additional information passed along, such as the argument name that caused the error
    pub info: Option<Vec<String>>,
}

impl UsnError{
    #[allow(dead_code)]
    pub fn invalid_v2_record(err: String)->Self{
        UsnError {
            message: format!("{}",err),
            kind: ErrorKind::InvalidUsnV2Record,
            info: Some(vec![]),
        }
    }
    #[allow(dead_code)]
    pub fn unsupported_version(err: String)->Self{
        UsnError {
            message: format!("{}",err),
            kind: ErrorKind::UnsupportedVersion,
            info: Some(vec![]),
        }
    }
    #[allow(dead_code)]
    pub fn io_error(err: String)->Self{
        UsnError {
            message: format!("{}",err),
            kind: ErrorKind::InvalidUsnV2Record,
            info: Some(vec![]),
        }
    }
}

impl From<io::Error> for UsnError {
    fn from(err: io::Error) -> Self {
        UsnError {
            message: format!("{}",err),
            kind: ErrorKind::IoError,
            info: Some(vec![]),
        }
    }
}

impl Display for UsnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { writeln!(f, "{}", self.message) }
}