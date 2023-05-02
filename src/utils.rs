use std::process::Command;

pub fn run(command: &mut Command) -> String {
    // println!("Running {:?}", command);
    let output = command.output().unwrap();
    eprint!("{}", String::from_utf8(output.stderr).unwrap());
    if output.status.code() != Some(0) {
        panic!("error running command {:?}", output.status.code());
    }
    String::from_utf8(output.stdout).unwrap()
}
