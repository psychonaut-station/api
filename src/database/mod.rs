mod player;
mod recent_test_merges;

pub use player::*;
pub use recent_test_merges::*;

use poem_openapi::payload::PlainText;
use thiserror::Error;

type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
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
