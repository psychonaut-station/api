use const_format::formatcp as const_format;
use futures::TryStreamExt as _;
use poem_openapi::Object;
use sqlx::{Executor as _, MySqlPool, Row as _, mysql::MySqlRow};

use super::Result;

const EXCLUSION_SUBNET: &str =
    " AND INET_NTOA(ip & INET_ATON('255.255.0.0')) NOT IN ('104.28.0.0')";

#[derive(Object)]
pub struct Lookup {
    /// The computer ID of the player
    pub computerid: String,
    /// The IP address of the player
    pub ip: String,
    /// The ckey of the player
    pub ckey: String,
}

impl TryFrom<MySqlRow> for Lookup {
    type Error = sqlx::Error;

    fn try_from(row: MySqlRow) -> std::result::Result<Self, Self::Error> {
        Ok(Lookup {
            computerid: row.try_get("computerid")?,
            ip: row.try_get("ip")?,
            ckey: row.try_get("ckey")?,
        })
    }
}

pub async fn lookup_cid(cid: &str, pool: &MySqlPool) -> Result<Vec<Lookup>> {
    let mut connection = pool.acquire().await?;

    const SQL: &str = const_format!(
        "SELECT computerid, INET_NTOA(ip) AS ip, ckey FROM connection_log WHERE
         ip IN (SELECT DISTINCT ip FROM connection_log WHERE computerid = ?) OR
         ckey IN (SELECT DISTINCT ckey FROM connection_log WHERE computerid = ?)
         {EXCLUSION_SUBNET} GROUP BY ckey, computerid, ip ORDER BY ckey, computerid, ip"
    );

    let query = sqlx::query(SQL).bind(cid).bind(cid);

    let mut stream = connection.fetch(query);

    let mut result = Vec::new();

    while let Some(row) = stream.try_next().await? {
        result.push(row.try_into()?);
    }

    Ok(result)
}

pub async fn lookup_ip(ip: &str, pool: &MySqlPool) -> Result<Vec<Lookup>> {
    let mut connection = pool.acquire().await?;

    const SQL: &str = const_format!(
        "SELECT computerid, INET_NTOA(ip) AS ip, ckey FROM connection_log WHERE
         computerid IN (SELECT DISTINCT computerid FROM connection_log WHERE ip = INET_ATON(?)) OR
         ckey IN (SELECT DISTINCT ckey FROM connection_log WHERE ip = INET_ATON(?))
         {EXCLUSION_SUBNET} GROUP BY ckey, computerid, ip ORDER BY ckey, computerid, ip"
    );

    let query = sqlx::query(SQL).bind(ip).bind(ip);

    let mut stream = connection.fetch(query);

    let mut result = Vec::new();

    while let Some(row) = stream.try_next().await? {
        result.push(row.try_into()?);
    }

    Ok(result)
}

pub async fn lookup_player(ckey: &str, pool: &MySqlPool) -> Result<Vec<Lookup>> {
    let mut connection = pool.acquire().await?;

    const SQL: &str = const_format!(
        "SELECT computerid, INET_NTOA(ip) AS ip, ckey FROM connection_log WHERE
         computerid IN (SELECT DISTINCT computerid FROM connection_log WHERE ckey = ?) OR
         ip IN (SELECT DISTINCT ip FROM connection_log WHERE ckey = ?)
         {EXCLUSION_SUBNET} GROUP BY ckey, computerid, ip ORDER BY ckey, computerid, ip"
    );

    let query = sqlx::query(SQL)
        .bind(ckey.to_lowercase())
        .bind(ckey.to_lowercase());

    let mut stream = connection.fetch(query);

    let mut result = Vec::new();

    while let Some(row) = stream.try_next().await? {
        result.push(row.try_into()?);
    }

    Ok(result)
}
