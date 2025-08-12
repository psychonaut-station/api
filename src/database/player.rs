use futures::TryStreamExt;
use poem_openapi::Object;
use sqlx::{Executor as _, MySql, MySqlPool, Row as _, pool::PoolConnection};

use crate::sqlxext::{Date, DateTime};

use super::{Error, Result};

#[derive(Object)]
pub struct Player {
    pub ckey: String,
    pub byond_key: Option<String>,
    pub first_seen: String,
    pub last_seen: String,
    pub first_seen_round: Option<u32>,
    pub last_seen_round: Option<u32>,
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
    pub bantime: String,
    pub round_id: Option<u32>,
    pub roles: Option<String>,
    pub expiration_time: Option<String>,
    pub reason: String,
    pub ckey: Option<String>,
    pub a_ckey: String,
    pub edits: Option<String>,
    pub unbanned_datetime: Option<String>,
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

async fn player_exists(ckey: &str, connection: &mut PoolConnection<MySql>) -> Result<bool> {
    let query = sqlx::query("SELECT 1 FROM player WHERE LOWER(ckey) = ?").bind(ckey.to_lowercase());
    Ok(connection.fetch_optional(query).await?.is_some())
}
