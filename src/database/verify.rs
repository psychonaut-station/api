//! Discord verification and Patreon status queries.
//!
//! Handles queries related to Discord account linking and Patreon supporter verification.
//! Integrates with the Discord API to check roles and membership.

use futures::TryStreamExt as _;
use sqlx::{MySqlPool, Row as _};

use crate::{
    config::DiscordConfig,
    http::{
        self,
        discord::{get_guild_member, search_members},
    },
};

use super::{Error, Result, ckey_from_user_id, user_id_from_ckey};

/// Verifies a Discord account with a one-time token and links it to a ckey.
///
/// Checks if the Discord ID is already linked to another account, then validates
/// the OTP token. If valid, links the Discord ID to the associated ckey.
///
/// # Arguments
///
/// * `id` - Discord user ID to link
/// * `otp` - One-time token (valid for 4 hours)
/// * `pool` - Database connection pool
///
/// # Returns
///
/// The linked ckey on success.
///
/// # Errors
///
/// Returns an error if:
/// - The Discord ID is already linked to another account
/// - The OTP token is invalid, expired, or already used
/// - A database error occurred
pub async fn verify_with_otp(id: i64, otp: &str, pool: &MySqlPool) -> Result<String> {
    if let Some(ckey) = ckey_from_user_id(id, pool).await? {
        return Err(Error::DiscordInUse(ckey));
    }

    let query = sqlx::query(
        "SELECT id, ckey, FROM discord_links WHERE valid = 0 AND discord_id IS NULL AND timestamp >= NOW() - INTERVAL 4 HOUR AND one_time_token = ? LIMIT 1"
    ).bind(otp);

    match query.fetch_optional(pool).await? {
        Some(row) => {
            let idx: i32 = row.try_get("id")?;
            let ckey: String = row.try_get("ckey")?;

            let query = sqlx::query(
                "UPDATE discord_links SET discord_id = ?, valid = 1 WHERE id = ? LIMIT 1",
            )
            .bind(id)
            .bind(idx);

            query.execute(pool).await?;

            Ok(ckey)
        }
        _ => Err(Error::TokenInvalid),
    }
}

/// Retrieves all patron ckeys from Discord and matches them with linked accounts.
///
/// Queries Discord API for members with the patreon role, then looks up their
/// linked ckeys in the database.
///
/// # Arguments
///
/// * `pool` - Database connection pool.
/// * `config` - Application configuration containing Discord credentials.
///
/// # Returns
///
/// A list of ckeys belonging to verified patrons.
pub async fn get_patrons(pool: &MySqlPool, config: &DiscordConfig) -> Result<Vec<String>> {
    let query = format!(
        r#"{{"or_query":{{}},"and_query":{{"role_ids":{{"and_query":["{}"]}}}},"limit":1000}}"#,
        config.patreon_role
    );

    let members = search_members(query, config.guild, &config.token).await?;

    if members.is_empty() {
        return Ok(Vec::new());
    }

    let sql = format!(
        "SELECT ckey FROM discord_links WHERE discord_id IN ({}) AND valid = 1",
        vec!["?"; members.len()].join(",")
    );

    let mut query = sqlx::query(&sql);

    for id in &members {
        query = query.bind(id);
    }

    let mut ckeys = Vec::with_capacity(members.len());

    let mut stream = query.fetch(pool);

    while let Some(row) = stream.try_next().await? {
        ckeys.push(row.try_get("ckey")?);
    }

    Ok(ckeys)
}

/// Checks if a player is a patron by verifying their Discord role.
///
/// Looks up the player's linked Discord account and checks if they have
/// the patreon role on the Discord server.
///
/// # Arguments
///
/// * `ckey` - Player's ckey to check.
/// * `pool` - Database connection pool.
/// * `config` - Application configuration containing Discord credentials.
///
/// # Returns
///
/// `true` if the player is a patron, `false` otherwise.
pub async fn is_patron(ckey: &str, pool: &MySqlPool, config: &DiscordConfig) -> Result<bool> {
    let Some(id) = user_id_from_ckey(ckey, pool).await? else {
        return Ok(false);
    };

    match get_guild_member(id, config.guild, &config.token).await {
        Ok(member) => {
            let role = config.patreon_role.to_string();
            Ok(member.roles.contains(&role))
        }
        // Unknown member | Unknown user
        Err(http::Error::Discord {
            code: 10007 | 10013,
            ..
        }) => Ok(false),
        Err(e) => Err(e)?,
    }
}
