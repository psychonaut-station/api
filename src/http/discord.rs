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
/// See: https://discord.com/developers/docs/topics/rate-limits
static DISCORD_GLOBAL_BUCKET: Lazy<TokenBucket> = Lazy::new(|| TokenBucket::new(50, 1.1));

#[derive(Debug, Deserialize)]
struct ErrorMessage {
    /// https://discord.com/developers/docs/topics/opcodes-and-status-codes#json
    code: u32,
    message: String,
}

#[allow(unused)]
#[derive(Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
}

/// [guild-member-object](https://discord.com/developers/docs/resources/guild#guild-member-object)
#[allow(unused)]
#[derive(Deserialize)]
pub struct GuildMember {
    pub roles: HashSet<String>,
    pub user: User,
}

static DISCORD_GET_MEMBER_BUCKET: Lazy<TokenBucket> = Lazy::new(|| TokenBucket::new(5, 1.1));

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

static DISCORD_SEARCH_MEMBER_BUCKET: Lazy<TokenBucket> = Lazy::new(|| TokenBucket::new(10, 10.1));

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
