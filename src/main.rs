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
