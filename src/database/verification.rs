use futures::TryStreamExt as _;
use sqlx::{MySqlPool, Row as _};

use crate::{
    config::Config,
    http::{
        self,
        discord::{get_guild_member, search_members},
    },
};

use super::Result;

pub async fn get_patrons(pool: &MySqlPool, config: &Config) -> Result<Vec<String>> {
    let query = format!(
        r#"{{"or_query":{{}},"and_query":{{"role_ids":{{"and_query":["{}"]}}}},"limit":1000}}"#,
        config.discord.patreon_role
    );

    let members = search_members(query, config.discord.guild, &config.discord.token).await?;

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

pub async fn is_patron(ckey: &str, pool: &MySqlPool, config: &Config) -> Result<bool> {
    let Some(id) = discord_id_from_ckey(ckey, pool).await? else {
        return Ok(false);
    };

    match get_guild_member(id, config.discord.guild, &config.discord.token).await {
        Ok(member) => {
            let role = config.discord.patreon_role.to_string();
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

pub async fn discord_id_from_ckey(ckey: &str, pool: &MySqlPool) -> Result<Option<i64>> {
    let query = sqlx::query(
        "SELECT discord_id FROM discord_links WHERE LOWER(ckey) = ? AND valid = 1 LIMIT 1",
    )
    .bind(ckey.to_lowercase());

    match query.fetch_optional(pool).await? {
        Some(row) => Ok(Some(row.try_get("discord_id")?)),
        None => Ok(None),
    }
}
