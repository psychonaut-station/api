use rocket::{get, http::Status, State};
use serde_json::{json, Value};
use sqlx::MySqlPool;

use crate::{
    config::{self, Config},
    database::{error::Error, *},
    http::{self, discord::get_guild_member},
    Database,
};

use super::{common::ApiKey, Json};

#[get("/patreon?<ckey>")]
pub async fn index(
    ckey: &str,
    database: &State<Database>,
    config: &State<Config>,
    _api_key: ApiKey,
) -> Result<Json<Value>, Status> {
    let Ok(patron) = is_patron(ckey, &database.pool, &config.discord).await else {
        return Err(Status::InternalServerError);
    };

    Ok(Json::Ok(json!({ "patron": patron })))
}

async fn is_patron(ckey: &str, pool: &MySqlPool, discord: &config::Discord) -> Result<bool, Error> {
    let mut connection = pool.acquire().await?;

    let Ok(discord_id) = discord_id_by_ckey(ckey, &mut connection).await else {
        return Ok(false);
    };

    connection.close().await?;

    let member = match get_guild_member(discord.guild, discord_id, &discord.token).await {
        Ok(member) => member,
        Err(http::Error::Discord(code)) => match code {
            10007 | 10013 => return Ok(false),
            _ => return Err(http::Error::Discord(code))?,
        },
        Err(e) => return Err(e)?,
    };

    Ok(member.roles.contains(&discord.patreon_role.to_string()))
}
