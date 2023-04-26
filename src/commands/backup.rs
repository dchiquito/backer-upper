use std::path::PathBuf;
use std::process::Command;

fn run(command: &mut Command) {
    let output = command.output().unwrap();
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8(output.stderr).unwrap());
        panic!("Error running command");
    }
}

pub fn backup(globs: &[String], output: &Option<PathBuf>) -> Result<(), clap::error::Error> {
    println!("{:?}", globs);
    let output_file = output
        .as_ref()
        .map(|pb| {
            pb.clone()
                .into_os_string()
                .into_string()
                .expect("error parsing output string")
        })
        .unwrap_or("/tmp/backup.tar".to_string());
    let files: Vec<PathBuf> = globs
        .iter()
        .flat_map(|g| glob::glob(g).unwrap())
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
    // TODO serialize backup.yml
    run(Command::new("touch").arg("/tmp/backup.yml"));
    run(Command::new("tar").args([
        "--absolute-names",
        "--transform=s|^.*$|backup.yml|",
        "-rf",
        &output_file,
        "/tmp/backup.yml",
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
