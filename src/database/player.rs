use chrono::{NaiveDate, NaiveDateTime};
use const_format::concatcp;
use rocket::futures::StreamExt as _;
use serde::Serialize;
use sqlx::{pool::PoolConnection, Executor as _, FromRow, MySql, MySqlPool, Row as _};

use crate::config::Config;

use super::error::Error;

#[derive(Debug, Serialize)]
pub struct Player {
    pub ckey: String,
    pub byond_key: Option<String>,
    #[serde(with = "crate::serde::datetime")]
    pub first_seen: NaiveDateTime,
    #[serde(with = "crate::serde::datetime")]
    pub last_seen: NaiveDateTime,
    pub first_seen_round: Option<u32>,
    pub last_seen_round: Option<u32>,
    #[serde(with = "crate::serde::opt_date")]
    pub byond_age: Option<NaiveDate>,
}

pub async fn get_player(ckey: &str, pool: &MySqlPool) -> Result<Player, Error> {
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
        first_seen: row.try_get("firstseen")?,
        last_seen: row.try_get("lastseen")?,
        first_seen_round: row.try_get("firstseen_round_id")?,
        last_seen_round: row.try_get("lastseen_round_id")?,
        byond_age: row.try_get("accountjoindate")?,
    };

    connection.close().await?;

    Ok(player)
}

#[derive(Debug, Serialize)]
pub struct JobRoletime {
    ckey: String,
    minutes: u32,
}

pub async fn get_top_roletime(job: &str, pool: &MySqlPool) -> Result<Vec<JobRoletime>, Error> {
    let mut connection = pool.acquire().await?;

    let query = sqlx::query(
        "SELECT ckey, minutes FROM role_time WHERE LOWER(job) = ? ORDER BY minutes DESC LIMIT 15",
    )
    .bind(job.to_lowercase());

    let mut roletimes = Vec::new();

    {
        let mut rows = connection.fetch(query);

        while let Some(row) = rows.next().await {
            let row = row?;

            let roletime = JobRoletime {
                ckey: row.try_get("ckey")?,
                minutes: row.try_get("minutes")?,
            };

            roletimes.push(roletime);
        }
    }

    connection.close().await?;

    Ok(roletimes)
}

#[derive(Debug, Serialize)]
pub struct PlayerRoletime {
    job: String,
    minutes: u32,
}

pub async fn get_roletime(ckey: &str, pool: &MySqlPool) -> Result<Vec<PlayerRoletime>, Error> {
    let mut connection = pool.acquire().await?;

    let query = sqlx::query(
        "SELECT job, minutes FROM role_time WHERE LOWER(ckey) = ? ORDER BY minutes DESC",
    )
    .bind(ckey.to_lowercase());

    let mut roletimes = Vec::new();

    {
        let mut rows = connection.fetch(query);

        while let Some(row) = rows.next().await {
            let row = row?;

            let roletime = PlayerRoletime {
                job: row.try_get("job")?,
                minutes: row.try_get("minutes")?,
            };

            roletimes.push(roletime);
        }
    }

    if roletimes.is_empty() && !player_exists(ckey, &mut connection).await {
        connection.close().await?;
        return Err(Error::PlayerNotFound);
    }

    connection.close().await?;

    Ok(roletimes)
}

pub async fn get_jobs(job: &str, pool: &MySqlPool) -> Result<Vec<String>, Error> {
    let mut connection = pool.acquire().await?;

    let query = sqlx::query(
        "SELECT DISTINCT job FROM role_time WHERE LOWER(job) LIKE ? ORDER BY job ASC LIMIT 25",
    )
    .bind(format!("%{}%", job.to_lowercase()));

    let mut jobs = Vec::new();

    {
        let mut rows = connection.fetch(query);

        while let Some(row) = rows.next().await {
            let row = row?;

            let job = row.try_get("job")?;

            jobs.push(job);
        }
    }

    connection.close().await?;

    Ok(jobs)
}

pub async fn get_ckeys(
    ckey: &str,
    pool: &MySqlPool,
    config: &Config,
) -> Result<Vec<String>, Error> {
    let mut connection = pool.acquire().await?;

    let sql = format!("SELECT p.ckey FROM {}.player p LEFT JOIN {}.hid_ckeys_autocomplete i ON i.ckey = p.ckey WHERE p.ckey LIKE ? AND (i.ckey IS NULL OR i.valid IS FALSE) ORDER BY p.ckey LIMIT 25", config.database.game_database, config.database.api_database);
    let query = sqlx::query(&sql).bind(format!("{ckey}%"));

    let mut ckeys = Vec::new();

    {
        let mut rows = connection.fetch(query);

        while let Some(row) = rows.next().await {
            let row = row?;

            let ckey = row.try_get("ckey")?;

            ckeys.push(ckey);
        }
    }

    connection.close().await?;

    Ok(ckeys)
}

#[derive(Debug, Serialize)]
pub struct Ban {
    #[serde(with = "crate::serde::datetime")]
    pub bantime: NaiveDateTime,
    pub round_id: Option<u32>,
    pub roles: Option<String>,
    #[serde(with = "crate::serde::opt_datetime")]
    pub expiration_time: Option<NaiveDateTime>,
    pub reason: String,
    pub ckey: Option<String>,
    pub a_ckey: String,
    pub edits: Option<String>,
    #[serde(with = "crate::serde::opt_datetime")]
    pub unbanned_datetime: Option<NaiveDateTime>,
    pub unbanned_ckey: Option<String>,
}

pub async fn get_ban(
    ckey: &str,
    permanent: bool,
    since: Option<&str>,
    pool: &MySqlPool,
) -> Result<Vec<Ban>, Error> {
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

        while let Some(row) = rows.next().await {
            let ban = row?;

            let ban = Ban {
                bantime: ban.try_get("bantime")?,
                round_id: ban.try_get("round_id")?,
                roles: ban.try_get("roles")?,
                expiration_time: ban.try_get("expiration_time")?,
                reason: ban.try_get("reason")?,
                ckey: ban.try_get("ckey")?,
                a_ckey: ban.try_get("a_ckey")?,
                edits: ban.try_get("edits")?,
                unbanned_datetime: ban.try_get("unbanned_datetime")?,
                unbanned_ckey: ban.try_get("unbanned_ckey")?,
            };

            bans.push(ban);
        }
    }

    if bans.is_empty() && !player_exists(ckey, &mut connection).await {
        connection.close().await?;
        return Err(Error::PlayerNotFound);
    }

    connection.close().await?;

    Ok(bans)
}

pub async fn player_exists(ckey: &str, connection: &mut PoolConnection<MySql>) -> bool {
    let query = sqlx::query("SELECT 1 FROM player WHERE LOWER(ckey) = ?").bind(ckey.to_lowercase());
    connection.fetch_one(query).await.is_ok()
}

#[derive(Debug, Serialize)]
pub struct IcName {
    pub name: String,
    pub ckey: String,
}

pub async fn get_ic_names(ic_name: &str, pool: &MySqlPool) -> Result<Vec<IcName>, Error> {
    let mut connection = pool.acquire().await?;

    let query = sqlx::query(
        "SELECT DISTINCT name, byondkey FROM death WHERE name LIKE ? ORDER BY name ASC LIMIT 25",
    )
    .bind(format!("%{ic_name}%"));

    let mut ckeys = Vec::new();

    {
        let mut rows = connection.fetch(query);

        while let Some(row) = rows.next().await {
            let row = row?;

            ckeys.push(IcName {
                name: row.try_get("name")?,
                ckey: row.try_get("byondkey")?,
            });
        }
    }

    connection.close().await?;

    Ok(ckeys)
}

pub async fn get_characters(ckey: &str, pool: &MySqlPool) -> Result<Vec<(String, i64)>, Error> {
    let mut connection = pool.acquire().await?;

    const EXCLUDED_ROLES: &str = "('Operative', 'Wizard')";

    let query = sqlx::query(concatcp!(
        "SELECT character_name, COUNT(*) AS times FROM manifest WHERE ckey = ? AND special NOT IN ",
        EXCLUDED_ROLES,
        " GROUP BY character_name ORDER BY times DESC"
    ))
    .bind(ckey.to_lowercase());

    let mut characters = Vec::new();

    {
        let mut rows = connection.fetch(query);

        while let Some(row) = rows.next().await {
            let row = row?;

            characters.push((row.try_get("character_name")?, row.try_get("times")?));
        }
    }

    if characters.is_empty() && !player_exists(ckey, &mut connection).await {
        connection.close().await?;
        return Err(Error::PlayerNotFound);
    }

    connection.close().await?;

    Ok(characters)
}

pub async fn get_activity(ckey: &str, pool: &MySqlPool) -> Result<Vec<(String, i64)>, Error> {
    let mut connection = pool.acquire().await?;

    let query = sqlx::query(
        "SELECT DATE(datetime) AS date, COUNT(DISTINCT round_id) AS rounds FROM connection_log WHERE ckey = ? AND datetime >= DATE_SUB(CURDATE(), INTERVAL 180 DAY) GROUP BY date;"
    )
    .bind(ckey.to_lowercase());

    let mut activity = Vec::new();

    {
        let mut rows = connection.fetch(query);

        while let Some(row) = rows.next().await {
            let row = row?;

            let date: NaiveDate = row.try_get("date")?;
            let date = date.format("%Y-%m-%d").to_string();

            activity.push((date, row.try_get("rounds")?));
        }
    }

    if activity.is_empty() && !player_exists(ckey, &mut connection).await {
        connection.close().await?;
        return Err(Error::PlayerNotFound);
    }

    connection.close().await?;

    Ok(activity)
}

#[derive(Debug, Serialize, FromRow)]
pub struct Achievement {
    pub achievement_key: String,
    pub achievement_version: u16,
    pub achievement_type: Option<String>,
    pub achievement_name: Option<String>,
    pub achievement_description: Option<String>,
    pub value: Option<i32>,
    #[serde(with = "crate::serde::datetime")]
    pub timestamp: NaiveDateTime,
}

pub async fn get_achievements(
    ckey: &str,
    achievement_type: Option<&str>,
    pool: &MySqlPool,
) -> Result<Vec<Achievement>, Error> {
    let mut connection = pool.acquire().await?;

    let mut sql = "SELECT a.value, a.last_updated, m.achievement_key, m.achievement_version, m.achievement_type, m.achievement_name, m.achievement_description FROM achievements a JOIN achievement_metadata m ON a.achievement_key = m.achievement_key WHERE LOWER(a.ckey) = ?".to_string();

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

        while let Some(row) = rows.next().await {
            let achievement = row?;

            let achievement = Achievement {
                achievement_key: achievement.try_get("achievement_key")?,
                achievement_version: achievement.try_get("achievement_version")?,
                achievement_type: achievement.try_get("achievement_type")?,
                achievement_name: achievement.try_get("achievement_name")?,
                achievement_description: achievement.try_get("achievement_description")?,
                value: achievement.try_get("value")?,
                timestamp: achievement.try_get("last_updated")?,
            };

            achievements.push(achievement);
        }
    }

    if achievements.is_empty() && !player_exists(ckey, &mut connection).await {
        connection.close().await?;
        return Err(Error::PlayerNotFound);
    }

    connection.close().await?;

    Ok(achievements)
}

pub async fn hide_ckey(
    ckey: &str,
    hid_by: i64,
    pool: &MySqlPool,
    config: &Config,
) -> Result<bool, Error> {
    let mut connection = pool.acquire().await?;

    let sql = format!(
        "SELECT 1 FROM {}.hid_ckeys_autocomplete WHERE ckey = ? AND valid = 1",
        config.database.api_database
    );
    let query = sqlx::query(&sql).bind(ckey.to_lowercase());

    if connection.fetch_optional(query).await?.is_some() {
        return Ok(false);
    }

    let sql = format!(
        "INSERT INTO {}.hid_ckeys_autocomplete (ckey, hid_by, valid) VALUES (?, ?, 1)",
        config.database.api_database
    );
    let query = sqlx::query(&sql).bind(ckey.to_lowercase()).bind(hid_by);

    connection.execute(query).await?;
    connection.close().await?;

    Ok(true)
}

pub async fn unhide_ckey(
    ckey: &str,
    unhid_by: i64,
    pool: &MySqlPool,
    config: &Config,
) -> Result<bool, Error> {
    let mut connection = pool.acquire().await?;

    let sql = format!(
        "SELECT 1 FROM {}.hid_ckeys_autocomplete WHERE ckey = ? AND valid = 1",
        config.database.api_database
    );
    let query = sqlx::query(&sql).bind(ckey.to_lowercase());

    if connection.fetch_optional(query).await?.is_none() {
        return Ok(false);
    }

    let sql = format!(
        "UPDATE {}.hid_ckeys_autocomplete SET valid = 0, unhid_by = ? WHERE ckey = ? AND valid = 1",
        config.database.api_database
    );
    let query = sqlx::query(&sql).bind(unhid_by).bind(ckey.to_lowercase());

    connection.execute(query).await?;
    connection.close().await?;

    Ok(true)
}

pub async fn lookup_player(
    ckey: Option<&str>,
    ip: Option<&str>,
    cid: Option<i64>,
    pool: &MySqlPool,
) -> Result<Vec<(String, String, String)>, Error> {
    let mut connection = pool.acquire().await?;

    let mut sql =
        "SELECT computerid, INET_NTOA(ip) AS ip, ckey FROM connection_log WHERE ".to_string();

    if ckey.is_some() {
        sql.push_str(
            "(computerid IN (SELECT DISTINCT computerid FROM connection_log WHERE ckey = ?) OR
             ip IN (SELECT DISTINCT ip FROM connection_log WHERE ckey = ?))",
        );
    } else if ip.is_some() {
        sql.push_str(
            "(computerid IN (SELECT DISTINCT computerid FROM connection_log WHERE ip = INET_ATON(?)) OR
             ckey IN (SELECT DISTINCT ckey FROM connection_log WHERE ip = INET_ATON(?)))"
        );
    } else if cid.is_some() {
        sql.push_str(
            "(ip IN (SELECT DISTINCT ip FROM connection_log WHERE computerid = ?) OR
             ckey IN (SELECT DISTINCT ckey FROM connection_log WHERE computerid = ?))",
        );
    } else {
        unreachable!();
    }

    sql.push_str(" GROUP BY ckey, computerid, ip ORDER BY ckey, computerid, ip");

    let mut query = sqlx::query(&sql);

    if let Some(ckey) = ckey {
        query = query.bind(ckey.to_lowercase()).bind(ckey.to_lowercase());
    } else if let Some(ip) = ip {
        query = query.bind(ip).bind(ip);
    } else if let Some(cid) = cid {
        query = query.bind(cid).bind(cid);
    }

    let mut rows = connection.fetch(query);

    let mut result = Vec::new();

    while let Some(row) = rows.next().await {
        let row = row?;

        let cid: String = row.try_get("computerid")?;
        let ip: String = row.try_get("ip")?;
        let ckey: String = row.try_get("ckey")?;

        result.push((cid, ip, ckey));
    }

    Ok(result)
}
