use rocket::{get, http::Status, State};

use crate::{database::*, Database};

use super::{common::ApiKey, Json};

#[get("/ban?<id>")]
pub async fn index(
    id: u32,
    database: &State<Database>,
    _api_key: ApiKey,
) -> Result<Json<Ban>, Status> {
    match get_ban_by_id(id, &database.pool).await {
        Ok(Some(ban)) => Ok(Json::Ok(ban)),
        Ok(None) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}
