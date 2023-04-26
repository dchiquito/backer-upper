use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::{write_config_file, Config, ConfigCollection};

fn run(command: &mut Command) {
    let output = command.output().unwrap();
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8(output.stderr).unwrap());
        panic!("error running command");
    }
}

pub fn backup(config: &Config) -> Result<(), clap::error::Error> {
    let output_file = config
        .output
        .as_ref()
        .map(|pb| {
            pb.clone()
                .into_os_string()
                .into_string()
                .expect("error parsing output string")
        })
        .unwrap_or("/tmp/backup.tar".to_string());
    let files: Vec<PathBuf> = config
        .globs
        .iter()
        .flat_map(|g| glob::glob(g).expect("error parsing glob"))
        .map(Result::unwrap)
        .map(std::fs::canonicalize)
        .map(Result::unwrap)
        .collect();
    run(Command::new("tar")
        .args([
            "--absolute-names",
            "--transform=s|^|root|",
            "-cf",
            &output_file,
        ])
        .args(&files));
    let configs = ConfigCollection::from_config("backup", config.clone());
    write_config_file(&configs, Path::new("/tmp/backup.toml"));
    run(Command::new("tar").args([
        "--absolute-names",
        "--transform=s|^.*$|backup.toml|",
        "-rf",
        &output_file,
        "/tmp/backup.toml",
    ]));
    run(Command::new("gzip").arg("-f").arg(&output_file));
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_foo() {
        assert!(false);
    }
}
