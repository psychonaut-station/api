mod lookup;
mod player;
mod recent_test_merges;
mod roletime;

pub use lookup::*;
pub use player::*;
pub use recent_test_merges::*;
pub use roletime::*;

use poem_openapi::payload::PlainText;
use sqlx::{Executor, MySql};

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

async fn player_exists(ckey: &str, executor: impl Executor<'_, Database = MySql>) -> Result<bool> {
    let query =
        sqlx::query("SELECT 1 FROM player WHERE LOWER(ckey) = ? LIMIT 1").bind(ckey.to_lowercase());
    Ok(query.fetch_optional(executor).await?.is_some())
}
