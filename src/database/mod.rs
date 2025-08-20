mod lookup;
mod player;
mod recent_test_merges;
mod roletime;

pub use lookup::*;
pub use player::*;
pub use recent_test_merges::*;
pub use roletime::*;

use poem_openapi::payload::PlainText;
use sqlx::{Executor as _, MySql, pool::PoolConnection};

type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("internal database error")]
    Sqlx(#[from] sqlx::Error),
    #[error("failed to parse JSON")]
    Json(#[from] serde_json::Error),
    #[error("failed to parse integer")]
    ParseInt(#[from] std::num::ParseIntError),
    //
    #[error("player not found")]
    PlayerNotFound,
}

impl From<Error> for PlainText<String> {
    fn from(val: Error) -> Self {
        PlainText(val.to_string())
    }
}

async fn player_exists(ckey: &str, connection: &mut PoolConnection<MySql>) -> Result<bool> {
    let query = sqlx::query("SELECT 1 FROM player WHERE LOWER(ckey) = ?").bind(ckey.to_lowercase());
    Ok(connection.fetch_optional(query).await?.is_some())
}
