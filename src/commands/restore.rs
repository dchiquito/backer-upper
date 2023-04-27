use std::path::Path;
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
    files: &Option<Vec<String>>,
    key: &Option<String>,
) -> Result<(), Error> {
    let files = files.clone().unwrap_or(vec![]);
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
