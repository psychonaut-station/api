pub mod status;
mod topic;

pub use status::status;

type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("BYOND topic timeout")]
    Elapsed(#[from] tokio::time::error::Elapsed),
    #[error("BYOND topic IO error")]
    Io(#[from] std::io::Error),
    #[error("BYOND topic int parse error")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("BYOND topic float parse error")]
    ParseFloat(#[from] std::num::ParseFloatError),

    #[error("BYOND topic invalid response")]
    InvalidResponse,
    #[error("BYOND topic unexpected response: {0:?}")]
    UnexpectedResponse(topic::Response),

    #[error("Unknown game state: {0}")]
    GameStateConversion(String),
    #[error("Unknown security level: {0}")]
    SecurityLevelConversion(String),
    #[error("Unknown shuttle mode: {0}")]
    ShuttleModeConversion(String),
}
