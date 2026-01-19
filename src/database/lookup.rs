//! Connection lookup queries.
//!
//! Provides functions to look up related connections by computer ID, IP address,
//! or player ckey. Used for administrative investigation and ban evasion detection.

use const_format::formatcp as const_format;
use poem_openapi::Object;
use sqlx::{FromRow, MySqlPool, Row as _, mysql::MySqlRow};

use super::Result;

/// SQL fragment to exclude Cloudflare IP ranges from lookup results.
const EXCLUSION_SUBNET: &str =
    " AND INET_NTOA(ip & INET_ATON('255.255.0.0')) NOT IN ('104.28.0.0')";

/// Represents a row for lookup results.
#[derive(Object)]
pub struct Lookup {
    /// The computer ID of the player
    pub computerid: String,
    /// The IP address of the player
    pub ip: String,
    /// The ckey of the player
    pub ckey: String,
}

impl FromRow<'_, MySqlRow> for Lookup {
    fn from_row(row: &MySqlRow) -> sqlx::Result<Self> {
        Ok(Lookup {
            computerid: row.try_get("computerid")?,
            ip: row.try_get("ip")?,
            ckey: row.try_get("ckey")?,
        })
    }
}

/// Looks up all connections associated with a computer ID.
///
/// Finds all IP addresses and ckeys that have been used with the same
/// computer ID or by the same players.
///
/// # Arguments
///
/// * `cid` - Computer ID to search for
/// * `pool` - Database connection pool
///
/// # Returns
///
/// A list of related connections grouped by ckey, computer ID, and IP
pub async fn lookup_cid(cid: &str, pool: &MySqlPool) -> Result<Vec<Lookup>> {
    const SQL: &str = const_format!(
        "SELECT computerid, INET_NTOA(ip) AS ip, ckey FROM connection_log WHERE
         ip IN (SELECT DISTINCT ip FROM connection_log WHERE computerid = ?) OR
         ckey IN (SELECT DISTINCT ckey FROM connection_log WHERE computerid = ?)
         {EXCLUSION_SUBNET} GROUP BY ckey, computerid, ip ORDER BY ckey, computerid, ip"
    );

    let query = sqlx::query_as(SQL).bind(cid).bind(cid);

    Ok(query.fetch_all(pool).await?)
}

/// Looks up all connections associated with an IP address.
///
/// Finds all computer IDs and ckeys that have been used with the same
/// IP address or by the same players.
///
/// # Arguments
///
/// * `ip` - IP address to search for
/// * `pool` - Database connection pool
///
/// # Returns
///
/// A list of related connections grouped by ckey, computer ID, and IP
pub async fn lookup_ip(ip: &str, pool: &MySqlPool) -> Result<Vec<Lookup>> {
    const SQL: &str = const_format!(
        "SELECT computerid, INET_NTOA(ip) AS ip, ckey FROM connection_log WHERE
         computerid IN (SELECT DISTINCT computerid FROM connection_log WHERE ip = INET_ATON(?)) OR
         ckey IN (SELECT DISTINCT ckey FROM connection_log WHERE ip = INET_ATON(?))
         {EXCLUSION_SUBNET} GROUP BY ckey, computerid, ip ORDER BY ckey, computerid, ip"
    );

    let query = sqlx::query_as(SQL).bind(ip).bind(ip);

    Ok(query.fetch_all(pool).await?)
}

/// Looks up all connections associated with a player's ckey.
///
/// Finds all computer IDs and IP addresses that have been used by the
/// specified player or by others sharing their connection details.
///
/// # Arguments
///
/// * `ckey` - Player's ckey to search for
/// * `pool` - Database connection pool
///
/// # Returns
///
/// A list of related connections grouped by ckey, computer ID, and IP
pub async fn lookup_player(ckey: &str, pool: &MySqlPool) -> Result<Vec<Lookup>> {
    const SQL: &str = const_format!(
        "SELECT computerid, INET_NTOA(ip) AS ip, ckey FROM connection_log WHERE
         computerid IN (SELECT DISTINCT computerid FROM connection_log WHERE ckey = ?) OR
         ip IN (SELECT DISTINCT ip FROM connection_log WHERE ckey = ?)
         {EXCLUSION_SUBNET} GROUP BY ckey, computerid, ip ORDER BY ckey, computerid, ip"
    );

    let query = sqlx::query_as(SQL).bind(ckey).bind(ckey);

    Ok(query.fetch_all(pool).await?)
}
