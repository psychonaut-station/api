use futures::TryStreamExt;
use poem_openapi::Object;
use sqlx::{Executor as _, MySqlPool, Row as _, mysql::MySqlRow};

use super::{Error, Result, player_exists};

#[derive(Object)]
pub struct PlayerRoletime {
    /// The name of the job
    pub job: String,
    /// The total minutes played in this job
    pub minutes: u32,
}

impl TryFrom<MySqlRow> for PlayerRoletime {
    type Error = sqlx::Error;

    fn try_from(row: MySqlRow) -> std::result::Result<Self, Self::Error> {
        Ok(PlayerRoletime {
            job: row.try_get("job")?,
            minutes: row.try_get("minutes")?,
        })
    }
}

pub async fn get_roletime_player(ckey: &str, pool: &MySqlPool) -> Result<Vec<PlayerRoletime>> {
    let mut connection = pool.acquire().await?;

    let query = sqlx::query(
        "SELECT job, minutes FROM role_time WHERE LOWER(ckey) = ? ORDER BY minutes DESC",
    )
    .bind(ckey.to_lowercase());

    let mut roletimes = Vec::new();

    let mut stream = connection.fetch(query);

    while let Some(row) = stream.try_next().await? {
        roletimes.push(row.try_into()?);
    }

    drop(stream);

    if roletimes.is_empty() && !player_exists(ckey, &mut connection).await? {
        return Err(Error::PlayerNotFound);
    }

    Ok(roletimes)
}

#[derive(Object)]
pub struct JobRoletime {
    /// The ckey of the player
    pub ckey: String,
    /// The total minutes played in this job
    pub minutes: u32,
}

impl TryFrom<MySqlRow> for JobRoletime {
    type Error = sqlx::Error;

    fn try_from(row: MySqlRow) -> std::result::Result<Self, Self::Error> {
        Ok(JobRoletime {
            ckey: row.try_get("ckey")?,
            minutes: row.try_get("minutes")?,
        })
    }
}

pub async fn get_roletime_top(job: &str, pool: &MySqlPool) -> Result<Vec<JobRoletime>> {
    let mut connection = pool.acquire().await?;

    let query = sqlx::query(
        "SELECT ckey, minutes FROM role_time WHERE LOWER(job) = ? ORDER BY minutes DESC LIMIT 15",
    )
    .bind(job.to_lowercase());

    let mut roletimes = Vec::new();

    let mut stream = connection.fetch(query);

    while let Some(row) = stream.try_next().await? {
        roletimes.push(row.try_into()?);
    }

    Ok(roletimes)
}
