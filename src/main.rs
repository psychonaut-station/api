//! Psychonaut Station API Server
//!
//! This is the main API server for Psychonaut Station, providing REST endpoints
//! for our website, Discord bot, and other services.

mod byond;
mod cache;
mod config;
mod database;
mod http;
mod route;
mod sqlxext;

use std::{net::SocketAddr, time::Duration};

use poem::{EndpointExt, Server, listener::TcpListener, middleware::AddData};
use sqlx::{
    MySqlPool,
    mysql::{MySqlConnectOptions, MySqlPoolOptions},
};

use crate::{
    cache::Cache,
    config::{Config, DatabaseConfig},
};

/// Main entry point for the API server.
///
/// Initializes error handling, logging, configuration, and starts the HTTP server.
/// The server listens on the address and port specified in the configuration file.
#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tracing::subscriber::set_global_default(tracing_subscriber::fmt().finish())?;

    let config = Config::read_from_file("config.toml")?;

    let socket = SocketAddr::new(config.address, config.port);

    let app = route::route()
        .with(AddData::new(pool(&config.database)))
        .with(AddData::new(Cache::default()))
        .with(AddData::new(config));

    let server = Server::new(TcpListener::bind(socket));

    server.run(app).await?;

    Ok(())
}

/// Creates a connection pool with the specified configuration.
///
/// # Arguments
///
/// * `config` - Database configuration containing connection details
///
/// # Returns
///
/// A configured MySQL connection pool with desired settings
fn pool(config: &DatabaseConfig) -> MySqlPool {
    let connect_options = MySqlConnectOptions::new()
        .username(&config.user)
        .password(&config.password)
        .host(&config.host)
        .port(config.port)
        .database(&config.name);

    let options = MySqlPoolOptions::new()
        .min_connections(5)
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(1))
        .max_lifetime(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(5));

    options.connect_lazy_with(connect_options)
}
