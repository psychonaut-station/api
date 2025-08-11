pub mod player;

use poem_openapi::payload::PlainText;
use thiserror::Error;

type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("internal database error")]
    Sqlx(#[from] sqlx::Error),
    //
    #[error("player not found")]
    PlayerNotFound,
}

impl From<Error> for PlainText<String> {
    fn from(val: Error) -> Self {
        PlainText(val.to_string())
    }
}
