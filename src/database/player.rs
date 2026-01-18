use chrono::{NaiveDate, NaiveDateTime};
use const_format::concatcp;
use rocket::futures::StreamExt as _;
use serde::Serialize;
use sqlx::{pool::PoolConnection, Executor as _, FromRow, MySql, MySqlPool, Row as _};

use crate::config::Config;

use super::{error::Error, Ban};

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
            bans.push(row?.try_into()?);
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
        "SELECT DISTINCT character_name, ckey FROM manifest WHERE character_name LIKE ? ORDER BY character_name ASC LIMIT 25",
    )
    .bind(format!("%{ic_name}%"));

    let mut ckeys = Vec::new();

    {
        let mut rows = connection.fetch(query);

        while let Some(row) = rows.next().await {
            let row = row?;

            ckeys.push(IcName {
                name: row.try_get("character_name")?,
                ckey: row.try_get("ckey")?,
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

pub async fn get_favorite_character(
    ckey: &str,
    pool: &MySqlPool,
) -> Result<(String, String), Error> {
    let mut connection = pool.acquire().await?;

    const EXCLUDED_ROLES: &str = "('Operative', 'Wizard')";

    let query = sqlx::query(concatcp!(
        "SELECT character_name, job, COUNT(*) AS times FROM manifest WHERE ckey = ? AND special NOT IN ",
        EXCLUDED_ROLES,
        " GROUP BY character_name, job ORDER BY times DESC LIMIT 1"
    ))
    .bind(ckey.to_lowercase());

    let Ok(row) = connection.fetch_one(query).await else {
        return Err(Error::PlayerNotFound);
    };

    let character_name = row.try_get("character_name")?;
    let job = row.try_get("job")?;

    connection.close().await?;

    Ok((character_name, job))
}

#[derive(Serialize, Debug, FromRow)]
pub struct TicketLog {
    pub action: String,
    pub message: String,
    pub sender: Option<String>,
    pub recipient: Option<String>,
    #[serde(with = "crate::serde::datetime")]
    pub timestamp: chrono::NaiveDateTime,
}

#[derive(Serialize, Debug)]
pub struct TicketGroup {
    pub ticket_id: u32,
    pub round_id: u32,
    pub logs: Vec<TicketLog>,
}

pub async fn get_tickets(
    ckey: &str,
    fetch_size: Option<i32>,
    page: Option<i32>,
    pool: &MySqlPool,
) -> Result<(Vec<TicketGroup>, i64), Error> {
    let fetch_size = fetch_size.unwrap_or(10);
    let page = page.unwrap_or(1);
    let offset = (page - 1) * fetch_size;

    let mut connection = pool.acquire().await?;

    let total_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT ticket, round_id) FROM ticket 
         WHERE action = 'Ticket Opened' AND ((LOWER(sender) = ? AND recipient IS NULL) OR (LOWER(recipient) = ?))"
    )
    .bind(ckey.to_lowercase())
    .bind(ckey.to_lowercase())
    .fetch_one(&mut *connection)
    .await?;

    let target_tickets = sqlx::query(
        "SELECT ticket, round_id, MAX(timestamp) as latest_time 
         FROM ticket 
         WHERE action = 'Ticket Opened' AND ((LOWER(sender) = ? AND recipient IS NULL) OR (LOWER(recipient) = ?)) 
         GROUP BY round_id, ticket 
         ORDER BY latest_time DESC, round_id DESC, ticket DESC 
         LIMIT ? OFFSET ?"
    )
    .bind(ckey.to_lowercase())
    .bind(ckey.to_lowercase())
    .bind(fetch_size)
    .bind(offset)
    .fetch_all(&mut *connection)
    .await?;

    let mut results = Vec::new();

    for row in target_tickets {
        let t_id: u32 = row.get("ticket");
        let r_id: u32 = row.get("round_id");

        let logs = sqlx::query_as::<_, TicketLog>(
            "SELECT action, message, sender, recipient, timestamp 
             FROM ticket 
             WHERE ticket = ? AND round_id = ? 
             AND action NOT IN ('Reconnected', 'Disconnected', 'Interaction')
             ORDER BY timestamp ASC",
        )
        .bind(t_id)
        .bind(r_id)
        .fetch_all(&mut *connection)
        .await?;

        results.push(TicketGroup {
            ticket_id: t_id,
            round_id: r_id,
            logs,
        });
    }

    if results.is_empty() && !player_exists(ckey, &mut connection).await {
        connection.close().await?;
        return Err(Error::PlayerNotFound);
    }

    connection.close().await?;

    Ok((results, total_count))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Message {
    pub id: i32,
    pub targetckey: String,
    pub adminckey: String,
    pub text: String,
    #[serde(with = "crate::serde::datetime")]
    pub timestamp: chrono::NaiveDateTime,
    pub server: Option<String>,
    pub round_id: Option<u32>,
    #[serde(with = "crate::serde::opt_datetime")]
    pub expire_timestamp: Option<chrono::NaiveDateTime>,
    pub severity: Option<String>,
    pub playtime: Option<u32>,
    pub lasteditor: Option<String>,
    pub days_passed: i32,
}

pub async fn get_messages(
    ckey: &str,
    fetch_size: Option<i32>,
    page: Option<i32>,
    pool: &MySqlPool,
) -> Result<(Vec<Message>, i64), Error> {
    let fetch_size = fetch_size.unwrap_or(10);
    let page = page.unwrap_or(1);
    let offset = (page - 1) * fetch_size;

    let mut connection = pool.acquire().await?;

    let sql = "SELECT COUNT(*) FROM messages WHERE type IN ('message', 'message sent') AND LOWER(targetckey) = ? AND secret = 0 AND deleted = 0 AND (expire_timestamp > NOW() OR expire_timestamp IS NULL)".to_string();

    let query = sqlx::query_scalar(&sql).bind(ckey.to_lowercase());

    let total_count = query.fetch_one(&mut *connection).await?;

    let messages = sqlx::query_as::<_, Message>(
        "SELECT 
            id, targetckey, adminckey, text, timestamp, 
            server, round_id, expire_timestamp, 
            severity, playtime, lasteditor,
            DATEDIFF(NOW(), timestamp) AS days_passed
        FROM messages
        WHERE type IN ('message', 'message sent') 
          AND LOWER(targetckey) = ? 
          AND secret = 0 
          AND deleted = 0 
          AND (expire_timestamp > NOW() OR expire_timestamp IS NULL)
        ORDER BY timestamp DESC 
        LIMIT ? OFFSET ?",
    )
    .bind(ckey.to_lowercase())
    .bind(fetch_size)
    .bind(offset)
    .fetch_all(&mut *connection)
    .await?;

    if messages.is_empty() && !player_exists(ckey, &mut connection).await {
        connection.close().await?;
        return Err(Error::PlayerNotFound);
    }

    connection.close().await?;

    Ok((messages, total_count))
}

pub async fn get_notes(
    ckey: &str,
    fetch_size: Option<i32>,
    page: Option<i32>,
    pool: &MySqlPool,
) -> Result<(Vec<Message>, i64), Error> {
    let fetch_size = fetch_size.unwrap_or(10);
    let page = page.unwrap_or(1);
    let offset = (page - 1) * fetch_size;

    let mut connection = pool.acquire().await?;

    let sql = "SELECT COUNT(*) FROM messages WHERE type = 'note' AND LOWER(targetckey) = ? AND secret = 0 AND deleted = 0 AND (expire_timestamp > NOW() OR expire_timestamp IS NULL)".to_string();

    let query = sqlx::query_scalar(&sql).bind(ckey.to_lowercase());

    let total_count = query.fetch_one(&mut *connection).await?;

    let messages = sqlx::query_as::<_, Message>(
        "SELECT 
            id, targetckey, adminckey, text, timestamp, 
            server, round_id, expire_timestamp, 
            severity, playtime, lasteditor,
            DATEDIFF(NOW(), timestamp) AS days_passed
        FROM messages
        WHERE type = 'note' 
          AND LOWER(targetckey) = ? 
          AND secret = 0 
          AND deleted = 0 
          AND (expire_timestamp > NOW() OR expire_timestamp IS NULL)
        ORDER BY timestamp DESC 
        LIMIT ? OFFSET ?",
    )
    .bind(ckey.to_lowercase())
    .bind(fetch_size)
    .bind(offset)
    .fetch_all(&mut *connection)
    .await?;

    if messages.is_empty() && !player_exists(ckey, &mut connection).await {
        connection.close().await?;
        return Err(Error::PlayerNotFound);
    }

    connection.close().await?;

    Ok((messages, total_count))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ManifestData {
    pub id: i32,
    pub round_id: i32,
    pub ckey: String,
    pub character_name: String,
    pub job: String,
    pub special: Option<String>,
    pub latejoin: bool,
    #[serde(with = "crate::serde::datetime")]
    pub timestamp: NaiveDateTime,
}

pub async fn get_player_rounds(
    ckey: &str,
    fetch_size: Option<i32>,
    page: Option<i32>,
    pool: &MySqlPool,
) -> Result<(Vec<ManifestData>, i64), Error> {
    let fetch_size = fetch_size.unwrap_or(10);
    let page = page.unwrap_or(1);
    let offset = (page - 1) * fetch_size;

    let mut connection = pool.acquire().await?;

    let sql = "SELECT COUNT(*) FROM manifest WHERE LOWER(ckey) = ?".to_string();

    let query = sqlx::query_scalar(&sql).bind(ckey.to_lowercase());

    let total_count = query.fetch_one(&mut *connection).await?;

    let rounds = sqlx::query_as::<_, ManifestData>(
        "SELECT id, round_id, ckey, character_name, job, special, latejoin, timestamp FROM manifest WHERE LOWER(ckey) = ? ORDER BY timestamp DESC LIMIT ? OFFSET ?"
    )
    .bind(ckey.to_lowercase())
    .bind(fetch_size)
    .bind(offset)
    .fetch_all(&mut *connection)
    .await?;

    if rounds.is_empty() && !player_exists(ckey, &mut connection).await {
        connection.close().await?;
        return Err(Error::PlayerNotFound);
    }

    connection.close().await?;

    Ok((rounds, total_count))
}

#[derive(Debug, Serialize, FromRow)]
pub struct Friendship {
    pub id: i32,
    pub user_ckey: String,
    pub friend_ckey: String,
    pub status: String,
    #[serde(with = "crate::serde::datetime")]
    pub created_at: NaiveDateTime,
    #[serde(with = "crate::serde::datetime")]
    pub updated_at: NaiveDateTime,
}

pub async fn get_friends(
    ckey: &str,
    pool: &MySqlPool,
    config: &Config,
) -> Result<Vec<Friendship>, Error> {
    let mut connection = pool.acquire().await?;

    let sql = format!(
        "SELECT * FROM {}.friendship WHERE (LOWER(user_ckey) = ? OR LOWER(friend_ckey) = ?) AND status = 'accepted'",
        config.database.api_database
    );

    let friends = sqlx::query_as::<_, Friendship>(&sql)
        .bind(ckey.to_lowercase())
        .bind(ckey.to_lowercase())
        .fetch_all(&mut *connection)
        .await?;

    if friends.is_empty() && !player_exists(ckey, &mut connection).await {
        connection.close().await?;
        return Err(Error::PlayerNotFound);
    }

    connection.close().await?;

    Ok(friends)
}

pub async fn get_friendship_invites(
    ckey: &str,
    pool: &MySqlPool,
    config: &Config,
) -> Result<(Vec<Friendship>, Vec<Friendship>), Error> {
    let mut connection = pool.acquire().await?;

    let sql = format!(
        "SELECT * FROM {}.friendship WHERE LOWER(friend_ckey) = ? AND status = 'pending'",
        config.database.api_database
    );

    let received_requests = sqlx::query_as::<_, Friendship>(&sql)
        .bind(ckey.to_lowercase())
        .fetch_all(&mut *connection)
        .await?;

    let sql = format!(
        "SELECT * FROM {}.friendship WHERE LOWER(user_ckey) = ? AND status = 'pending'",
        config.database.api_database
    );

    let sent_requests = sqlx::query_as::<_, Friendship>(&sql)
        .bind(ckey.to_lowercase())
        .fetch_all(&mut *connection)
        .await?;

    if received_requests.is_empty()
        && sent_requests.is_empty()
        && !player_exists(ckey, &mut connection).await
    {
        connection.close().await?;
        return Err(Error::PlayerNotFound);
    }

    connection.close().await?;

    Ok((received_requests, sent_requests))
}

pub async fn check_friendship(
    ckey: &str,
    friend: &str,
    pool: &MySqlPool,
    config: &Config,
) -> Result<Option<Friendship>, Error> {
    let mut connection = pool.acquire().await?;

    let sql = format!(
        "SELECT * FROM {}.friendship WHERE ((LOWER(user_ckey) = ? AND LOWER(friend_ckey) = ?) OR (LOWER(user_ckey) = ? AND LOWER(friend_ckey) = ?)) LIMIT 1",
        config.database.api_database
    );

    let friendship = sqlx::query_as::<_, Friendship>(&sql)
        .bind(ckey.to_lowercase())
        .bind(friend.to_lowercase())
        .bind(friend.to_lowercase())
        .bind(ckey.to_lowercase())
        .fetch_optional(&mut *connection) // connection.acquire()'a gerek yok, direkt pool kullanabilirsin
        .await?;

    if friendship.is_none()
        && !player_exists(friend, &mut connection).await
        && !player_exists(ckey, &mut connection).await
    {
        connection.close().await?;
        return Err(Error::PlayerNotFound);
    }

    connection.close().await?;

    Ok(friendship)
}

pub async fn add_friend(
    ckey: &str,
    friend: &str,
    pool: &MySqlPool,
    config: &Config,
) -> Result<Option<Friendship>, Error> {
    let mut connection = pool.acquire().await?;

    if !player_exists(friend, &mut connection).await && !player_exists(ckey, &mut connection).await
    {
        connection.close().await?;
        return Err(Error::PlayerNotFound);
    }

    let sql = format!(
        "INSERT INTO {}.friendship (user_ckey, friend_ckey) VALUES (?, ?) ON DUPLICATE KEY UPDATE 
            user_ckey = IF(status = 'accepted', user_ckey, VALUES(user_ckey)),
            friend_ckey = IF(status = 'accepted', friend_ckey, VALUES(friend_ckey)),
            status = IF(status = 'accepted', 'accepted', 'pending')",
        config.database.api_database
    );
    let result = sqlx::query(&sql)
        .bind(ckey.to_lowercase())
        .bind(friend.to_lowercase())
        .execute(&mut *connection)
        .await?;

    if result.rows_affected() > 0 {
        let sql = format!(
            "SELECT * FROM {}.friendship WHERE LOWER(user_ckey) = ? AND LOWER(friend_ckey) = ?",
            config.database.api_database
        );
        let updated_row = sqlx::query_as::<_, Friendship>(&sql)
            .bind(ckey.to_lowercase())
            .bind(friend.to_lowercase())
            .fetch_one(&mut *connection)
            .await?;

        connection.close().await?;
        return Ok(Some(updated_row));
    }

    connection.close().await?;
    Ok(None)
}

pub async fn remove_friend(
    ckey: &str,
    friendship_id: i32,
    pool: &MySqlPool,
    config: &Config,
) -> Result<Option<Friendship>, Error> {
    let mut connection = pool.acquire().await?;

    if !player_exists(ckey, &mut connection).await {
        connection.close().await?;
        return Err(Error::PlayerNotFound);
    }

    let sql = format!(
        "UPDATE {}.friendship SET status = 'removed' WHERE id = ? AND (LOWER(user_ckey) = ? OR LOWER(friend_ckey) = ?) AND status = 'accepted'",
        config.database.api_database
    );
    let result = sqlx::query(&sql)
        .bind(friendship_id)
        .bind(ckey.to_lowercase())
        .bind(ckey.to_lowercase())
        .execute(&mut *connection)
        .await?;

    if result.rows_affected() > 0 {
        let sql = format!(
            "SELECT * FROM {}.friendship WHERE id = ?",
            config.database.api_database
        );
        let updated_row = sqlx::query_as::<_, Friendship>(&sql)
            .bind(friendship_id)
            .fetch_one(&mut *connection)
            .await?;

        connection.close().await?;
        return Ok(Some(updated_row));
    }

    connection.close().await?;
    Ok(None)
}

pub async fn accept_friend(
    ckey: &str,
    friendship_id: i32,
    pool: &MySqlPool,
    config: &Config,
) -> Result<Option<Friendship>, Error> {
    let mut connection = pool.acquire().await?;

    if !player_exists(ckey, &mut connection).await {
        connection.close().await?;
        return Err(Error::PlayerNotFound);
    }

    let sql = format!(
        "UPDATE {}.friendship SET status = 'accepted' WHERE id = ? AND LOWER(friend_ckey) = ? AND status = 'pending'",
        config.database.api_database
    );
    let result = sqlx::query(&sql)
        .bind(friendship_id)
        .bind(ckey.to_lowercase())
        .execute(&mut *connection)
        .await?;

    if result.rows_affected() > 0 {
        let sql = format!(
            "SELECT * FROM {}.friendship WHERE id = ?",
            config.database.api_database
        );
        let updated_row = sqlx::query_as::<_, Friendship>(&sql)
            .bind(friendship_id)
            .fetch_one(&mut *connection)
            .await?;
        connection.close().await?;
        return Ok(Some(updated_row));
    }

    connection.close().await?;
    Ok(None)
}

pub async fn decline_friend(
    ckey: &str,
    friendship_id: i32,
    pool: &MySqlPool,
    config: &Config,
) -> Result<Option<Friendship>, Error> {
    let mut connection = pool.acquire().await?;

    if !player_exists(ckey, &mut connection).await {
        connection.close().await?;
        return Err(Error::PlayerNotFound);
    }

    let sql = format!(
        "UPDATE {}.friendship SET status = 'declined' WHERE id = ? AND (LOWER(user_ckey) = ? OR LOWER(friend_ckey) = ?) AND status = 'pending'",
        config.database.api_database
    );

    let result = sqlx::query(&sql)
        .bind(friendship_id)
        .bind(ckey.to_lowercase())
        .bind(ckey.to_lowercase())
        .execute(&mut *connection)
        .await?;

    if result.rows_affected() > 0 {
        let sql = format!(
            "SELECT * FROM {}.friendship WHERE id = ?",
            config.database.api_database
        );
        let updated_row = sqlx::query_as::<_, Friendship>(&sql)
            .bind(friendship_id)
            .fetch_one(&mut *connection)
            .await?;
        connection.close().await?;
        return Ok(Some(updated_row));
    }

    connection.close().await?;
    Ok(None)
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
        "SELECT computerid, INET_NTOA(ip) AS readable_ip, ckey FROM connection_log WHERE "
            .to_string();

    // filters 104.28.0.0/16 (cloudflare) subnet
    const EXCLUSION_SUBNET: &str =
        " AND INET_NTOA(ip & INET_ATON('255.255.0.0')) NOT IN ('104.28.0.0')";

    if ckey.is_some() {
        sql.push_str(
            "(computerid IN (SELECT DISTINCT computerid FROM connection_log WHERE ckey = ?) OR
             ip IN (SELECT DISTINCT ip FROM connection_log WHERE ckey = ?))",
        );
        sql.push_str(EXCLUSION_SUBNET);
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
        sql.push_str(EXCLUSION_SUBNET);
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
        let ip: String = row.try_get("readable_ip")?;
        let ckey: String = row.try_get("ckey")?;

        result.push((cid, ip, ckey));
    }

    Ok(result)
}
