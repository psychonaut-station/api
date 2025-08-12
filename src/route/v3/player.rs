use poem::web::Data;
use poem_openapi::{
    ApiResponse, OpenApi,
    param::{Path, Query},
    payload::{Json, PlainText},
};
use sqlx::MySqlPool;
use tracing::error;

use crate::database::{Ban, Error as DatabaseError, Player, get_player, get_player_bans};

pub struct Endpoint;

#[OpenApi]
impl Endpoint {
    /// /v3/player/{ckey}
    ///
    /// Retrieves basic player information by ckey
    #[oai(path = "/player/:ckey", method = "get")]
    async fn player(
        &self,
        /// The player's unique ckey identifier
        ckey: Path<String>,
        pool: Data<&MySqlPool>,
    ) -> PlayerResponse {
        match get_player(&ckey, &pool).await {
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

    /// /v3/player/{ckey}/bans
    ///
    /// Retrieves ban history for a specific player
    #[oai(path = "/player/:ckey/bans", method = "get")]
    async fn player_bans(
        &self,
        /// The player's unique ckey identifier
        ckey: Path<String>,
        /// Optional boolean to filter for permanent bans only
        permanent: Query<Option<bool>>,
        /// Optional date string (YYYY-MM-DD format) to filter bans after a specific date
        #[oai(validator(pattern = "/\\d{4}-\\d{2}-\\d{2}/"))]
        since: Query<Option<String>>,
        pool: Data<&MySqlPool>,
    ) -> PlayerBansResponse {
        match get_player_bans(&ckey, permanent.unwrap_or(false), &since, &pool).await {
            Ok(bans) => PlayerBansResponse::Success(Json(bans)),
            Err(e) => match e {
                DatabaseError::PlayerNotFound => PlayerBansResponse::NotFound(e.into()),
                _ => {
                    error!("Error fetching bans for player `{}`: {e:?}", *ckey);
                    PlayerBansResponse::InternalError(e.into())
                }
            },
        }
    }
}

#[derive(ApiResponse)]
enum PlayerResponse {
    /// Returns when wlayer data successfully retrieved
    #[oai(status = 200)]
    Success(Json<Player>),
    /// Returns when player with the specified ckey does not exist
    #[oai(status = 404)]
    NotFound(PlainText<String>),
    /// Returns when a database error occurred
    #[oai(status = 500)]
    InternalError(PlainText<String>),
}

#[derive(ApiResponse)]
enum PlayerBansResponse {
    /// Returns when player bans successfully retrieved
    #[oai(status = 200)]
    Success(Json<Vec<Ban>>),
    /// Returns when player with the specified ckey does not exist
    #[oai(status = 404)]
    NotFound(PlainText<String>),
    /// Returns when a database error occurred
    #[oai(status = 500)]
    InternalError(PlainText<String>),
}
