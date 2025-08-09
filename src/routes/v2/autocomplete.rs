use rocket::{get, http::Status, post, State};

use crate::{config::Config, database::*, Database};

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
    config: &State<Config>,
    _api_key: ApiKey,
) -> Result<Json<Vec<String>>, Status> {
    let Ok(ckeys) = get_ckeys(ckey, &database.pool, config).await else {
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

#[post("/autocomplete/ckey/hide?<ckey>&<hid_by>")]
pub async fn hide_ckey_autocomplete(
    ckey: &str,
    hid_by: i64,
    database: &State<Database>,
    config: &State<Config>,
    _api_key: ApiKey,
) -> Result<Json<bool>, Status> {
    match hide_ckey(ckey, hid_by, &database.pool, config).await {
        Ok(success) => Ok(Json::Ok(success)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/autocomplete/ckey/unhide?<ckey>&<unhid_by>")]
pub async fn unhide_ckey_autocomplete(
    ckey: &str,
    unhid_by: i64,
    database: &State<Database>,
    config: &State<Config>,
    _api_key: ApiKey,
) -> Result<Json<bool>, Status> {
    match unhide_ckey(ckey, unhid_by, &database.pool, config).await {
        Ok(success) => Ok(Json::Ok(success)),
        Err(_) => Err(Status::InternalServerError),
    }
}
