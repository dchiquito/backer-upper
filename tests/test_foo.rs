use std::path::{Path, PathBuf};
use std::process::Command;

use serial_test::serial;

use backer_upper::commands::backup::backup;
use backer_upper::commands::restore::restore;
use backer_upper::config::Config;

fn run(command: &mut Command) {
    let output = command.output().unwrap();
    eprint!("{}", String::from_utf8(output.stderr).unwrap());
    if output.status.code() != Some(0) {
        panic!("error running command {:?}", output.status.code());
    }
}
fn root() -> PathBuf {
    Path::new("/tmp/backer-upper/").into()
}
fn test_file(root: &Path, name: &str) {
    std::fs::write(root.join(name), name).unwrap();
}

static mut GENKEYFILE: Option<PathBuf> = None;

fn setup_test_env() {
    // Locate the genkey file before changing cwd
    let genkeyfile;
    unsafe {
        if GENKEYFILE.is_none() {
            GENKEYFILE = Some(
                std::env::current_dir()
                    .unwrap()
                    .join("tests")
                    .join("genkey"),
            );
        }
        genkeyfile = GENKEYFILE.clone().unwrap();
    }

    // Delete and recreate the test data dir
    let root = &root();
    if root.exists() {
        std::fs::remove_dir_all(root).unwrap();
    }
    std::fs::create_dir_all(root.join("dir")).unwrap();
    std::env::set_current_dir(root).unwrap();
    test_file(root, "a.txt");
    test_file(root, "b.txt");
    test_file(root, "dir/c.txt");
    test_file(root, "dir/d.txt");

    // Set GNUPGHOME to avoid contaminating the system GPG namespace
    let gnupghome = Path::new("/tmp/backer-upper-gpg/");
    if gnupghome.exists() {
        std::fs::remove_dir_all(gnupghome).unwrap();
    }
    std::fs::create_dir_all(gnupghome).unwrap();
    std::env::set_var("GNUPGHOME", "/tmp/backer-upper-gpg/");

    // Set up a new test key
    run(Command::new("gpg").args([
        "--generate-key",
        "--batch",
        genkeyfile.as_os_str().to_str().unwrap(),
    ]));
}
fn sanitize_test_env() {
    let root = &root();
    std::fs::remove_dir_all(root).unwrap();
    std::fs::create_dir_all(root).unwrap();
}
fn assert_files(files: &[&str]) {
    let root = &root();
    for file in files.iter() {
        let path = root.join(file);
        assert!(path.exists(), "{:?} doesn't exist", path)
    }
}
fn assert_no_files(files: &[&str]) {
    let root = &root();
    for file in files.iter() {
        let path = root.join(file);
        assert!(!path.exists(), "{:?} exists", path)
    }
}

#[test]
#[serial]
fn test_backup_restore_glob_star() -> Result<(), clap::error::Error> {
    setup_test_env();
    // backup all files
    let config = Config {
        globs: vec!["*".to_string()],
        output: None,
        gpg_id: None,
    };
    backup(&config)?;
    sanitize_test_env();
    // restore all files
    restore(Path::new("/tmp/backup.tar.gz"), &None, &None)?;
    assert_files(&["a.txt", "b.txt", "dir/c.txt", "dir/d.txt"]);
    Ok(())
}

#[test]
#[serial]
fn test_backup_restore_single_file() -> Result<(), clap::error::Error> {
    setup_test_env();
    // backup a single file
    let config = Config {
        globs: vec!["b.txt".to_string()],
        output: None,
        gpg_id: None,
    };
    backup(&config)?;
    sanitize_test_env();
    // restore all files
    restore(Path::new("/tmp/backup.tar.gz"), &None, &None)?;
    assert_files(&["b.txt"]);
    assert_no_files(&["a.txt", "dir/c.txt", "dir/d.txt"]);
    Ok(())
}

#[test]
#[serial]
fn test_backup_restore_single_file_from_glob_star() -> Result<(), clap::error::Error> {
    setup_test_env();
    // backup all files
    let config = Config {
        globs: vec!["*".to_string()],
        output: None,
        gpg_id: None,
    };
    backup(&config)?;
    sanitize_test_env();
    // restore a single file
    restore(
        Path::new("/tmp/backup.tar.gz"),
        &Some(vec!["/tmp/backer-upper/b.txt".to_string()]),
        &None,
    )?;
    assert_files(&["b.txt"]);
    assert_no_files(&["a.txt", "dir/c.txt", "dir/d.txt"]);
    Ok(())
}

#[test]
#[serial]
fn test_backup_restore_explicit_output() -> Result<(), clap::error::Error> {
    setup_test_env();
    // backup a single file
    let config = Config {
        globs: vec!["dir/c.txt".to_string()],
        output: Some(Path::new("/tmp/backer-upper-test-backup.tar.gz").into()),
        gpg_id: None,
    };
    backup(&config)?;
    sanitize_test_env();
    // restore all files
    restore(
        Path::new("/tmp/backer-upper-test-backup.tar.gz"),
        &None,
        &None,
    )?;
    assert_files(&["dir/c.txt"]);
    assert_no_files(&["a.txt", "b.txt", "dir/d.txt"]);
    Ok(())
}

#[test]
#[serial]
fn test_backup_restore_encrypted() -> Result<(), clap::error::Error> {
    setup_test_env();
    // backup all files
    let config = Config {
        globs: vec!["*".to_string()],
        output: None,
        gpg_id: Some("test@chiquit.ooo".to_string()),
    };
    backup(&config)?;
    sanitize_test_env();
    // restore all files
    restore(Path::new("/tmp/backup.tar.gz.gpg"), &None, &config.gpg_id)?;
    assert_files(&["a.txt", "b.txt", "dir/c.txt", "dir/d.txt"]);
    Ok(())
}

#[test]
#[serial]
fn test_backup_restore_encrypted_with_output() -> Result<(), clap::error::Error> {
    setup_test_env();
    // backup all files
    let config = Config {
        globs: vec!["*".to_string()],
        output: Some(Path::new("/tmp/backer-upper-test-backup.tar.gz.gpg").into()),
        gpg_id: Some("test@chiquit.ooo".to_string()),
    };
    backup(&config)?;
    sanitize_test_env();
    // restore all files
    restore(
        Path::new("/tmp/backer-upper-test-backup.tar.gz.gpg"),
        &None,
        &config.gpg_id,
    )?;
    assert_files(&["a.txt", "b.txt", "dir/c.txt", "dir/d.txt"]);
    Ok(())
}
