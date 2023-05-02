use chrono::{DateTime, Days, Duration, Months, TimeZone};
use clap::error::Error;
use regex::Regex;
use std::path::Path;

use crate::config::read_config_file;

fn offset_by_interval<T: TimeZone>(now: DateTime<T>, interval: &str) -> DateTime<T> {
    let pattern = Regex::new(r"^([0-9]+)\W*([Mwdhms]|month|months|week|weeks|day|days|hour|hours|minute|minutes|second|seconds)$").unwrap();
    let captures = pattern
        .captures(interval)
        .unwrap_or_else(|| panic!("invalid date string {}", interval));
    let count: u32 = captures[1].parse().unwrap();
    let unit: &str = &captures[2];
    match unit {
        "M" | "month" | "months" => now - Months::new(count),
        "w" | "week" | "weeks" => now - Duration::weeks(count.into()),
        "d" | "day" | "days" => now - Days::new(count.into()),
        "h" | "hour" | "hours" => now - Duration::hours(count.into()),
        "m" | "minute" | "minutes" => now - Duration::minutes(count.into()),
        "s" | "second" | "seconds" => now - Duration::seconds(count.into()),
        _ => panic!("invalid time unit {}", unit),
    }
}

pub fn sync(file: &Path) -> Result<(), Error> {
    todo!()
}

#[cfg(test)]
mod test {
    use chrono::{TimeZone, Utc};

    use super::*;

    #[test]
    fn test_offset_by_interval_month() {
        let now = Utc.with_ymd_and_hms(2000, 7, 1, 0, 0, 0).unwrap();
        assert_eq!(
            offset_by_interval(now, "0 M"),
            Utc.with_ymd_and_hms(2000, 7, 1, 0, 0, 0).unwrap(),
        );
        assert_eq!(
            offset_by_interval(now, "1M"),
            Utc.with_ymd_and_hms(2000, 6, 1, 0, 0, 0).unwrap(),
        );
        assert_eq!(
            offset_by_interval(now, "6month"),
            Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap(),
        );
        assert_eq!(
            offset_by_interval(now, "10 months"),
            Utc.with_ymd_and_hms(1999, 9, 1, 0, 0, 0).unwrap(),
        );
    }

    #[test]
    fn test_offset_by_interval_week() {
        let now = Utc.with_ymd_and_hms(2000, 7, 28, 0, 0, 0).unwrap();
        assert_eq!(
            offset_by_interval(now, "0 w"),
            Utc.with_ymd_and_hms(2000, 7, 28, 0, 0, 0).unwrap(),
        );
        assert_eq!(
            offset_by_interval(now, "1w"),
            Utc.with_ymd_and_hms(2000, 7, 21, 0, 0, 0).unwrap(),
        );
        assert_eq!(
            offset_by_interval(now, "2week"),
            Utc.with_ymd_and_hms(2000, 7, 14, 0, 0, 0).unwrap(),
        );
        assert_eq!(
            offset_by_interval(now, "4 weeks"),
            Utc.with_ymd_and_hms(2000, 6, 30, 0, 0, 0).unwrap(),
        );
    }

    #[test]
    fn test_offset_by_interval_day() {
        let now = Utc.with_ymd_and_hms(2000, 7, 30, 0, 0, 0).unwrap();
        assert_eq!(
            offset_by_interval(now, "0 d"),
            Utc.with_ymd_and_hms(2000, 7, 30, 0, 0, 0).unwrap(),
        );
        assert_eq!(
            offset_by_interval(now, "1d"),
            Utc.with_ymd_and_hms(2000, 7, 29, 0, 0, 0).unwrap(),
        );
        assert_eq!(
            offset_by_interval(now, "2day"),
            Utc.with_ymd_and_hms(2000, 7, 28, 0, 0, 0).unwrap(),
        );
        assert_eq!(
            offset_by_interval(now, "30 days"),
            Utc.with_ymd_and_hms(2000, 6, 30, 0, 0, 0).unwrap(),
        );
    }

    #[test]
    fn test_offset_by_interval_hour() {
        let now = Utc.with_ymd_and_hms(2000, 7, 3, 0, 0, 0).unwrap();
        assert_eq!(
            offset_by_interval(now, "0 h"),
            Utc.with_ymd_and_hms(2000, 7, 3, 0, 0, 0).unwrap(),
        );
        assert_eq!(
            offset_by_interval(now, "1h"),
            Utc.with_ymd_and_hms(2000, 7, 2, 23, 0, 0).unwrap(),
        );
        assert_eq!(
            offset_by_interval(now, "24hour"),
            Utc.with_ymd_and_hms(2000, 7, 2, 0, 0, 0).unwrap(),
        );
        assert_eq!(
            offset_by_interval(now, "30 hours"),
            Utc.with_ymd_and_hms(2000, 7, 1, 18, 0, 0).unwrap(),
        );
    }

    #[test]
    fn test_offset_by_interval_minute() {
        let now = Utc.with_ymd_and_hms(2000, 7, 2, 0, 0, 0).unwrap();
        assert_eq!(
            offset_by_interval(now, "0 m"),
            Utc.with_ymd_and_hms(2000, 7, 2, 0, 0, 0).unwrap(),
        );
        assert_eq!(
            offset_by_interval(now, "1m"),
            Utc.with_ymd_and_hms(2000, 7, 1, 23, 59, 0).unwrap(),
        );
        assert_eq!(
            offset_by_interval(now, "30minute"),
            Utc.with_ymd_and_hms(2000, 7, 1, 23, 30, 0).unwrap(),
        );
        assert_eq!(
            offset_by_interval(now, "90 minutes"),
            Utc.with_ymd_and_hms(2000, 7, 1, 22, 30, 0).unwrap(),
        );
    }

    #[test]
    fn test_offset_by_interval_second() {
        let now = Utc.with_ymd_and_hms(2000, 7, 2, 0, 0, 0).unwrap();
        assert_eq!(
            offset_by_interval(now, "0 s"),
            Utc.with_ymd_and_hms(2000, 7, 2, 0, 0, 0).unwrap(),
        );
        assert_eq!(
            offset_by_interval(now, "1s"),
            Utc.with_ymd_and_hms(2000, 7, 1, 23, 59, 59).unwrap(),
        );
        assert_eq!(
            offset_by_interval(now, "30second"),
            Utc.with_ymd_and_hms(2000, 7, 1, 23, 59, 30).unwrap(),
        );
        assert_eq!(
            offset_by_interval(now, "90 seconds"),
            Utc.with_ymd_and_hms(2000, 7, 1, 23, 58, 30).unwrap(),
        );
    }
}
