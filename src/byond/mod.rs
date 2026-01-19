//! BYOND server communication module.
//!
//! Implements the BYOND topic protocol for communicating with BYOND game servers.

pub mod status;
mod topic;

pub use status::status;

/// Result type for BYOND operations.
type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during BYOND server communication.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("BYOND topic timeout: {0}")]
    Elapsed(#[from] tokio::time::error::Elapsed),
    #[error("BYOND topic IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("BYOND topic int parse error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("BYOND topic float parse error: {0}")]
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
