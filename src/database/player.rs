use const_format::formatcp as const_format;
use futures::TryStreamExt;
use poem_openapi::{Enum, Object};
use sqlx::{Executor as _, MySqlPool, Row as _};

use crate::sqlxext::{Date, DateTime};

use super::{Error, Result, player_exists};

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

pub async fn get_player(ckey: &str, pool: &MySqlPool) -> Result<Player> {
    let mut connection = pool.acquire().await?;

    let query = sqlx::query(
        "SELECT ckey, byond_key, firstseen, firstseen_round_id, lastseen, lastseen_round_id, INET_NTOA(ip), computerid, accountjoindate FROM player WHERE LOWER(ckey) = ?"
    )
    .bind(ckey.to_lowercase());

    let Ok(row) = connection.fetch_one(query).await else {
        return Err(Error::PlayerNotFound);
    };

    let player = Player {
        ckey: row.try_get("ckey")?,
        byond_key: row.try_get("byond_key")?,
        first_seen: row.try_get::<DateTime, _>("firstseen")?.into(),
        last_seen: row.try_get::<DateTime, _>("lastseen")?.into(),
        first_seen_round: row.try_get("firstseen_round_id")?,
        last_seen_round: row.try_get("lastseen_round_id")?,
        byond_age: row
            .try_get::<Option<Date>, _>("accountjoindate")?
            .map(Into::into),
    };

    Ok(player)
}

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

pub async fn get_player_achievements(
    ckey: &str,
    achievement_type: &Option<String>,
    pool: &MySqlPool,
) -> Result<Vec<Achievement>> {
    let mut connection = pool.acquire().await?;

    let mut sql = "SELECT m.achievement_key, m.achievement_version, m.achievement_type, m.achievement_name, m.achievement_description, a.value, a.last_updated FROM achievements a JOIN achievement_metadata m ON a.achievement_key = m.achievement_key WHERE LOWER(a.ckey) = ?".to_string();

    if achievement_type.is_some() {
        sql.push_str(" AND m.achievement_type = ?");
    }

    sql.push_str(" ORDER BY a.last_updated DESC");

    let mut query = sqlx::query(&sql).bind(ckey.to_lowercase());

    if let Some(achievement_type) = achievement_type {
        query = query.bind(achievement_type);
    }

    let mut achievements = Vec::new();

    {
        let mut rows = connection.fetch(query);

        while let Some(row) = rows.try_next().await? {
            achievements.push(Achievement {
                achievement_key: row.try_get("achievement_key")?,
                achievement_version: row.try_get("achievement_version")?,
                achievement_type: row
                    .try_get::<Option<String>, _>("achievement_type")?
                    .map(Into::into),
                achievement_name: row.try_get("achievement_name")?,
                achievement_description: row.try_get("achievement_description")?,
                value: row.try_get("value")?,
                timestamp: row.try_get::<DateTime, _>("last_updated")?.into(),
            });
        }
    }

    if achievements.is_empty() && !player_exists(ckey, &mut connection).await? {
        return Err(Error::PlayerNotFound);
    }

    Ok(achievements)
}

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

pub async fn get_player_bans(
    ckey: &str,
    permanent: bool,
    since: &Option<String>,
    pool: &MySqlPool,
) -> Result<Vec<Ban>> {
    let mut connection = pool.acquire().await?;

    let mut sql = "SELECT id, bantime, round_id, GROUP_CONCAT(role ORDER BY role SEPARATOR ', ') AS roles, expiration_time, reason, ckey, a_ckey, edits, unbanned_datetime, unbanned_ckey FROM ban WHERE LOWER(ckey) = ?".to_string();

    if permanent {
        sql.push_str(" AND expiration_time IS NULL");
    }

    if since.is_some() {
        sql.push_str(" AND bantime > ?");
    }

    sql.push_str(" GROUP BY bantime");

    let mut query = sqlx::query(&sql).bind(ckey.to_lowercase());

    if let Some(since) = since {
        query = query.bind(since);
    }

    let mut bans = Vec::new();

    {
        let mut rows = connection.fetch(query);

        while let Some(row) = rows.try_next().await? {
            bans.push(Ban {
                bantime: row.try_get::<DateTime, _>("bantime")?.into(),
                round_id: row.try_get("round_id")?,
                roles: row.try_get("roles")?,
                expiration_time: row
                    .try_get::<Option<DateTime>, _>("expiration_time")?
                    .map(Into::into),
                reason: row.try_get("reason")?,
                ckey: row.try_get("ckey")?,
                a_ckey: row.try_get("a_ckey")?,
                edits: row.try_get("edits")?,
                unbanned_datetime: row
                    .try_get::<Option<DateTime>, _>("unbanned_datetime")?
                    .map(Into::into),
                unbanned_ckey: row.try_get("unbanned_ckey")?,
            });
        }
    }

    if bans.is_empty() && !player_exists(ckey, &mut connection).await? {
        return Err(Error::PlayerNotFound);
    }

    Ok(bans)
}

#[derive(Object)]
pub struct Character {
    /// The name of the character
    pub name: String,
    /// The number of times this character has been played
    pub occurrences: i64,
}

pub async fn get_player_characters(ckey: &str, pool: &MySqlPool) -> Result<Vec<Character>> {
    let mut connection = pool.acquire().await?;

    const EXCLUDED_ROLES: &str = "('Operative', 'Wizard')";

    let sql = const_format!(
        "SELECT character_name, COUNT(*) AS times FROM manifest WHERE ckey = ? AND special NOT IN {EXCLUDED_ROLES} GROUP BY character_name ORDER BY times DESC"
    );

    let query = sqlx::query(sql).bind(ckey.to_lowercase());

    let mut characters = Vec::new();

    {
        let mut rows = connection.fetch(query);

        while let Some(row) = rows.try_next().await? {
            characters.push(Character {
                name: row.try_get("character_name")?,
                occurrences: row.try_get("times")?,
            });
        }
    }

    if characters.is_empty() && !player_exists(ckey, &mut connection).await? {
        return Err(Error::PlayerNotFound);
    }

    Ok(characters)
}

#[derive(Object)]
pub struct Activity {
    /// The date of the activity in YYYY-MM-DD format
    pub date: String,
    /// The number of rounds played on that date
    pub rounds: i64,
}

pub async fn get_player_activity(ckey: &str, pool: &MySqlPool) -> Result<Vec<Activity>> {
    let mut connection = pool.acquire().await?;

    let query = sqlx::query(
        "SELECT DATE(datetime) AS date, COUNT(DISTINCT round_id) AS rounds FROM connection_log WHERE ckey = ? AND datetime >= DATE_SUB(CURDATE(), INTERVAL 180 DAY) GROUP BY date;"
    )
    .bind(ckey.to_lowercase());

    let mut activity = Vec::new();

    {
        let mut rows = connection.fetch(query);

        while let Some(row) = rows.try_next().await? {
            activity.push(Activity {
                date: row.try_get::<Date, _>("date")?.into(),
                rounds: row.try_get("rounds")?,
            });
        }
    }

    if activity.is_empty() && !player_exists(ckey, &mut connection).await? {
        return Err(Error::PlayerNotFound);
    }

    Ok(activity)
}
