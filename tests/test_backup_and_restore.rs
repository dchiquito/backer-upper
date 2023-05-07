use std::path::{Path, PathBuf};
use std::process::Command;

use serial_test::serial;

use backer_upper::commands::backup::backup;
use backer_upper::commands::restore::restore;
use backer_upper::commands::sync::sync_config;
use backer_upper::config::Config;
use backer_upper::utils::run;

fn root() -> PathBuf {
    Path::new("/tmp/backer-upper/").into()
}
fn test_file(root: &Path, name: &str) {
    std::fs::write(root.join(name), name).unwrap();
}

static mut GENKEYFILE: Option<PathBuf> = None;

fn setup_test_env() {
    // Enable logging
    #![allow(unused_must_use)]
    env_logger::try_init();
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
    backup(&["*".to_string()], Path::new("/tmp/backup.tar.gz"), &None)?;
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
    backup(
        &["b.txt".to_string()],
        Path::new("/tmp/backup.tar.gz"),
        &None,
    )?;
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
    backup(&["*".to_string()], Path::new("/tmp/backup.tar.gz"), &None)?;
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
    backup(
        &["dir/c.txt".to_string()],
        Path::new("/tmp/backer-upper-test-backup.tar.gz"),
        &None,
    )?;
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
    backup(
        &["*".to_string()],
        Path::new("/tmp/backup.tar.gz.gpg"),
        &Some("test@chiquit.ooo".to_string()),
    )?;
    sanitize_test_env();
    // restore all files
    restore(
        Path::new("/tmp/backup.tar.gz.gpg"),
        &None,
        &Some("test@chiquit.ooo".to_string()),
    )?;
    assert_files(&["a.txt", "b.txt", "dir/c.txt", "dir/d.txt"]);
    Ok(())
}

#[test]
#[serial]
fn test_backup_restore_encrypted_with_output() -> Result<(), clap::error::Error> {
    setup_test_env();
    // backup all files
    backup(
        &["*".to_string()],
        Path::new("/tmp/backer-upper-test-backup.tar.gz.gpg"),
        &Some("test@chiquit.ooo".to_string()),
    )?;
    sanitize_test_env();
    // restore all files
    restore(
        Path::new("/tmp/backer-upper-test-backup.tar.gz.gpg"),
        &None,
        &Some("test@chiquit.ooo".to_string()),
    )?;
    assert_files(&["a.txt", "b.txt", "dir/c.txt", "dir/d.txt"]);
    Ok(())
}

#[test]
#[serial]
fn test_sync_and_restore() -> Result<(), clap::error::Error> {
    setup_test_env();
    std::fs::create_dir_all("/tmp/backer-upper-sync/").unwrap();
    let backup = sync_config(
        "test",
        &Config {
            globs: vec!["/tmp/backer-upper/*".to_string()],
            gpg_id: Some("test@chiquit.ooo".to_string()),
            host: None,
            dir: "/tmp/backer-upper-sync/".to_string(),
            format: "test_sync_%Y-%m-%d_%H:%M:%S.tar.gz.gpg".to_string(),
            interval: "1 second".to_string(),
            copies: None,
        },
    )?
    .unwrap();
    sanitize_test_env();
    restore(&backup, &None, &Some("test@chiquit.ooo".to_string()))?;
    assert_files(&["a.txt", "b.txt", "dir/c.txt", "dir/d.txt"]);
    Ok(())
}

#[test]
#[serial]
fn test_sync_redundant() -> Result<(), clap::error::Error> {
    setup_test_env();
    std::fs::create_dir_all("/tmp/backer-upper-sync/").unwrap();
    let config = Config {
        globs: vec!["/tmp/backer-upper/*".to_string()],
        gpg_id: Some("test@chiquit.ooo".to_string()),
        host: None,
        dir: "/tmp/backer-upper-sync/".to_string(),
        format: "test_redundant_sync_%Y-%m-%d_%H:%M:%S.tar.gz.gpg".to_string(),
        // This test will fail if run multiple times within two seconds
        interval: "2 seconds".to_string(),
        copies: None,
    };
    let backup = sync_config("test", &config)?.unwrap();
    assert!(backup.exists());
    // Sync again, this one shouldn't need a new backup
    assert_eq!(sync_config("test", &config)?, None);
    Ok(())
}

#[test]
#[serial]
fn test_sync_copies() -> Result<(), clap::error::Error> {
    setup_test_env();
    std::fs::create_dir_all("/tmp/backer-upper-sync/").unwrap();
    let config = Config {
        globs: vec!["/tmp/backer-upper/*".to_string()],
        gpg_id: Some("test@chiquit.ooo".to_string()),
        host: None,
        dir: "/tmp/backer-upper-sync/".to_string(),
        format: "test_sync_copies_%Y-%m-%d_%H:%M:%S.tar.gz.gpg".to_string(),
        // Always run
        interval: "0 seconds".to_string(),
        copies: Some(1),
    };
    let backup_1 = sync_config("test", &config)?.unwrap();
    assert!(backup_1.exists());
    let backup_2 = sync_config("test", &config)?.unwrap();
    assert!(backup_2.exists());
    // The second backup should have cleaned up the first
    assert!(!backup_1.exists());
    Ok(())
}
