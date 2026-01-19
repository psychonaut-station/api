mod lookup;
mod player;
mod recent_test_merges;
mod roletime;
mod verification;

pub use lookup::*;
pub use player::*;
pub use recent_test_merges::*;
pub use roletime::*;
pub use verification::*;

use poem_openapi::payload::PlainText;
use sqlx::{Executor, MySql};

type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("internal database error")]
    Sqlx(#[from] sqlx::Error),
    #[error("failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to perform HTTP request: {0}")]
    Http(#[from] crate::http::Error),
    #[error("failed to parse integer: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    //
    #[error("player not found")]
    PlayerNotFound,
    #[error("player not linked to Discord account")]
    NotLinked,
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
