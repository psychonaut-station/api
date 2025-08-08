use rocket::{get, http::Status, post, serde::json, State};
use serde::Deserialize;

use crate::{database::*, Database};

use super::{common::ApiKey, Json};

#[get("/autocomplete/job?<job>")]
pub async fn job(
    job: &str,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Vec<String>>, Status> {
    let Ok(jobs) = get_jobs(job, &database.pool).await else {
        return Err(Status::InternalServerError);
    };

    Ok(Json::Ok(jobs))
}

#[get("/autocomplete/ckey?<ckey>")]
pub async fn ckey(
    ckey: &str,
    database: &State<Database>,
    api_database: &State<ServerDatabase>,
    _api_key: ApiKey,
) -> Result<Json<Vec<String>>, Status> {
    let Ok(ckeys) = get_ckeys(ckey, &database.pool, &api_database.0.pool).await else {
        return Err(Status::InternalServerError);
    };

    Ok(Json::Ok(ckeys))
}

#[get("/autocomplete/ic_name?<ic_name>")]
pub async fn ic_name(
    ic_name: &str,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Vec<IcName>>, Status> {
    let Ok(ic_names) = get_ic_names(ic_name, &database.pool).await else {
        return Err(Status::InternalServerError);
    };

    Ok(Json::Ok(ic_names))
}

#[derive(Deserialize)]
pub struct IgnoreData<'r> {
    ckey: &'r str,
}

#[post("/autocomplete/ckey/ignore", data = "<data>")]
pub async fn ignore_autocomplete(
    data: json::Json<IgnoreData<'_>>,
    database: &State<ServerDatabase>,
    _api_key: ApiKey,
) -> Result<Json<String>, Status> {
    match ignore_ckey(data.ckey, &database.0.pool).await {
        Ok(account) => Ok(Json::Ok(account)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/autocomplete/ckey/unignore", data = "<data>")]
pub async fn unignore_autocomplete(
    data: json::Json<IgnoreData<'_>>,
    database: &State<ServerDatabase>,
    _api_key: ApiKey,
) -> Result<Json<String>, Status> {
    match unignore_ckey(data.ckey, &database.0.pool).await {
        Ok(account) => Ok(Json::Ok(account)),
        Err(_) => Err(Status::InternalServerError),
    }
}