use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub globs: Vec<String>,
    pub output: Option<PathBuf>,
    pub key: Option<String>,
}

pub fn read_config_file(file: &Path) -> Config {
    let contents = std::fs::read_to_string(file).expect("error reading config file");
    toml::from_str(&contents).expect("error deserializing config file")
}

pub fn write_config_file(config: &Config, file: &Path) {
    let contents = toml::to_string(config).expect("error serializing config file");
    std::fs::write(file, contents).expect("error writing config file");
}
