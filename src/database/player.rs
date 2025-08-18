use const_format::formatcp as const_format;
use futures::TryStreamExt;
use poem_openapi::Object;
use sqlx::{Executor as _, MySql, MySqlPool, Row as _, pool::PoolConnection};

use crate::sqlxext::{Date, DateTime};

use super::{Error, Result};

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

async fn player_exists(ckey: &str, connection: &mut PoolConnection<MySql>) -> Result<bool> {
    let query = sqlx::query("SELECT 1 FROM player WHERE LOWER(ckey) = ?").bind(ckey.to_lowercase());
    Ok(connection.fetch_optional(query).await?.is_some())
}
