use std::path::Path;
use std::process::Command;

use clap::error::Error;

fn run(command: &mut Command) {
    let output = command.output().unwrap();
    if output.status.code() != Some(0) {
        eprint!("{}", String::from_utf8(output.stderr).unwrap());
        panic!("error running command");
    }
}

pub fn restore(
    backup: &Path,
    files: &Option<Vec<String>>,
    gpg_id: &Option<String>,
) -> Result<(), Error> {
    let mut backup_file = backup
        .as_os_str()
        .to_str()
        .expect("error parsing path to backup file");
    if gpg_id.is_some() {
        run(Command::new("gpg").args([
            "--decrypt",
            "--yes",
            "--output",
            "/tmp/backup.tar.gz",
            backup_file,
        ]));
        backup_file = "/tmp/backup.tar.gz";
    }
    let files = files.clone().unwrap_or(vec![]);
    run(Command::new("tar")
        .args(["--absolute-names", "-xzf", backup_file])
        .args(&files));
    Ok(())
}
