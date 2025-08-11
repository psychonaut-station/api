mod byond;
mod cache;
mod database;
mod route;
mod sqlxext;

use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

use poem::{EndpointExt, Server, listener::TcpListener, middleware::AddData};
use sqlx::{MySqlPool, mysql::MySqlPoolOptions};
use urlencoding::encode;

use crate::cache::Cache;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = tracing_subscriber::fmt().finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let db_url = format!(
        "mysql://{db_user}:{}@{db_host}:{db_port}/{db_name}",
        encode(db_pass)
    );

    let app = route::route()
        .with(AddData::new(pool(&db_url)))
        .with(AddData::new(Cache::default()));

    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 3000);
    let server = Server::new(TcpListener::bind(socket));

    server.run(app).await?;

    Ok(())
}

fn pool(url: &str) -> MySqlPool {
    let options = MySqlPoolOptions::new()
        .min_connections(5)
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(1))
        .max_lifetime(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(5));

    options.connect_lazy(url).unwrap()
}
