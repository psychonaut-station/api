use rocket::{get, http::Status, post, State};
use serde_json::{json, Value};

use crate::{
    config::Config,
    database::{error::Error, *},
    Database,
};

use super::{common::ApiKey, Json};

#[get("/player?<ckey>")]
pub async fn index(
    ckey: &str,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Player>, Status> {
    match get_player(ckey, &database.pool).await {
        Ok(player) => Ok(Json::Ok(player)),
        Err(Error::PlayerNotFound) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/player/ban?<ckey>&<permanent>&<since>")]
pub async fn ban(
    ckey: &str,
    permanent: Option<bool>,
    since: Option<&str>,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Vec<Ban>>, Status> {
    match get_ban(ckey, permanent.unwrap_or(false), since, &database.pool).await {
        Ok(bans) => Ok(Json::Ok(bans)),
        Err(Error::PlayerNotFound) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/player/characters?<ckey>")]
pub async fn characters(
    ckey: &str,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Vec<(String, i64)>>, Status> {
    match get_characters(ckey, &database.pool).await {
        Ok(characters) => Ok(Json::Ok(characters)),
        Err(Error::PlayerNotFound) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/player/roletime?<ckey>")]
pub async fn roletime(
    ckey: &str,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Vec<PlayerRoletime>>, Status> {
    match get_roletime(ckey, &database.pool).await {
        Ok(roletimes) => Ok(Json::Ok(roletimes)),
        Err(Error::PlayerNotFound) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/player/roletime/top?<job>")]
pub async fn top(
    job: &str,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Vec<JobRoletime>>, Status> {
    let Ok(roletimes) = get_top_roletime(job, &database.pool).await else {
        return Err(Status::InternalServerError);
    };

    Ok(Json::Ok(roletimes))
}

#[get("/player/activity?<ckey>")]
pub async fn activity(
    ckey: &str,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Vec<(String, i64)>>, Status> {
    match get_activity(ckey, &database.pool).await {
        Ok(activity) => Ok(Json::Ok(activity)),
        Err(Error::PlayerNotFound) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/player/discord?<ckey>&<discord_id>")]
pub async fn discord(
    ckey: Option<&str>,
    discord_id: Option<&str>,
    database: &State<Database>,
    config: &State<Config>,
    _api_key: ApiKey,
) -> Result<Json<Value>, Status> {
    if ckey.is_some() ^ discord_id.is_none() {
        return Err(Status::BadRequest);
    }

    if let Some(ckey) = ckey {
        return match fetch_discord_by_ckey(ckey, &config.discord.token, &database.pool).await {
            Ok(user) => Ok(Json::Ok(json!(user))),
            Err(Error::PlayerNotFound) => Err(Status::NotFound),
            Err(Error::NotLinked) => Err(Status::Conflict),
            Err(_) => Err(Status::InternalServerError),
        };
    } else if let Some(discord_id) = discord_id {
        return match get_ckey_by_discord_id(discord_id, &database.pool).await {
            Ok(ckey) => Ok(Json::Ok(Value::String(ckey))),
            Err(Error::NotLinked) => Err(Status::Conflict),
            Err(_) => Err(Status::InternalServerError),
        };
    }

    unreachable!()
}

#[get("/player/achievements?<ckey>&<achievement_type>")]
pub async fn achievements(
    ckey: &str,
    achievement_type: Option<&str>,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Value>, Status> {
    match get_achievements(ckey, achievement_type, &database.pool).await {
        Ok(achievements) => Ok(Json::Ok(json!(achievements))),
        Err(Error::PlayerNotFound) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/player/favorite_character?<ckey>")]
pub async fn fav_character(
    ckey: &str,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<(String, String)>, Status> {
    match get_favorite_character(ckey, &database.pool).await {
        Ok(data) => Ok(Json::Ok(data)),
        Err(Error::PlayerNotFound) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/player/tickets?<ckey>&<fetch_size>&<page>")]
pub async fn tickets(
    ckey: &str,
    fetch_size: Option<i32>,
    page: Option<i32>,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Value>, Status> {
    match get_tickets(ckey, fetch_size, page, &database.pool).await {
        Ok((tickets, total_count)) => Ok(Json::Ok(json!({
            "data": tickets,
            "total_count": total_count
        }))),
        Err(Error::PlayerNotFound) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/player/messages?<ckey>&<fetch_size>&<page>")]
pub async fn messages(
    ckey: &str,
    fetch_size: Option<i32>,
    page: Option<i32>,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Value>, Status> {
    match get_messages(ckey, fetch_size, page, &database.pool).await {
        Ok((messages, total_count)) => Ok(Json::Ok(json!({
            "data": messages,
            "total_count": total_count
        }))),
        Err(Error::PlayerNotFound) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/player/notes?<ckey>&<fetch_size>&<page>")]
pub async fn notes(
    ckey: &str,
    fetch_size: Option<i32>,
    page: Option<i32>,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Value>, Status> {
    match get_notes(ckey, fetch_size, page, &database.pool).await {
        Ok((notes, total_count)) => Ok(Json::Ok(json!({
            "data": notes,
            "total_count": total_count
        }))),
        Err(Error::PlayerNotFound) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/player/rounds?<ckey>&<fetch_size>&<page>")]
pub async fn rounds(
    ckey: &str,
    fetch_size: Option<i32>,
    page: Option<i32>,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Value>, Status> {
    match get_player_rounds(ckey, fetch_size, page, &database.pool).await {
        Ok((rounds, total_count)) => Ok(Json::Ok(json!({
            "data": rounds,
            "total_count": total_count
        }))),
        Err(Error::PlayerNotFound) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/player/friends?<ckey>")]
pub async fn friends(
    ckey: &str,
    config: &State<Config>,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Vec<Friendship>>, Status> {
    match get_friends(ckey, &database.pool, config).await {
        Ok(friends) => Ok(Json::Ok(friends)),
        Err(Error::PlayerNotFound) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/player/friend_invites?<ckey>")]
pub async fn friend_invites(
    ckey: &str,
    config: &State<Config>,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Value>, Status> {
    match get_friendship_invites(ckey, &database.pool, config).await {
        Ok((received, sent)) => Ok(Json::Ok(json!({
            "received": received,
            "sent": sent
        }))),
        Err(Error::PlayerNotFound) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/player/check_friends?<ckey>&<friend>")]
pub async fn check_friends(
    ckey: &str,
    friend: &str,
    config: &State<Config>,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Option<Friendship>>, Status> {
    match check_friendship(ckey, friend, &database.pool, config).await {
        Ok(friend) => Ok(Json::Ok(friend)),
        Err(Error::PlayerNotFound) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/player/add_friend?<ckey>&<friend>")]
pub async fn addfriend(
    ckey: &str,
    friend: &str,
    config: &State<Config>,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Option<Friendship>>, Status> {
    match add_friend(ckey, friend, &database.pool, config).await {
        Ok(friend) => Ok(Json::Ok(friend)),
        Err(Error::PlayerNotFound) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/player/remove_friend?<ckey>&<friendship_id>")]
pub async fn removefriend(
    ckey: &str,
    friendship_id: i32,
    config: &State<Config>,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Option<Friendship>>, Status> {
    match remove_friend(ckey, friendship_id, &database.pool, config).await {
        Ok(friend) => Ok(Json::Ok(friend)),
        Err(Error::PlayerNotFound) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/player/accept_friend?<ckey>&<friendship_id>")]
pub async fn acceptfriend(
    ckey: &str,
    friendship_id: i32,
    config: &State<Config>,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Option<Friendship>>, Status> {
    match accept_friend(ckey, friendship_id, &database.pool, config).await {
        Ok(friend) => Ok(Json::Ok(friend)),
        Err(Error::PlayerNotFound) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/player/decline_friend?<ckey>&<friendship_id>")]
pub async fn declinefriend(
    ckey: &str,
    friendship_id: i32,
    config: &State<Config>,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Option<Friendship>>, Status> {
    match decline_friend(ckey, friendship_id, &database.pool, config).await {
        Ok(friend) => Ok(Json::Ok(friend)),
        Err(Error::PlayerNotFound) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/player/lookup?<ckey>&<ip>&<cid>")]
pub async fn lookup(
    ckey: Option<&str>,
    ip: Option<&str>,
    cid: Option<i64>,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Value>, Status> {
    if ckey.is_none() && ip.is_none() && cid.is_none() {
        return Err(Status::BadRequest);
    }

    match lookup_player(ckey, ip, cid, &database.pool).await {
        Ok(result) => Ok(Json::Ok(json!(result))),
        Err(_) => Err(Status::InternalServerError),
    }
}
