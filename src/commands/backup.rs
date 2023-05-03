use std::path::{Path, PathBuf};
use std::process::Command;

use crate::utils::run;

pub fn backup(
    globs: &[String],
    output: &Path,
    gpg_id: &Option<String>,
) -> Result<(), clap::error::Error> {
    let output = output.to_path_buf().into_os_string().into_string().unwrap();
    let tar_gz_file = if gpg_id.is_none() {
        output.clone()
    } else {
        "/tmp/backup.tar.gz".to_string()
    };
    let files: Vec<PathBuf> = globs
        .iter()
        .flat_map(|g| glob::glob(g).expect("error parsing glob"))
        .map(Result::unwrap)
        .map(std::fs::canonicalize)
        .map(Result::unwrap)
        .collect();
    run(Command::new("tar")
        .args(["--absolute-names", "-czf", &tar_gz_file])
        .args(&files));
    if gpg_id.is_some() {
        run(Command::new("gpg").args([
            "--encrypt",
            "--yes",
            "--output",
            &output,
            "--recipient",
            gpg_id.as_ref().unwrap(),
            &tar_gz_file,
        ]));
    }
    Ok(())
}
