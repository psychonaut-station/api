use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::{mysql::MySqlRow, Executor as _, MySqlPool, Row as _};

use super::error::Error;

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

impl TryFrom<MySqlRow> for Ban {
    type Error = sqlx::Error;

    fn try_from(value: MySqlRow) -> Result<Self, Self::Error> {
        Ok(Self {
            bantime: value.try_get("bantime")?,
            round_id: value.try_get("round_id")?,
            roles: value.try_get("roles")?,
            expiration_time: value.try_get("expiration_time")?,
            reason: value.try_get("reason")?,
            ckey: value.try_get("ckey")?,
            a_ckey: value.try_get("a_ckey")?,
            edits: value.try_get("edits")?,
            unbanned_datetime: value.try_get("unbanned_datetime")?,
            unbanned_ckey: value.try_get("unbanned_ckey")?,
        })
    }
}

pub async fn get_ban_by_id(id: u32, pool: &MySqlPool) -> Result<Option<Ban>, Error> {
    let mut connection = pool.acquire().await?;

    let query = sqlx::query(
    	"SELECT id, bantime, round_id, GROUP_CONCAT(role ORDER BY role SEPARATOR ', ') AS roles, expiration_time, reason, ckey, a_ckey, edits, unbanned_datetime, unbanned_ckey FROM ban WHERE id = ?"
    ).bind(id);

    let ban = match connection.fetch_optional(query).await? {
        Some(row) => Some(row.try_into()?),
        None => None,
    };

    connection.close().await?;

    Ok(ban)
}
