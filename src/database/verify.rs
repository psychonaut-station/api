use rand::Rng as _;
use regex::Regex;
use sqlx::{pool::PoolConnection, Executor as _, MySql, MySqlPool, Row as _};

use crate::http::discord::{self, User};

use super::{error::Error, player_exists};

pub async fn verify_discord(
    discord_id: &str,
    one_time_token: Option<&str>,
    ckey: Option<&str>,
    pool: &MySqlPool,
) -> Result<Option<String>, Error> {
    let mut connection = pool.acquire().await?;

    if let Ok(ckey) = ckey_by_discord_id(discord_id, &mut connection).await {
        return Err(Error::DiscordInUse(ckey));
    }

    if let Some(one_time_token) = one_time_token {
        let regex = Regex::new(r"^\d{3}-\d{3}$").unwrap();
        if !regex.is_match(one_time_token) {
            return Err(Error::TokenInvalid);
        }

        match discord_id_by_token(one_time_token, false, &mut connection).await {
            Err(Error::NotLinked) => {}
            _ => return Err(Error::TokenInvalid),
        }

        if let Ok(discord_id) = discord_id_by_token(one_time_token, true, &mut connection).await {
            return Err(Error::CkeyInUse(discord_id));
        }

        let query = sqlx::query(
            "UPDATE discord_links SET discord_id = ?, valid = 1 WHERE one_time_token = ?",
        )
        .bind(discord_id)
        .bind(one_time_token);

        connection.execute(query).await?;

        let ckey = ckey_by_discord_id(discord_id, &mut connection).await?;

        connection.close().await?;

        return Ok(Some(ckey));
    } else if let Some(ckey) = ckey {
        if !player_exists(ckey, &mut connection).await {
            return Err(Error::PlayerNotFound);
        }

        if let Ok(discord_id) = discord_id_by_ckey(ckey, &mut connection).await {
            return Err(Error::CkeyInUse(discord_id));
        }

        let token = generate_one_time_token(&mut connection).await;
        let discord_id = discord_id.parse::<i64>()?;

        let query = sqlx::query(
            "INSERT INTO discord_links (discord_id, ckey, one_time_token, valid) VALUES (?, ?, ?, 1)",
        )
        .bind(discord_id)
        .bind(ckey.to_lowercase())
        .bind(token);

        connection.execute(query).await?;
        connection.close().await?;

        return Ok(None);
    }

    unreachable!()
}

pub async fn unverify_discord(
    discord_id: Option<&str>,
    ckey: Option<&str>,
    pool: &MySqlPool,
) -> Result<String, Error> {
    let mut connection = pool.acquire().await?;

    if let Some(discord_id) = discord_id {
        let ckey = ckey_by_discord_id(discord_id, &mut connection).await?;

        let query =
            sqlx::query("UPDATE discord_links SET valid = 0 WHERE discord_id = ? AND valid = 1")
                .bind(discord_id);

        connection.execute(query).await?;
        connection.close().await?;

        return Ok(ckey);
    } else if let Some(ckey) = ckey {
        let discord_id = discord_id_by_ckey(ckey, &mut connection).await?;

        let query =
            sqlx::query("UPDATE discord_links SET valid = 0 WHERE LOWER(ckey) = ? AND valid = 1")
                .bind(ckey.to_lowercase());

        connection.execute(query).await?;
        connection.close().await?;

        return Ok(format!("@{discord_id}"));
    }

    unreachable!()
}

async fn ckey_by_discord_id(
    discord_id: &str,
    connection: &mut PoolConnection<MySql>,
) -> Result<String, Error> {
    let query = sqlx::query("SELECT ckey FROM discord_links WHERE discord_id = ? AND valid = 1")
        .bind(discord_id);

    if let Ok(row) = connection.fetch_one(query).await {
        return Ok(row.try_get("ckey")?);
    }

    Err(Error::NotLinked)
}

pub async fn discord_id_by_ckey(
    ckey: &str,
    connection: &mut PoolConnection<MySql>,
) -> Result<i64, Error> {
    let query =
        sqlx::query("SELECT discord_id FROM discord_links WHERE LOWER(ckey) = ? AND valid = 1")
            .bind(ckey.to_lowercase());

    if let Ok(row) = connection.fetch_one(query).await {
        return Ok(row.try_get("discord_id")?);
    }

    if !player_exists(ckey, connection).await {
        return Err(Error::PlayerNotFound);
    }

    Err(Error::NotLinked)
}

async fn discord_id_by_token(
    one_time_token: &str,
    only_valid: bool,
    connection: &mut PoolConnection<MySql>,
) -> Result<i64, Error> {
    let mut sql = "SELECT discord_id FROM discord_links WHERE one_time_token = ?".to_string();

    if only_valid {
        sql.push_str(" AND valid = 1");
    }

    let query = sqlx::query(&sql).bind(one_time_token);

    if let Ok(row) = connection.fetch_one(query).await {
        return row.try_get("discord_id").map_err(|_| Error::NotLinked);
    }

    Err(Error::TokenInvalid)
}

pub async fn fetch_discord_by_ckey(
    ckey: &str,
    discord_token: &str,
    pool: &MySqlPool,
) -> Result<User, Error> {
    let mut connection = pool.acquire().await?;

    let discord_id = discord_id_by_ckey(ckey, &mut connection).await?;

    connection.close().await?;

    let user = discord::get_user(discord_id, discord_token).await?;

    Ok(user)
}

pub async fn get_ckey_by_discord_id(discord_id: &str, pool: &MySqlPool) -> Result<String, Error> {
    let mut connection = pool.acquire().await?;

    let ckey = ckey_by_discord_id(discord_id, &mut connection).await?;

    connection.close().await?;

    Ok(ckey)
}

async fn generate_one_time_token(connection: &mut PoolConnection<MySql>) -> String {
    loop {
        let token: u32 = rand::thread_rng().gen_range(1..=999_999);
        let token = format!("{:03}-{:03}", token / 1_000, token % 1_000);

        if let Err(Error::TokenInvalid) = discord_id_by_token(&token, false, connection).await {
            return token;
        }
    }
}
