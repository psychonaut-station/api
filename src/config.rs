//! Configuration management module.
//!
//! Handles loading and parsing of the application configuration from TOML files.
//! Includes settings for required services such as the database and Discord API.

use std::{
    fs::read_to_string,
    net::{IpAddr, SocketAddr},
    ops::Deref,
    sync::Arc,
};

use serde::Deserialize;

/// Application configuration wrapper.
///
/// This is a thread-safe wrapper around the internal configuration that can be cloned
/// and shared across multiple tasks.
#[derive(Clone)]
pub struct Config(Arc<InnerConfig>);

impl Config {
    /// Reads and parses the configuration from a TOML file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or if the TOML is invalid
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

/// Internal configuration structure containing all application settings.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InnerConfig {
    /// IP address to bind the server to
    pub address: IpAddr,
    /// Port number to listen on
    pub port: u16,
    /// Database connection configuration
    pub database: DatabaseConfig,
    /// Discord API configuration
    pub discord: DiscordConfig,
    /// List of game servers to monitor
    pub servers: Vec<ServerConfig>,
}

/// Database connection configuration.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatabaseConfig {
    /// Database username
    pub user: String,
    /// Database password
    pub password: String,
    /// Database host address
    pub host: String,
    /// Database port
    pub port: u16,
    /// Database name
    pub name: String,
}

/// Discord API configuration.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscordConfig {
    /// Discord bot token for API authentication
    pub token: String,
    /// Discord guild (server) ID to monitor
    pub guild: i64,
    /// Discord role ID for Patreon supporters
    pub patreon_role: i64,
}

/// Game server configuration.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    /// Display name of the server
    pub name: String,
    /// Internal socket address for queries
    pub address: SocketAddr,
    /// Public connection address for players
    pub connection_address: String,
    /// Error message to display when server is unavailable
    pub error_message: String,
}

/// Configuration-related errors.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to read configuration file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse configuration file: {0}")]
    Toml(#[from] toml::de::Error),
}
