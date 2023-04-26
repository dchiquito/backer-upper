use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub globs: Vec<String>,
    pub output: Option<PathBuf>,
    pub key: Option<String>,
}

pub fn read_config_file(file: &Path) -> Config {
    let reader = std::fs::File::open(file).expect("error reading config file");
    serde_yaml::from_reader(reader).expect("error parsing config file")
}

pub fn write_config_file(config: &Config, file: &Path) {
    let writer = std::fs::File::open(file).expect("error opening config file");
    serde_yaml::to_writer(writer, config).expect("error writer config file")
}
