use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::{DateTime, Days, Duration, Months, TimeZone, Utc};
use clap::error::Error;
use log::{debug, trace};
use regex::Regex;

use crate::commands::backup::backup;
use crate::config::{read_config_file, Config};
use crate::utils::run;

/// Offset a DateTime<T> by a specially formatted interval string.
///
/// The interval might require use of Months or Days, so we cannot simply return a Duration.
fn offset_by_interval<T: TimeZone>(now: DateTime<T>, interval: &str) -> DateTime<T> {
    let pattern = Regex::new(r"^([0-9]+)\W*([Mwdhms]|month|months|week|weeks|day|days|hour|hours|minute|minutes|second|seconds)$").unwrap();
    let captures = pattern
        .captures(interval)
        .unwrap_or_else(|| panic!("invalid date string {}", interval));
    let count: u32 = captures[1].parse().unwrap();
    let unit: &str = &captures[2];
    let _now = now.clone(); // hack to sneak the value around the borrow
    let offset = match unit {
        "M" | "month" | "months" => now - Months::new(count),
        "w" | "week" | "weeks" => now - Duration::weeks(count.into()),
        "d" | "day" | "days" => now - Days::new(count.into()),
        "h" | "hour" | "hours" => now - Duration::hours(count.into()),
        "m" | "minute" | "minutes" => now - Duration::minutes(count.into()),
        "s" | "second" | "seconds" => now - Duration::seconds(count.into()),
        _ => panic!("invalid time unit {}", unit),
    };
    debug!(
        "Offset {:?} by {} to get {:?}",
        &_now.clone(),
        &(_now - offset.clone()),
        &offset
    );
    offset
}

/// Parse the output of the `ls -At --full-time` command into file names and last modified times.
fn parse_ls(raw: &str) -> Vec<(String, DateTime<Utc>)> {
    let pattern = Regex::new(r"[drwx\-]{10} [0-9]+\W+\w+\W+\w+\W+[0-9]+ ([0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}.[0-9]+ [+\-][0-9]{4}) (.+)").unwrap();
    pattern
        .captures_iter(raw)
        .map(|captures| {
            (
                captures[2].to_string(),
                DateTime::parse_from_str(&captures[1], "%Y-%m-%d %H:%M:%S.%f %z")
                    .unwrap()
                    .with_timezone(&Utc),
            )
        })
        .collect()
}

/// Test if a formatted file name could plausibly have been produced by the given format string and
/// creation time. Because zipping and uploading the archive can take some time, all times up to an
/// hour before the creation date are tested.
fn time_matches(name: &str, time: &DateTime<Utc>, format: &str) -> bool {
    trace!(
        "Testing if {} matches expected {} within an hour",
        name,
        time.format(format)
    );
    let mut attempt = *time;
    while attempt > *time - Duration::hours(1) {
        if name == format!("{}", attempt.format(format)) {
            return true;
        }
        attempt -= Duration::seconds(1);
    }
    trace!("No match found in the last hour");
    false
}

/// Find the most recent file matching a format string. Assumes that the files are already in
/// sorted order.
fn find_most_recent_matching(
    files: &[(String, DateTime<Utc>)],
    format: &str,
) -> Option<(String, DateTime<Utc>)> {
    files
        .iter()
        .find(|(name, time)| time_matches(name, time, format))
        .cloned()
}

/// Filter out any files that were not plausibly generated by this sync process.
fn filter_backups(files: &[(String, DateTime<Utc>)], format: &str) -> Vec<(String, DateTime<Utc>)> {
    files
        .iter()
        .filter(|(name, time)| time_matches(name, time, format))
        .cloned()
        .collect()
}

pub fn sync_config(name: &str, config: &Config) -> Result<Option<PathBuf>, Error> {
    debug!("Syncing config {}: {:?}", name, config);

    // Check for existing backups, remotely if necessary
    let raw_ls;
    if let Some(host) = &config.host {
        raw_ls = run(Command::new("ssh").args([host, "ls", "-At", "--full-time", &config.dir]));
    } else {
        raw_ls = run(Command::new("ls").args(["-At", "--full-time", &config.dir]))
    }
    let existing_files = parse_ls(&raw_ls);

    // Skip this backup if the last backup was too recent
    let last_backup =
        find_most_recent_matching(&existing_files, &config.format).map(|(_, time)| time);
    let now = Utc::now();
    if let Some(last_backup) = last_backup {
        debug!("Last backup was at {}", last_backup);
        if last_backup > offset_by_interval(now, &config.interval) {
            debug!("Skipping backup");
            return Ok(None);
        }
    }

    let filename = format!("{}", now.format(&config.format));
    let destination = Path::new(&config.dir).join(&filename);
    // Use a temporary file if we are uploading, otherwise use the destination
    let output = if config.host.is_some() {
        Path::new("/tmp/").join(&filename)
    } else {
        destination.clone()
    };

    // Run the backup
    backup(&config.globs, &output, &config.gpg_id)?;

    // Copy the archive if there is a host specified
    if let Some(host) = &config.host {
        run(Command::new("scp").args([
            output.into_os_string().to_str().unwrap(),
            &format!(
                "{}:{}",
                &host,
                destination.clone().into_os_string().to_str().unwrap()
            ),
        ]));
    }

    // Delete redundant copies if necessary
    if let Some(copies) = config.copies {
        let files_to_remove: Vec<String> = filter_backups(&existing_files, &config.format)
            .iter()
            .skip(copies - 1)
            .map(|(name, _)| {
                Path::new(&config.dir)
                    .join(name)
                    .into_os_string()
                    .into_string()
                    .unwrap()
            })
            .collect();
        if !files_to_remove.is_empty() {
            debug!("Removing redundant backups {:?}", files_to_remove);
            if let Some(host) = &config.host {
                run(Command::new("ssh")
                    .args([host, "rm"])
                    .args(&files_to_remove));
            } else {
                run(Command::new("rm").args(&files_to_remove));
            }
        }
    }
    Ok(Some(destination))
}

pub fn sync(file: &Path) -> Result<(), Error> {
    debug!("Syncing file {:?}", file);
    let configs = read_config_file(file);
    for (name, config) in configs.configs.iter() {
        sync_config(name, config)?;
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
