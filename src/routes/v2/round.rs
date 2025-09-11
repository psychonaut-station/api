use rocket::{get, http::Status as HttpStatus, State};

use serde_json::{json, Value};

use crate::{
    config::Config,
    database::{error::Error, *},
    Database,
};

use super::{common::ApiKey, Json};

#[get("/round?<round_id>")]
pub async fn index(
    round_id: i32,
    database: &State<Database>,
    config: &State<Config>,
    _api_key: ApiKey
) -> Result<Json<RoundData>, HttpStatus> {
    match get_round(round_id, config, &database.pool).await {
        Ok(round) => Ok(Json::Ok(round)),
        Err(Error::RoundNotFound) => Err(HttpStatus::NotFound),
        Err(_) => Err(HttpStatus::InternalServerError),
    }
}

#[get("/rounds?<fetch_size>&<page>&<round_id>")]
pub async fn rounds(
    fetch_size: Option<i32>,
    page: Option<i32>,
    round_id: Option<i32>,
    config: &State<Config>,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Value>, HttpStatus> {
    match get_rounds(fetch_size, page, round_id, config, &database.pool).await {
        Ok((rounds, total_count)) => Ok(Json::Ok(json!({
            "data": rounds,
            "total_count": total_count
        }))),
        Err(_) => Err(HttpStatus::InternalServerError),
    }
}
