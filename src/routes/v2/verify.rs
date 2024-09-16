use rocket::{http::Status, post, serde::json, State};
use serde::Deserialize;

use crate::{
    database::{error::Error, *},
    Database,
};

use super::{common::ApiKey, Json};

#[derive(Deserialize)]
pub struct VerifyData<'r> {
    discord_id: &'r str,
    one_time_token: Option<&'r str>,
    ckey: Option<&'r str>,
    skip_ckey: Option<bool>,
}

#[post("/verify", data = "<data>")]
pub async fn index(
    data: json::Json<VerifyData<'_>>,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Option<String>>, Status> {
    if data.one_time_token.is_some() ^ data.ckey.is_none() {
        return Err(Status::BadRequest);
    }

    match verify_discord(
        data.discord_id,
        data.one_time_token,
        data.ckey,
        data.skip_ckey,
        &database.pool,
    )
    .await
    {
        Ok(ckey) => Ok(Json::Ok(ckey)),
        Err(Error::DiscordInUse(ckey)) => Ok(Json::Conflict(Some(ckey))),
        Err(Error::CkeyInUse(discord_id)) => Ok(Json::Conflict(Some(format!("@{discord_id}")))),
        Err(Error::TokenInvalid) => Err(Status::NotFound),
        Err(Error::PlayerNotFound) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[derive(Deserialize)]
pub struct UnverifyData<'r> {
    discord_id: Option<&'r str>,
    ckey: Option<&'r str>,
}

#[post("/unverify", data = "<data>")]
pub async fn unverify(
    data: json::Json<UnverifyData<'_>>,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<String>, Status> {
    if data.discord_id.is_some() ^ data.ckey.is_none() {
        return Err(Status::BadRequest);
    }

    match unverify_discord(data.discord_id, data.ckey, &database.pool).await {
        Ok(account) => Ok(Json::Ok(account)),
        Err(Error::PlayerNotFound) => Err(Status::NotFound),
        Err(Error::NotLinked) => Err(Status::Conflict),
        Err(_) => Err(Status::InternalServerError),
    }
}
