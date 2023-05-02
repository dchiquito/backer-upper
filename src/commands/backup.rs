use std::path::PathBuf;
use std::process::Command;

use crate::config::Config;

fn run(command: &mut Command) {
    let output = command.output().unwrap();
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8(output.stderr).unwrap());
        panic!("error running command");
    }
}

pub fn backup(config: &Config) -> Result<(), clap::error::Error> {
    let output_option = config.output.as_ref().map(|pb| {
        pb.clone()
            .into_os_string()
            .into_string()
            .expect("error parsing output string")
    });
    let tar_gz_file = &output_option
        .clone()
        .map(|output| {
            if config.gpg_id.is_none() {
                output
            } else {
                "/tmp/backup.tar.gz".to_string()
            }
        })
        .unwrap_or("/tmp/backup.tar.gz".to_string());
    let output_file: &str = &output_option.unwrap_or(if config.gpg_id.is_some() {
        "/tmp/backup.tar.gz.gpg".to_string()
    } else {
        "/tmp/backup.tar.gz".to_string()
    });
    let files: Vec<PathBuf> = config
        .globs
        .iter()
        .flat_map(|g| glob::glob(g).expect("error parsing glob"))
        .map(Result::unwrap)
        .map(std::fs::canonicalize)
        .map(Result::unwrap)
        .collect();
    run(Command::new("tar")
        .args(["--absolute-names", "-czf", tar_gz_file])
        .args(&files));
    if config.gpg_id.is_some() {
        run(Command::new("gpg").args([
            "--encrypt",
            "--yes",
            "--output",
            output_file,
            "--recipient",
            config.gpg_id.as_ref().unwrap(),
            tar_gz_file,
        ]));
    }
    Ok(())
}
