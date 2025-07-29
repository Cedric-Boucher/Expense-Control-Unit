use chrono::{DateTime, TimeZone, Utc};
use sqlx::types::time::OffsetDateTime;



pub fn convert_time_to_chrono(dt: OffsetDateTime) -> DateTime<Utc> {
    let timestamp = dt.unix_timestamp();
    let nanosecond = dt.nanosecond();

    Utc.timestamp_opt(timestamp, nanosecond).unwrap()
}

pub fn convert_chrono_to_time(dt: DateTime<Utc>) -> OffsetDateTime {
    let timestamp = dt.timestamp();
    let nanosecond = dt.timestamp_subsec_nanos();

    OffsetDateTime::from_unix_timestamp_nanos(timestamp as i128 * 1_000_000_000 + nanosecond as i128).expect("Valid timestamp")
}
