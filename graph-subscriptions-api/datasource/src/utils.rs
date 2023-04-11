use chrono::{Datelike as _, TimeZone, Utc};

/// Given the timestamp of the message, build a date range for:
/// - the start of the day, in unix-timestamp, represented by the timestamp
/// - the end of the day, in unix-timestamp, represented by the timestamp
///
/// # Arguments
///
/// - `timestamp` - the unix-timestamp when the message was received
///
/// # Examples
/// ```
/// use crate::utils::build_timerange_from_message_timestamp;
/// let timestamp: i64 = 1680027362;
/// let (start, end) = build_timerange_timestamp(timestamp);
/// assert_eq!(start, 1679961600);
/// assert_eq!(end, 1680047999);
/// ```
pub fn build_timerange_timestamp(timestamp: i64) -> (i64, i64) {
    let timestamp_utc = Utc.timestamp_opt(timestamp, 0).unwrap();
    let start = Utc
        .with_ymd_and_hms(
            timestamp_utc.year(),
            timestamp_utc.month(),
            timestamp_utc.day(),
            0,
            0,
            0,
        )
        .unwrap()
        .timestamp();
    let end = Utc
        .with_ymd_and_hms(
            timestamp_utc.year(),
            timestamp_utc.month(),
            timestamp_utc.day(),
            23,
            59,
            59,
        )
        .unwrap()
        .timestamp();

    (start, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_the_start_and_end_for_now() {
        let now: i64 = 1680027362;
        let expected_start: i64 = 1679961600;
        let expected_end: i64 = 1680047999;

        let (actual_start, actual_end) = build_timerange_timestamp(now);

        assert_eq!(actual_start, expected_start);
        assert_eq!(actual_end, expected_end);
    }
}
