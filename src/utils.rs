use time::Duration;
use chrono::{DateTime, NaiveDate, Utc};


/// Convert a u64 Windows 100 nanosecond timestamp to a chrono DateTime
///
pub fn u64_to_datetime(timestamp_u64: u64) -> DateTime<Utc> {
    DateTime::from_utc(
        NaiveDate::from_ymd(1601, 1, 1)
            .and_hms_nano(0, 0, 0, 0)
            + Duration::microseconds(
                (timestamp_u64 / 10) as i64
            ),
        Utc,
    )
}