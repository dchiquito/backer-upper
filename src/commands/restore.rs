use std::path::{Path, PathBuf};
use std::process::Command;

use clap::error::Error;

fn run(command: &mut Command) {
    let output = command.output().unwrap();
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8(output.stderr).unwrap());
        panic!("error running command");
    }
}

pub fn restore(
    backup: &Path,
    globs: &Option<Vec<String>>,
    key: &Option<String>,
) -> Result<(), Error> {
    let files: Vec<PathBuf> = globs
        .clone()
        .map(|globs| {
            globs
                .iter()
                .flat_map(|g| glob::glob(g).expect("error parsing glob"))
                .map(Result::unwrap)
                .map(std::fs::canonicalize)
                .map(Result::unwrap)
                .collect()
        })
        .unwrap_or(vec![]);
    run(Command::new("tar")
        .args([
            "--absolute-names",
            "-xzf",
            backup
                .as_os_str()
                .to_str()
                .expect("error parsing path to backup file"),
        ])
        .args(&files));
    Ok(())
}
