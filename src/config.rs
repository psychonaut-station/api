use std::{fs::read_to_string, net::IpAddr, sync::Arc};

use serde::Deserialize;

#[allow(dead_code)]
pub type Config = Arc<InnerConfig>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InnerConfig {
    pub address: IpAddr,
    pub port: u16,
    pub database: DatabaseConfig,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatabaseConfig {
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub name: String,
}

impl InnerConfig {
    pub fn read_from_file(path: &str) -> Result<Self, Error> {
        let content = read_to_string(path)?;
        Ok(toml::from_str::<Self>(&content)?)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to read configuration file")]
    Io(#[from] std::io::Error),
    #[error("failed to parse configuration file")]
    Toml(#[from] toml::de::Error),
}
