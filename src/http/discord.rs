//! Discord API integration.
//!
//! Provides functions for interacting with the Discord API.
//! Implements rate limiting to comply with Discord's API limits.

use std::collections::HashSet;

use once_cell::sync::Lazy;
use reqwest::StatusCode;
use serde::Deserialize;

use super::{Error, HTTP_CLIENT, Result, TokenBucket};

/// Global token bucket used to rate limit Discord API requests.
///
/// Discord enforces strict global and per-route rate limits. Sending too many
/// concurrent requests can result in HTTP 429 responses or temporary bans.
/// By using a token bucket, we can allow a certain number of requests to be
/// made in a given time period, while queuing excess requests until tokens
/// become available.
///
/// This helps us stay within Discord's rate limits while still allowing
/// some level of concurrency.
///
/// Also each route has its own token bucket to further limit request rates.
///
/// See: <https://discord.com/developers/docs/topics/rate-limits>
static DISCORD_GLOBAL_BUCKET: Lazy<TokenBucket> = Lazy::new(|| TokenBucket::new(50, 1.1));

/// Structure representing an error message returned by the Discord API.
#[derive(Debug, Deserialize)]
struct ErrorMessage {
    /// Discord error code
    ///
    /// See: <https://discord.com/developers/docs/topics/opcodes-and-status-codes#json>
    code: u32,
    /// Human-readable error message
    message: String,
}

/// Represents a Discord user returned by the API.
///
/// See: <https://discord.com/developers/docs/resources/user#user-object>
#[allow(unused)]
#[derive(Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
}

/// Represents a Discord guild member returned by the API.
///
/// See: <https://discord.com/developers/docs/resources/guild#guild-member-object>
#[allow(unused)]
#[derive(Deserialize)]
pub struct GuildMember {
    pub roles: HashSet<String>,
    pub user: User,
}

/// Token bucket for the [`get_guild_member`] Discord API route.
static DISCORD_GET_MEMBER_BUCKET: Lazy<TokenBucket> = Lazy::new(|| TokenBucket::new(5, 1.1));

/// Retrieves information about a guild member from Discord.
///
/// # Arguments
///
/// * `user_id` - Discord user ID
/// * `guild_id` - Our guild (server) ID
/// * `token` - Our bot token
///
/// # Returns
///
/// Guild member object
///
/// # Errors
///
/// Returns an error if the request fails or the user is not a member of the guild
pub async fn get_guild_member(user_id: i64, guild_id: i64, token: &str) -> Result<GuildMember> {
    let _permit = DISCORD_GLOBAL_BUCKET.acquire().await;
    let _permit = DISCORD_GET_MEMBER_BUCKET.acquire().await;

    let response = HTTP_CLIENT
        .get(format!(
            "https://discord.com/api/v10/guilds/{guild_id}/members/{user_id}"
        ))
        .header("Authorization", format!("Bot {token}"))
        .send()
        .await?;

    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        tracing::warn!("received 429 Too Many Requests from Discord API (get_guild_member)");
        return Err(Error::RateLimited);
    }

    let body = response.text().await?;

    let Ok(member) = serde_json::from_str(&body) else {
        let ErrorMessage { code, message } = serde_json::from_str(&body)?;
        return Err(Error::Discord { code, message });
    };

    Ok(member)
}

/// Token bucket for the [`search_members`] Discord API route.
static DISCORD_SEARCH_MEMBER_BUCKET: Lazy<TokenBucket> = Lazy::new(|| TokenBucket::new(10, 10.1));

/// Searches for guild members matching the specified query.
///
/// # Arguments
///
/// * `query` - JSON-encoded search query with filters
/// * `guild_id` - Our guild (server) ID
/// * `token` - Our bot token
///
/// # Returns
///
/// A list of Discord user IDs matching the search criteria
///
/// # Errors
///
/// Returns an error if the request fails or is rate-limited
pub async fn search_members(query: String, guild_id: i64, token: &str) -> Result<Vec<String>> {
    let _permit = DISCORD_GLOBAL_BUCKET.acquire().await;
    let _permit = DISCORD_SEARCH_MEMBER_BUCKET.acquire().await;

    let response = HTTP_CLIENT
        .post(format!(
            "https://discord.com/api/v10/guilds/{guild_id}/members-search"
        ))
        .header("Authorization", format!("Bot {token}"))
        .header("Content-Type", "application/json")
        .body(query.clone())
        .send()
        .await?;

    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        tracing::warn!("received 429 Too Many Requests from Discord API (search_members)");
        return Err(Error::RateLimited);
    }

    let body = response.text().await?;

    #[derive(Deserialize)]
    struct Response {
        members: Vec<SearchMember>,
    }

    #[derive(Deserialize)]
    struct SearchMember {
        member: GuildMember,
    }

    let Ok(res) = serde_json::from_str::<Response>(&body) else {
        let ErrorMessage { code, message } = serde_json::from_str(&body)?;
        return Err(Error::Discord { code, message });
    };

    let members = res.members.into_iter().map(|m| m.member.user.id).collect();

    Ok(members)
}
