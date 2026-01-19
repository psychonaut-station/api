//! Role time tracking queries.
//!
//! Provides functions to query player role times (time spent playing specific jobs)
//! and job leaderboards.

use poem_openapi::Object;
use sqlx::{FromRow, MySqlPool, Row as _, mysql::MySqlRow};

use super::{Error, Result, player_exists};

/// Represents a player's roletime for a specific job.
#[derive(Object)]
pub struct PlayerRoletime {
    /// The name of the job
    pub job: String,
    /// The total minutes played in this job
    pub minutes: u32,
}

impl FromRow<'_, MySqlRow> for PlayerRoletime {
    fn from_row(row: &MySqlRow) -> sqlx::Result<Self> {
        Ok(PlayerRoletime {
            job: row.try_get("job")?,
            minutes: row.try_get("minutes")?,
        })
    }
}

/// Retrieves a player's roletime statistics for all jobs.
///
/// # Arguments
///
/// * `ckey` - Player's ckey (case-insensitive)
/// * `pool` - Database connection pool
///
/// # Returns
///
/// A list of job roletimes ordered by minutes played (highest first)
///
/// # Errors
///
/// Returns `Error::PlayerNotFound` if the player doesn't exist
pub async fn get_roletime_player(ckey: &str, pool: &MySqlPool) -> Result<Vec<PlayerRoletime>> {
    let query = sqlx::query_as(
        "SELECT job, minutes FROM role_time WHERE LOWER(ckey) = ? ORDER BY minutes DESC",
    )
    .bind(ckey.to_lowercase());

    let roletimes = query.fetch_all(pool).await?;

    if roletimes.is_empty() && !player_exists(ckey, pool).await? {
        return Err(Error::PlayerNotFound);
    }

    Ok(roletimes)
}

/// Represents a job's roletime for a specific player.
#[derive(Object)]
pub struct JobRoletime {
    /// The ckey of the player
    pub ckey: String,
    /// The total minutes played in this job
    pub minutes: u32,
}

impl FromRow<'_, MySqlRow> for JobRoletime {
    fn from_row(row: &MySqlRow) -> sqlx::Result<Self> {
        Ok(JobRoletime {
            ckey: row.try_get("ckey")?,
            minutes: row.try_get("minutes")?,
        })
    }
}

/// Retrieves the top 15 players by roletime for a specific job.
///
/// # Arguments
///
/// * `job` - Job name to query (case-insensitive)
/// * `pool` - Database connection pool
///
/// # Returns
///
/// A list of up to 15 players ordered by minutes played (highest first)
pub async fn get_roletime_top(job: &str, pool: &MySqlPool) -> Result<Vec<JobRoletime>> {
    let query = sqlx::query_as(
        "SELECT ckey, minutes FROM role_time WHERE LOWER(job) = ? ORDER BY minutes DESC LIMIT 15",
    )
    .bind(job.to_lowercase());

    Ok(query.fetch_all(pool).await?)
}
