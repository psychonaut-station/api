use futures::TryStreamExt;
use poem_openapi::Object;
use sqlx::{Executor as _, MySqlPool, Row as _};

use super::Result;

#[derive(Object)]
pub struct Lookup {
    /// The computer ID of the player
    pub computerid: String,
    /// The IP address of the player
    pub ip: String,
    /// The ckey of the player
    pub ckey: String,
}

pub async fn lookup_cid(cid: &str, pool: &MySqlPool) -> Result<Vec<Lookup>> {
    let mut connection = pool.acquire().await?;

    let query = sqlx::query(
        "SELECT computerid, INET_NTOA(ip) AS ip, ckey FROM connection_log WHERE ip IN (SELECT DISTINCT ip FROM connection_log WHERE computerid = ?) OR
             ckey IN (SELECT DISTINCT ckey FROM connection_log WHERE computerid = ?) GROUP BY ckey, computerid, ip ORDER BY ckey, computerid, ip"
    ).bind(cid).bind(cid);

    let mut rows = connection.fetch(query);

    let mut result = Vec::new();

    while let Some(row) = rows.try_next().await? {
        result.push(Lookup {
            computerid: row.try_get("computerid")?,
            ip: row.try_get("ip")?,
            ckey: row.try_get("ckey")?,
        });
    }

    Ok(result)
}

pub async fn lookup_ip(ip: &str, pool: &MySqlPool) -> Result<Vec<Lookup>> {
    let mut connection = pool.acquire().await?;

    let query = sqlx::query(
        "SELECT computerid, INET_NTOA(ip) AS ip, ckey FROM connection_log WHERE computerid IN (SELECT DISTINCT computerid FROM connection_log WHERE ip = INET_ATON(?)) OR
            ckey IN (SELECT DISTINCT ckey FROM connection_log WHERE ip = INET_ATON(?)) GROUP BY ckey, computerid, ip ORDER BY ckey, computerid, ip"
    ).bind(ip).bind(ip);

    let mut rows = connection.fetch(query);

    let mut result = Vec::new();

    while let Some(row) = rows.try_next().await? {
        result.push(Lookup {
            computerid: row.try_get("computerid")?,
            ip: row.try_get("ip")?,
            ckey: row.try_get("ckey")?,
        });
    }

    Ok(result)
}

pub async fn lookup_player(ckey: &str, pool: &MySqlPool) -> Result<Vec<Lookup>> {
    let mut connection = pool.acquire().await?;

    let query = sqlx::query(
        "SELECT computerid, INET_NTOA(ip) AS ip, ckey FROM connection_log WHERE computerid IN (SELECT DISTINCT computerid FROM connection_log WHERE ckey = ?) OR
            ip IN (SELECT DISTINCT ip FROM connection_log WHERE ckey = ?) GROUP BY ckey, computerid, ip ORDER BY ckey, computerid, ip"
    ).bind(ckey.to_lowercase()).bind(ckey.to_lowercase());

    let mut rows = connection.fetch(query);

    let mut result = Vec::new();

    while let Some(row) = rows.try_next().await? {
        result.push(Lookup {
            computerid: row.try_get("computerid")?,
            ip: row.try_get("ip")?,
            ckey: row.try_get("ckey")?,
        });
    }

    Ok(result)
}
