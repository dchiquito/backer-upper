use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod backup;
pub mod restore;
pub mod sync;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,
}

impl Cli {
    pub fn run_command(&self) -> Result<(), clap::error::Error> {
        match &self.commands {
            Commands::Backup {
                globs,
                output,
                gpg_id,
            } => backup::backup(globs, output, gpg_id),
            Commands::Restore {
                file,
                globs,
                gpg_id,
            } => restore::restore(file, globs, gpg_id),
            Commands::Sync { file } => sync::sync(file),
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Back up files.
    Backup {
        /// The files to back up.
        globs: Vec<String>,
        /// The destination file. Defaults to /tmp/backup.tar.gz.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Optional. The id of the GPG key to use for encryption.
        #[arg(short, long)]
        gpg_id: Option<String>,
    },
    /// Restore files from a backup.
    Restore {
        /// The archive to restore from.
        file: PathBuf,
        /// Optional. Specific files within the archive to restore.
        globs: Option<Vec<String>>,
        /// Optional. The id of the GPG key used to encrypt the archive.
        #[arg(short, long)]
        gpg_id: Option<String>,
    },
    /// Synchronize any number of backups according to a schedule.
    ///
    /// Consult the README for information on the file format.
    Sync {
        /// The TOML file describing the backups.
        file: PathBuf,
    },
}
