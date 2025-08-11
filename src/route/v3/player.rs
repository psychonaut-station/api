use poem::web::Data;
use poem_openapi::{
    ApiResponse, OpenApi,
    param::Path,
    payload::{Json, PlainText},
};
use sqlx::MySqlPool;
use tracing::error;

use crate::database::{
    Error as DatabaseError,
    player::{Player, get_player},
};

pub struct Endpoint;

#[OpenApi]
impl Endpoint {
    #[oai(path = "/player/:ckey", method = "get")]
    async fn player(&self, ckey: Path<String>, pool: Data<&MySqlPool>) -> PlayerResponse {
        match get_player(&ckey, *pool).await {
            Ok(player) => PlayerResponse::Success(Json(player)),
            Err(e) => match e {
                DatabaseError::PlayerNotFound => PlayerResponse::NotFound(e.into()),
                _ => {
                    error!("Error fetching player `{}`: {e:?}", *ckey);
                    PlayerResponse::InternalError(e.into())
                }
            },
        }
    }
}

#[derive(ApiResponse)]
enum PlayerResponse {
    #[oai(status = 200)]
    Success(Json<Player>),
    #[oai(status = 404)]
    NotFound(PlainText<String>),
    #[oai(status = 500)]
    InternalError(PlainText<String>),
}
