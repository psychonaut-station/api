use std::{
    fs::read_to_string,
    net::{IpAddr, SocketAddr},
    ops::Deref,
    sync::Arc,
};

use serde::Deserialize;

#[derive(Clone)]
pub struct Config(Arc<InnerConfig>);

impl Config {
    pub fn read_from_file(path: &str) -> Result<Self, Error> {
        let content = read_to_string(path)?;
        Ok(Self(Arc::new(toml::from_str::<InnerConfig>(&content)?)))
    }
}

impl Deref for Config {
    type Target = InnerConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InnerConfig {
    pub address: IpAddr,
    pub port: u16,
    pub database: DatabaseConfig,
    pub discord: DiscordConfig,
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
pub struct DiscordConfig {
    pub token: String,
    pub guild: i64,
    pub patreon_role: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    pub name: String,
    pub address: SocketAddr,
    pub connection_address: String,
    pub error_message: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to read configuration file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse configuration file: {0}")]
    Toml(#[from] toml::de::Error),
}
