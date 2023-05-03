use log::{debug, error};
use std::process::Command;

pub fn run(command: &mut Command) -> String {
    debug!("Running {:?}", command);
    let output = command.output().unwrap();
    let err = String::from_utf8(output.stderr).unwrap();
    if !err.is_empty() {
        error!("{}", err);
    }
    if output.status.code() != Some(0) {
        panic!("error running command {:?}", output.status.code());
    }
    String::from_utf8(output.stdout).unwrap()
}
