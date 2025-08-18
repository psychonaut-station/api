use poem::web::Data;
use poem_openapi::{
    ApiResponse, OpenApi,
    param::Path,
    payload::{Json, PlainText},
};
use sqlx::MySqlPool;
use tracing::error;

use crate::database::{Error as DatabaseError, PlayerRoletime, get_roletime_player};

pub struct Endpoint;

#[OpenApi]
impl Endpoint {
    /// /v3/roletime/player/{ckey}
    ///
    /// Retrieves the minutes played in each job for a player.
    #[oai(path = "/roletime/player/:ckey", method = "get")]
    async fn roletime_player(
        &self,
        /// The player's ckey
        ckey: Path<String>,
        pool: Data<&MySqlPool>,
    ) -> RoletimePlayerResponse {
        match get_roletime_player(&ckey, &pool).await {
            Ok(roletime) => RoletimePlayerResponse::Success(Json(roletime)),
            Err(e) => match e {
                DatabaseError::PlayerNotFound => RoletimePlayerResponse::NotFound(e.into()),
                _ => {
                    error!("Error fetching roletimes for player `{}`: {e:?}", *ckey);
                    RoletimePlayerResponse::InternalError(e.into())
                }
            },
        }
    }
}

#[derive(ApiResponse)]
enum RoletimePlayerResponse {
    /// Returns when roletimes successfully retrieved
    #[oai(status = 200)]
    Success(Json<Vec<PlayerRoletime>>),
    /// Returns when player not found
    #[oai(status = 404)]
    NotFound(PlainText<String>),
    /// Returns when an internal error occurs
    #[oai(status = 500)]
    InternalError(PlainText<String>),
}
