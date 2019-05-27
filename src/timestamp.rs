use chrono;
use std::fmt;
use std::fmt::{Display, Debug};
use serde::ser;

pub static mut TIMESTAMP_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S%.6f";

#[derive(Clone)]
pub struct WinTimestamp(
    pub u64
);
impl WinTimestamp {
    pub fn new(u64_int: u64) -> WinTimestamp {
        WinTimestamp(u64_int)
    }

    pub fn to_datetime(&self) -> chrono::NaiveDateTime {
        // Get nanoseconds (100-nanosecond intervals)
        let t_micro = self.0 / 10;
        // Add microseconds to timestamp via Duration
        (
            chrono::NaiveDate::from_ymd(
                1601, 1, 1
            ).and_hms_nano(0, 0, 0, 0) + // Win Epoc = 1601-01-01
            chrono::Duration::microseconds(t_micro as i64)
        ) as chrono::NaiveDateTime
    }
}
impl Display for WinTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.to_datetime())
    }
}
impl Debug for WinTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.to_datetime())
    }
}
impl ser::Serialize for WinTimestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: ser::Serializer
    {
        serializer.serialize_str(
            &format!("{}",
            self.to_datetime().format(unsafe{TIMESTAMP_FORMAT}).to_string())
        )
    }
}