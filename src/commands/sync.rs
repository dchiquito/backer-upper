use chrono::{DateTime, Days, Duration, Months, TimeZone, Utc};
use clap::error::Error;
use regex::Regex;
use std::path::Path;
use std::process::Command;

use crate::commands::backup::backup;
use crate::config::read_config_file;
use crate::utils::run;

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

fn parse_ls(raw: &str) -> Vec<(String, DateTime<Utc>)> {
    let pattern = Regex::new(r"[drwx\-]{10} [0-9]+\W+\w+\W+\w+\W+[0-9]+ ([0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}.[0-9]+ [+\-][0-9]{4}) (.+)").unwrap();
    pattern
        .captures_iter(raw)
        .map(|captures| {
            (
                captures[2].to_string(),
                DateTime::parse_from_str(&captures[1], "%Y-%m-%d %H:%M:%S.%f %z")
                    //2023-03-25 21:50:22.453741042 -0400
                    // "1983 Apr 13 12:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z");
                    .unwrap()
                    .with_timezone(&Utc),
            )
        })
        .collect()
}

fn find_most_recent_matching(
    files: &[(String, DateTime<Utc>)],
    format: &str,
) -> Option<(String, DateTime<Utc>)> {
    files
        .iter()
        .find(|(name, time)| {
            println!("{} {}", name, time.format(format));
            name == &format!("{}", time.format(format))
        })
        .cloned()
}

pub fn sync(file: &Path) -> Result<(), Error> {
    let configs = read_config_file(file);
    for (name, config) in configs.configs.iter() {
        if config.host.is_some() {
            let raw_ls = run(Command::new("ssh").args([
                &config.host.clone().unwrap(),
                "ls",
                "-At",
                "--full-time",
            ]));
            let existing_files = parse_ls(&raw_ls);
            let last_backup =
                find_most_recent_matching(&existing_files, &config.format).map(|(_, time)| time);
            let now = Utc::now();
            if let Some(last_backup) = last_backup {
                if last_backup > offset_by_interval(now, &config.interval) {
                    println!("Skipping {}", name);
                    continue;
                }
            }

            let filename = format!("{}", now.format(&config.format));
            let destination = Path::new(&config.dir).join(&filename);
            println!("Backing up {} to {}", name, filename);
            let output = if config.host.is_some() {
                Path::new("/tmp/").join(&filename)
            } else {
                destination.clone()
            };
            backup(&config.globs, &Some(output.clone()), &config.gpg_id)?;
            if let Some(host) = &config.host {
                run(Command::new("scp").args([
                    output.into_os_string().to_str().unwrap(),
                    &format!(
                        "{}:{}",
                        &host,
                        destination.into_os_string().to_str().unwrap()
                    ),
                ]));
            }
        }
    }
    Ok(())
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

    #[test]
    fn test_parse_ls() {
        let raw = "
total 0
drwx------ 1 daniel users 12 2023-03-25 21:43:28.131444555 -0400 .config
drwx------ 1 daniel users 30 2023-03-25 21:51:43.614781911 -0400 .ssh
";
        assert_eq!(
            parse_ls(raw),
            vec![
                (
                    ".config".to_string(),
                    DateTime::parse_from_rfc3339("2023-03-26T01:43:28.131444555Z")
                        .unwrap()
                        .with_timezone(&Utc)
                ),
                (
                    ".ssh".to_string(),
                    DateTime::parse_from_rfc3339("2023-03-26T01:51:43.614781911Z")
                        .unwrap()
                        .with_timezone(&Utc)
                )
            ]
        );
    }

    #[test]
    fn test_find_most_recent_matching() {
        let files = [
            (
                "a-01".to_string(),
                Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap(),
            ),
            (
                "b-02".to_string(),
                Utc.with_ymd_and_hms(2000, 1, 2, 0, 0, 0).unwrap(),
            ),
            (
                "b-03".to_string(),
                Utc.with_ymd_and_hms(2000, 1, 3, 0, 0, 0).unwrap(),
            ),
        ];
        assert_eq!(
            find_most_recent_matching(&files, "a-%d"),
            Some(files[0].clone())
        );
        assert_eq!(
            find_most_recent_matching(&files, "b-%d"),
            Some(files[1].clone())
        );
        assert_eq!(find_most_recent_matching(&files, "c-%d"), None);
    }
}
