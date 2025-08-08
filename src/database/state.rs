use std::time::Duration;

use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use urlencoding::encode;

use crate::config;

pub struct Database {
    pub pool: MySqlPool,
}

pub struct ServerDatabase(pub Database);

impl Database {
    pub fn new(
        config: &config::Database,
        database: &str,
    ) -> Result<Self, sqlx::Error> {
        let options = MySqlPoolOptions::new()
            .min_connections(5)
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(1))
            .max_lifetime(Duration::from_secs(3))
            .idle_timeout(Duration::from_secs(5));

        let url = format!(
            "mysql://{}:{}@{}:{}/{}",
            config.user,
            encode(&config.password),
            config.host,
            config.port,
            database
        );

        let pool = options.connect_lazy(&url)?;

        Ok(Self { pool })
    }
}
