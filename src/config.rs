use log::LevelFilter;
use serde::{Deserialize, Serialize};
use serde_yml;
use std::collections::HashMap;
use std::{fs, io, result};
use thiserror::Error;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Config {
    #[serde(default)]
    pub aws: AwsConfig,
    #[serde(default)]
    pub files: HashMap<String, Source>,
    #[serde(default = "Config::default_log_level")]
    pub log_level: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
pub struct AwsConfig {
    pub region: Option<String>,
    pub assume_role_arn: Option<String>,
    pub assume_role_external_id: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Source {
    pub source_arn: String,
}

impl Config {
    pub fn from_yaml_str(str: &str) -> Result<Self> {
        let config: Config = serde_yml::from_str(str)?;
        Ok(config)
    }

    pub fn from_file(path: &str) -> Result<Self> {
        let yaml = fs::read_to_string(path)?;
        Config::from_yaml_str(&yaml)
    }

    fn default_log_level() -> String {
        LevelFilter::Info.to_string()
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Yaml(#[from] serde_yml::Error),
}

pub type Result<T> = result::Result<T, Error>;
