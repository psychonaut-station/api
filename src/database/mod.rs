//! Database access layer.
//!
//! Provides various functions for querying the database.

pub mod ban;
mod lookup;
mod player;
mod recent_test_merges;
mod roletime;
mod verify;

pub use ban::*;
pub use lookup::*;
pub use player::*;
pub use recent_test_merges::*;
pub use roletime::*;
pub use verify::*;

use poem_openapi::payload::PlainText;
use sqlx::{Executor, MySql, Row as _};

/// Result type for database operations.
type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during database operations.
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
    // verification errors
    #[error("Discord account already linked to ckey: {0}")]
    DiscordInUse(String),
    #[error("invalid or expired one-time token")]
    TokenInvalid,
}

impl From<Error> for PlainText<String> {
    fn from(val: Error) -> Self {
        PlainText(val.to_string())
    }
}

/// Checks if a player exists in the database.
///
/// # Arguments
///
/// * `ckey` - Player's ckey (case-insensitive).
/// * `executor` - Database executor or connection.
///
/// # Returns
///
/// `true` if the player exists, `false` otherwise.
async fn player_exists(ckey: &str, executor: impl Executor<'_, Database = MySql>) -> Result<bool> {
    let query =
        sqlx::query("SELECT 1 FROM player WHERE LOWER(ckey) = ? LIMIT 1").bind(ckey.to_lowercase());
    Ok(query.fetch_optional(executor).await?.is_some())
}

/// Retrieves a player's linked Discord ID from the database.
///
/// # Arguments
///
/// * `ckey` - Player's ckey (case-insensitive).
/// * `pool` - Database connection pool.
///
/// # Returns
///
/// `Some(id)` if the player has a valid link, `None` otherwise.
pub async fn user_id_from_ckey(
    ckey: &str,
    pool: impl Executor<'_, Database = MySql>,
) -> Result<Option<i64>> {
    let query = sqlx::query(
        "SELECT discord_id FROM discord_links WHERE LOWER(ckey) = ? AND valid = 1 LIMIT 1",
    )
    .bind(ckey.to_lowercase());

    match query.fetch_optional(pool).await? {
        Some(row) => Ok(Some(row.try_get("discord_id")?)),
        None => Ok(None),
    }
}

/// Retrieves a player's ckey from their Discord ID.
///
/// # Arguments
///
/// * `id` - Discord user ID to look up
/// * `pool` - Database connection pool
///
/// # Returns
///
/// `Some(ckey)` if the Discord account has a valid link, `None` otherwise.
pub async fn ckey_from_user_id(
    id: i64,
    pool: impl Executor<'_, Database = MySql>,
) -> Result<Option<String>> {
    let query =
        sqlx::query("SELECT ckey FROM discord_links WHERE discord_id = ? AND valid = 1 LIMIT 1")
            .bind(id);

    match query.fetch_optional(pool).await? {
        Some(row) => Ok(Some(row.try_get("ckey")?)),
        None => Ok(None),
    }
}
