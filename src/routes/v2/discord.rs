use rocket::{get, http::Status, State};

use crate::{
    config::Config,
    http::{
        self,
        discord::{self, GuildMember, User},
    },
};

use super::{common::ApiKey, Json};

#[get("/discord/user?<discord_id>")]
pub async fn user(
    discord_id: &str,
    config: &State<Config>,
    _api_key: ApiKey,
) -> Result<Json<User>, Status> {
    let Ok(id) = discord_id.parse::<i64>() else {
        return Err(Status::BadRequest);
    };

    match discord::get_user(id, &config.discord.token).await {
        Ok(user) => Ok(Json::Ok(user)),
        Err(http::Error::Discord(code)) => match code {
            10013 => Err(Status::NotFound),
            _ => Err(Status::InternalServerError),
        },
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/discord/member?<discord_id>")]
pub async fn member(
    discord_id: &str,
    config: &State<Config>,
    _api_key: ApiKey,
) -> Result<Json<GuildMember>, Status> {
    let Ok(id) = discord_id.parse::<i64>() else {
        return Err(Status::BadRequest);
    };

    match discord::get_guild_member(config.discord.guild, id, &config.discord.token).await {
        Ok(member) => Ok(Json::Ok(member)),
        Err(http::Error::Discord(code)) => match code {
            10007 | 10013 => Err(Status::NotFound),
            _ => Err(Status::InternalServerError),
        },
        Err(_) => Err(Status::InternalServerError),
    }
}
