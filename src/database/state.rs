use std::time::Duration;

use sqlx::{
    mysql::{MySqlConnectOptions, MySqlPoolOptions},
    MySqlPool,
};

use crate::config;

pub struct Database {
    pub pool: MySqlPool,
}

impl Database {
    pub fn new(config: &config::Database) -> Result<Self, sqlx::Error> {
        let options = MySqlPoolOptions::new()
            .min_connections(5)
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(1))
            .max_lifetime(Duration::from_secs(3))
            .idle_timeout(Duration::from_secs(5));

        let connect_options = MySqlConnectOptions::new()
            .username(&config.user)
            .password(&config.password)
            .database(&config.game_database);

        let connect_options = match &config.socket {
            Some(socket) => connect_options.socket(socket),
            None => connect_options
                .host(&config.host.to_string())
                .port(config.port),
        };

        let pool = options.connect_lazy_with(connect_options);

        Ok(Self { pool })
    }
}
