use rocket::{get, http::Status, State};
use serde_json::{json, Value};

use crate::{database::*, Config, Database};

use super::{common::ApiKey, Json};

#[get("/events/overview?<limit>")]
pub async fn overview(
    limit: Option<i32>,
    config: &State<Config>,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Vec<Overview>>, Status> {
    match get_overview(limit.unwrap_or(1), config, &database.pool).await {
        Ok(overview) => Ok(Json::Ok(overview)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/events/deaths?<fetch_size>&<page>")]
pub async fn deaths(
    fetch_size: Option<i32>,
    page: Option<i32>,
    config: &State<Config>,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Value>, Status> {
    match get_deaths(fetch_size, page, config, &database.pool).await {
        Ok((deaths, total_count)) => Ok(Json::Ok(json!({
            "data": deaths,
            "total_count": total_count
        }))),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/events/citations?<fetch_size>&<page>")]
pub async fn citations(
    fetch_size: Option<i32>,
    page: Option<i32>,
    config: &State<Config>,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Value>, Status> {
    match get_citations(fetch_size, page, config, &database.pool).await {
        Ok((citations, total_count)) => Ok(Json::Ok(json!({
            "data": citations,
            "total_count": total_count
        }))),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/events/crimes?<fetch_size>&<page>")]
pub async fn crimes(
    fetch_size: Option<i32>,
    page: Option<i32>,
    config: &State<Config>,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Value>, Status> {
    match get_crimes(fetch_size, page, config, &database.pool).await {
        Ok((crimes, total_count)) => Ok(Json::Ok(json!({
            "data": crimes,
            "total_count": total_count
        }))),
        Err(_) => Err(Status::InternalServerError),
    }
}
