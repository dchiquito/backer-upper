use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// A configuration for a single backup. A config file can have multiple Configs.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config {
    pub globs: Vec<String>,
    pub gpg_id: Option<String>,
    pub host: Option<String>,
    pub dir: String,
    pub format: String,
    pub interval: String,
    pub copies: Option<usize>,
}

/// A collection of Configs. This is the format used for saving configs to a file.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ConfigCollection {
    #[serde(flatten)]
    pub configs: HashMap<String, Config>,
}

impl ConfigCollection {
    pub fn new() -> ConfigCollection {
        ConfigCollection {
            configs: HashMap::new(),
        }
    }
    pub fn from_config(name: &str, config: Config) -> ConfigCollection {
        let mut config_collection = ConfigCollection::new();
        config_collection.configs.insert(name.to_string(), config);
        config_collection
    }
}

pub fn read_config_file(file: &Path) -> ConfigCollection {
    let contents = std::fs::read_to_string(file).expect("error reading config file");
    toml::from_str(&contents).expect("error deserializing config file")
}

pub fn write_config_file(config: &ConfigCollection, file: &Path) {
    let contents = toml::to_string(config).expect("error serializing config file");
    std::fs::write(file, contents).expect("error writing config file");
}
