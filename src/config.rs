use std::{
    fs::read_to_string,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use serde::Deserialize;

#[allow(dead_code)]
pub type Config = Arc<InnerConfig>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InnerConfig {
    pub address: IpAddr,
    pub port: u16,
    pub database: DatabaseConfig,
    pub servers: Vec<ServerConfig>,
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    pub name: String,
    pub address: SocketAddr,
    pub connection_address: String,
    pub error_message: String,
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
