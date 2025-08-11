use poem_openapi::Object;
use sqlx::{Executor as _, MySqlPool, Row as _};

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
