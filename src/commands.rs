use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::config::Config;
// use sudo::with_env;

// use crate::context::Context;

pub mod backup;
pub mod restore;
pub mod sync;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,

    /// The repository the configuration files are stored in
    #[arg(short, long)]
    pub repo: Option<PathBuf>,

    /// Run the command as root
    #[arg(long)]
    pub root: bool,
}

impl Cli {
    pub fn run_command(&self) -> Result<(), clap::error::Error> {
        // if self.root {
        //     with_env(&["CONFIGURATOR"]).unwrap();
        // }
        // let ctx = Context::new(&self.repo);
        match &self.commands {
            Commands::Backup { globs, output, key } => {
                let config = Config {
                    globs: globs.to_vec(),
                    output: output.clone(),
                    key: key.clone(),
                };
                backup::backup(&config)
            }
            Commands::Restore { file, globs, key } => restore::restore(file, globs, key),
            Commands::Sync { file } => sync::sync(file),
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add/update configuration files to the repository
    Backup {
        /// The configuration file to add/update
        globs: Vec<String>,
        ///
        /// The repository the configuration files are stored in
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(short, long)]
        key: Option<String>,
    },
    Restore {
        file: PathBuf,
        globs: Option<Vec<String>>,
        #[arg(short, long)]
        key: Option<String>,
    },
    Sync {
        file: PathBuf,
    },
}
