use rocket::{catch, catchers, http::Status, Config as RocketConfig, Request};
use thiserror::Error;
use tracing::info;

use crate::{
    config::Config,
    cors::cors,
    database::{Database, ServerDatabase}};

mod byond;
mod config;
mod cors;
mod database;
mod http;
mod routes;
mod serde;

#[rocket::main]
#[allow(clippy::result_large_err)]
async fn main() -> Result<(), Error> {
    let subscriber = tracing_subscriber::fmt().finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let config = Config::read_from_file()?;
    let database = Database::new(&config.database, &config.database.database)?;
    let server_db = ServerDatabase(Database::new(
        &config.database,
        &config.database.server_database
    )?);

    info!(
        "Server has launched from http://{}:{}",
        config.address, config.port
    );

    let provider = RocketConfig {
        address: config.address,
        port: config.port,
        cli_colors: config.cli_colors,
        log_level: config.log_level,
        ..Default::default()
    };

    let rocket = rocket::custom(provider)
        .attach(cors()?)
        .manage(config)
        .manage(database)
        .manage(server_db)
        .register("/", catchers![empty_catcher]);

    let rocket = routes::mount(rocket);

    rocket.launch().await?;

    Ok(())
}

#[catch(default)]
fn empty_catcher(_: Status, _: &Request) {}

#[derive(Debug, Error)]
#[error(transparent)]
enum Error {
    Config(#[from] config::Error),
    Cors(#[from] rocket_cors::Error),
    Rocket(#[from] rocket::Error),
    Sqlx(#[from] sqlx::Error),
    SetGlobalDefault(#[from] tracing::subscriber::SetGlobalDefaultError),
}
