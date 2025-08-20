use poem::web::Data;
use poem_openapi::{
    ApiResponse, OpenApi,
    param::Path,
    payload::{Json, PlainText},
};
use sqlx::MySqlPool;
use tracing::error;

use crate::database::{
    Error as DatabaseError, JobRoletime, PlayerRoletime, get_roletime_player, get_roletime_top,
};

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

    /// /v3/roletime/top/{job}
    ///
    /// Retrieves the top 15 players for a specific job based on minutes played.
    #[oai(path = "/roletime/top/:job", method = "get")]
    async fn roletime_top(
        &self,
        /// The job to filter by
        job: Path<String>,
        pool: Data<&MySqlPool>,
    ) -> RoletimeTopResponse {
        match get_roletime_top(&job, &pool).await {
            Ok(roletime) => RoletimeTopResponse::Success(Json(roletime)),
            Err(e) => {
                error!("Error fetching top roletimes for job `{}`: {e:?}", *job);
                RoletimeTopResponse::InternalError(e.into())
            }
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

#[derive(ApiResponse)]
enum RoletimeTopResponse {
    /// Returns when top players successfully retrieved
    #[oai(status = 200)]
    Success(Json<Vec<JobRoletime>>),
    #[oai(status = 500)]
    InternalError(PlainText<String>),
}
