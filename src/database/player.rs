//! Player information queries.
//!
//! Handles queries for player data.

use const_format::formatcp as const_format;
use poem_openapi::{Enum, Object};
use sqlx::{FromRow, MySqlPool, Row as _, mysql::MySqlRow};

use crate::sqlxext::{Date, DateTime};

use super::{Error, Result, player_exists};

/// Represents a player's basic information.
#[derive(Object)]
pub struct Player {
    /// The player's ckey
    pub ckey: String,
    /// The player's BYOND key, if available
    pub byond_key: Option<String>,
    /// The date and time when the player first joined the game
    /// in YYYY-MM-DD HH:MM:SS format
    pub first_seen: String,
    /// The date and time when the player was last seen
    /// in YYYY-MM-DD HH:MM:SS format
    pub last_seen: String,
    /// The round ID when the player first joined
    pub first_seen_round: Option<u32>,
    /// The round ID when the player was last seen
    pub last_seen_round: Option<u32>,
    /// The player's BYOND account age, if available
    /// in YYYY-MM-DD format
    pub byond_age: Option<String>,
}

impl FromRow<'_, MySqlRow> for Player {
    fn from_row(row: &MySqlRow) -> sqlx::Result<Self> {
        Ok(Player {
            ckey: row.try_get("ckey")?,
            byond_key: row.try_get("byond_key")?,
            first_seen: row.try_get::<DateTime, _>("firstseen")?.to_string(),
            last_seen: row.try_get::<DateTime, _>("lastseen")?.to_string(),
            first_seen_round: row.try_get("firstseen_round_id")?,
            last_seen_round: row.try_get("lastseen_round_id")?,
            byond_age: row
                .try_get::<Option<Date>, _>("accountjoindate")?
                .map(|d| d.to_string()),
        })
    }
}

/// Retrieves a player's basic information from the database.
///
/// # Arguments
///
/// * `ckey` - Player's ckey (case-insensitive)
/// * `pool` - Database connection pool
///
/// # Returns
///
/// The player's information
///
/// # Errors
///
/// Returns `Error::PlayerNotFound` if the player doesn't exist
pub async fn get_player(ckey: &str, pool: &MySqlPool) -> Result<Player> {
    let query = sqlx::query_as(
        "SELECT ckey, byond_key, firstseen, firstseen_round_id, lastseen, lastseen_round_id, INET_NTOA(ip), computerid, accountjoindate FROM player WHERE LOWER(ckey) = ? LIMIT 1"
    )
    .bind(ckey.to_lowercase());

    let player = query.fetch_optional(pool).await?;

    player.ok_or(Error::PlayerNotFound)
}

/// Represents an achievement in the game.
#[derive(Object)]
pub struct Achievement {
    /// The unique key of the achievement
    pub achievement_key: String,
    /// The version of the achievement
    pub achievement_version: u16,
    /// The type of the achievement
    pub achievement_type: Option<AchievementType>,
    /// The name of the achievement
    pub achievement_name: Option<String>,
    /// The description of the achievement
    pub achievement_description: Option<String>,
    /// If achievement type is Score, this is the score value such as the number of times Bubblegum has been killed;
    /// If achievement type is Achievement, this is always 1
    pub value: Option<i32>,
    /// If achievement type is Score, this is the timestamp when the score was last updated;
    /// If achievement type is Achievement, this is the timestamp when the achievement was unlocked
    /// both in YYYY-MM-DD HH:MM:SS format
    pub timestamp: String,
}

impl FromRow<'_, MySqlRow> for Achievement {
    fn from_row(row: &MySqlRow) -> sqlx::Result<Self> {
        Ok(Achievement {
            achievement_key: row.try_get("achievement_key")?,
            achievement_version: row.try_get("achievement_version")?,
            achievement_type: row
                .try_get::<Option<String>, _>("achievement_type")?
                .map(Into::into),
            achievement_name: row.try_get("achievement_name")?,
            achievement_description: row.try_get("achievement_description")?,
            value: row.try_get("value")?,
            timestamp: row.try_get::<DateTime, _>("last_updated")?.to_string(),
        })
    }
}

/// Possible types of achievements.
#[derive(Enum)]
#[oai(rename_all = "lowercase")]
pub enum AchievementType {
    /// Represents a simple achievement that can be unlocked
    Achievement,
    /// Represents a score-based achievement
    Score,
    /// Base abstract type for achievements
    Award,
}

impl From<String> for AchievementType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "achievement" => AchievementType::Achievement,
            "score" => AchievementType::Score,
            _ => AchievementType::Award,
        }
    }
}

/// Retrieves all achievements for a player, optionally filtered by type.
///
/// # Arguments
///
/// * `ckey` - Player's ckey (case-insensitive)
/// * `achievement_type` - Optional filter for achievement type ("achievement", "score", etc.)
/// * `pool` - Database connection pool
///
/// # Returns
///
/// A list of achievements, ordered by most recent first
///
/// # Errors
///
/// Returns `Error::PlayerNotFound` if the player doesn't exist
pub async fn get_player_achievements(
    ckey: &str,
    achievement_type: &Option<String>,
    pool: &MySqlPool,
) -> Result<Vec<Achievement>> {
    let mut sql = "SELECT m.achievement_key, m.achievement_version, m.achievement_type, m.achievement_name, m.achievement_description, a.value, a.last_updated FROM achievements a JOIN achievement_metadata m ON a.achievement_key = m.achievement_key WHERE LOWER(a.ckey) = ?".to_string();

    if achievement_type.is_some() {
        sql.push_str(" AND m.achievement_type = ?");
    }

    sql.push_str(" ORDER BY a.last_updated DESC");

    let mut query = sqlx::query_as(&sql).bind(ckey.to_lowercase());

    if let Some(achievement_type) = achievement_type {
        query = query.bind(achievement_type);
    }

    let achievements = query.fetch_all(pool).await?;

    if achievements.is_empty() && !player_exists(ckey, pool).await? {
        return Err(Error::PlayerNotFound);
    }

    Ok(achievements)
}

/// Represents a ban record for a player.
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

/// Retrieves all bans for a player with optional filters.
///
/// # Arguments
///
/// * `ckey` - Player's ckey (case-insensitive)
/// * `permanent` - If true, only return permanent bans (no expiration time)
/// * `since` - Optional date filter to only return bans issued after this date
/// * `pool` - Database connection pool
///
/// # Returns
///
/// A list of bans grouped by ban time
///
/// # Errors
///
/// Returns `Error::PlayerNotFound` if the player doesn't exist
pub async fn get_player_bans(
    ckey: &str,
    permanent: bool,
    since: &Option<String>,
    pool: &MySqlPool,
) -> Result<Vec<Ban>> {
    let mut sql = "SELECT id, bantime, round_id, GROUP_CONCAT(role ORDER BY role SEPARATOR ', ') AS roles, expiration_time, reason, ckey, a_ckey, edits, unbanned_datetime, unbanned_ckey FROM ban WHERE LOWER(ckey) = ?".to_string();

    if permanent {
        sql.push_str(" AND expiration_time IS NULL");
    }

    if since.is_some() {
        sql.push_str(" AND bantime > ?");
    }

    sql.push_str(" GROUP BY bantime");

    let mut query = sqlx::query_as(&sql).bind(ckey.to_lowercase());

    if let Some(since) = since {
        query = query.bind(since);
    }

    let bans = query.fetch_all(pool).await?;

    if bans.is_empty() && !player_exists(ckey, pool).await? {
        return Err(Error::PlayerNotFound);
    }

    Ok(bans)
}

/// Represents a character used by a player.
#[derive(Object)]
pub struct Character {
    /// The name of the character
    pub name: String,
    /// The number of times this character has been played
    pub occurrences: i64,
}

impl FromRow<'_, MySqlRow> for Character {
    fn from_row(row: &MySqlRow) -> sqlx::Result<Self> {
        Ok(Character {
            name: row.try_get("character_name")?,
            occurrences: row.try_get("times")?,
        })
    }
}

/// Retrieves a player's most frequently used characters.
///
/// # Arguments
///
/// * `ckey` - Player's ckey (case-insensitive)
/// * `pool` - Database connection pool
///
/// # Returns
///
/// A list of characters ordered by frequency of use (excluding certain special roles)
///
/// # Errors
///
/// Returns `Error::PlayerNotFound` if the player doesn't exist
pub async fn get_player_characters(ckey: &str, pool: &MySqlPool) -> Result<Vec<Character>> {
    // Special roles misleadingly counted as characters, exclude them
    const EXCLUDED_ROLES: &str = "('Cargorilla')";

    let sql = const_format!(
        "SELECT character_name, COUNT(*) AS times FROM manifest WHERE ckey = ? AND special NOT IN {EXCLUDED_ROLES} GROUP BY character_name ORDER BY times DESC"
    );

    let query = sqlx::query_as(sql).bind(ckey.to_lowercase());

    let characters = query.fetch_all(pool).await?;

    if characters.is_empty() && !player_exists(ckey, pool).await? {
        return Err(Error::PlayerNotFound);
    }

    Ok(characters)
}

/// Represents a player's daily activity.
#[derive(Object)]
pub struct Activity {
    /// The date of the activity in YYYY-MM-DD format
    pub date: String,
    /// The number of rounds played on that date
    pub rounds: i64,
}

impl FromRow<'_, MySqlRow> for Activity {
    fn from_row(row: &MySqlRow) -> sqlx::Result<Self> {
        Ok(Activity {
            date: row.try_get::<Date, _>("date")?.to_string(),
            rounds: row.try_get("rounds")?,
        })
    }
}

/// Retrieves a player's activity over the last 180 days.
///
/// # Arguments
///
/// * `ckey` - Player's ckey (case-insensitive)
/// * `pool` - Database connection pool
///
/// # Returns
///
/// A list of daily activity records showing rounds played per day
///
/// # Errors
///
/// Returns `Error::PlayerNotFound` if the player doesn't exist
pub async fn get_player_activity(ckey: &str, pool: &MySqlPool) -> Result<Vec<Activity>> {
    let query = sqlx::query_as(
        "SELECT DATE(datetime) AS date, COUNT(DISTINCT round_id) AS rounds FROM connection_log WHERE ckey = ? AND datetime >= DATE_SUB(CURDATE(), INTERVAL 180 DAY) GROUP BY date;"
    )
    .bind(ckey.to_lowercase());

    let activity = query.fetch_all(pool).await?;

    if activity.is_empty() && !player_exists(ckey, pool).await? {
        return Err(Error::PlayerNotFound);
    }

    Ok(activity)
}
