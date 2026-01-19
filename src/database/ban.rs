use poem_openapi::Object;
use sqlx::{FromRow, MySqlPool, Row as _, mysql::MySqlRow};

use crate::sqlxext::DateTime;

use super::Result;

/// Represents a ban record of a player.
#[derive(Object)]
pub struct Ban {
    /// The time the ban was issued in YYYY-MM-DD HH:MM:SS format
    pub bantime: String,
    /// The round ID when the ban was issued
    pub round_id: Option<u32>,
    /// The roles affected by the ban, comma-separated
    pub roles: Option<String>,
    /// The expiration time of the ban, if applicable
    /// in YYYY-MM-DD HH:MM:SS format, or null if permanent
    pub expiration_time: Option<String>,
    /// The reason for the ban
    pub reason: String,
    /// The ckey of the banned player
    pub ckey: Option<String>,
    /// The ckey of the admin who issued the ban
    pub a_ckey: String,
    /// Additional edits or notes about the ban
    pub edits: Option<String>,
    /// The datetime when the ban was unbanned, if applicable
    /// in YYYY-MM-DD HH:MM:SS format, or null if still banned
    pub unbanned_datetime: Option<String>,
    /// The ckey of the admin who unbanned the player, if applicable
    /// null if the player is still banned
    pub unbanned_ckey: Option<String>,
}

impl FromRow<'_, MySqlRow> for Ban {
    fn from_row(row: &MySqlRow) -> sqlx::Result<Self> {
        Ok(Ban {
            bantime: row.try_get::<DateTime, _>("bantime")?.to_string(),
            round_id: row.try_get("round_id")?,
            roles: row.try_get("roles")?,
            expiration_time: row
                .try_get::<Option<DateTime>, _>("expiration_time")?
                .map(|d| d.to_string()),
            reason: row.try_get("reason")?,
            ckey: row.try_get("ckey")?,
            a_ckey: row.try_get("a_ckey")?,
            edits: row.try_get("edits")?,
            unbanned_datetime: row
                .try_get::<Option<DateTime>, _>("unbanned_datetime")?
                .map(|d| d.to_string()),
            unbanned_ckey: row.try_get("unbanned_ckey")?,
        })
    }
}

/// Retrieves a ban record by its ID.
///
/// # Arguments
///
/// * `id` - The ID of the ban to retrieve
/// * `pool` - Database connection pool
///
/// # Returns
///
/// An optional `Ban` record if found
pub async fn get_ban_by_id(id: u32, pool: &MySqlPool) -> Result<Option<Ban>> {
    let query = sqlx::query_as(
    	"SELECT id, bantime, round_id, GROUP_CONCAT(role ORDER BY role SEPARATOR ', ') AS roles, expiration_time, reason, ckey, a_ckey, edits, unbanned_datetime, unbanned_ckey FROM ban WHERE id = ? LIMIT 1"
    ).bind(id);

    Ok(query.fetch_optional(pool).await?)
}
