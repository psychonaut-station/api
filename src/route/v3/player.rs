//! Player information API endpoints.
//!
//! Provides REST endpoints for querying data about players.

use poem::web::Data;
use poem_openapi::{
    ApiResponse, OpenApi,
    param::{Path, Query},
    payload::{Json, PlainText},
};
use sqlx::MySqlPool;
use tracing::error;

use crate::database::{
    Achievement, Activity, Ban, Character, Error as DatabaseError, Player, get_player,
    get_player_achievements, get_player_activity, get_player_bans, get_player_characters,
};

pub struct Endpoint;

#[OpenApi]
impl Endpoint {
    /// /v3/player/{ckey}
    ///
    /// Retrieves basic player information by ckey.
    #[oai(path = "/player/:ckey", method = "get")]
    async fn player(
        &self,
        /// The player's ckey.
        ckey: Path<String>,
        pool: Data<&MySqlPool>,
    ) -> PlayerResponse {
        match get_player(&ckey, &pool).await {
            Ok(player) => PlayerResponse::Success(Json(player)),
            Err(e) => match e {
                DatabaseError::PlayerNotFound => PlayerResponse::NotFound(e.into()),
                _ => {
                    error!(ckey = *ckey, err = ?e, "error fetching player");
                    PlayerResponse::InternalError(e.into())
                }
            },
        }
    }

    /// /v3/player/{ckey}/achievements
    ///
    /// Retrieves unlocked achievements of the player.
    #[oai(path = "/player/:ckey/achievements", method = "get")]
    async fn player_achievements(
        &self,
        /// The player's ckey.
        ckey: Path<String>,
        /// Optional filter for specific achievement type.
        achievement_type: Query<Option<String>>,
        pool: Data<&MySqlPool>,
    ) -> PlayerAchievementsResponse {
        match get_player_achievements(&ckey, &achievement_type, &pool).await {
            Ok(achievements) => PlayerAchievementsResponse::Success(Json(achievements)),
            Err(e) => match e {
                DatabaseError::PlayerNotFound => PlayerAchievementsResponse::NotFound(e.into()),
                _ => {
                    error!(ckey = *ckey, err = ?e, "error fetching player achievements");
                    PlayerAchievementsResponse::InternalError(e.into())
                }
            },
        }
    }

    /// /v3/player/{ckey}/activity
    ///
    /// Retrieves 180 day activity for the player.
    #[oai(path = "/player/:ckey/activity", method = "get")]
    async fn player_activity(
        &self,
        /// The player's ckey.
        ckey: Path<String>,
        pool: Data<&MySqlPool>,
    ) -> PlayerActivityResponse {
        match get_player_activity(&ckey, &pool).await {
            Ok(activity) => PlayerActivityResponse::Success(Json(activity)),
            Err(e) => match e {
                DatabaseError::PlayerNotFound => PlayerActivityResponse::NotFound(e.into()),
                _ => {
                    error!(ckey = *ckey, err = ?e, "error fetching player activity");
                    PlayerActivityResponse::InternalError(e.into())
                }
            },
        }
    }

    /// /v3/player/{ckey}/bans
    ///
    /// Retrieves ban history for a specific player.
    #[oai(path = "/player/:ckey/bans", method = "get")]
    async fn player_bans(
        &self,
        /// The player's ckey.
        ckey: Path<String>,
        /// Optional boolean to filter for permanent bans only.
        permanent: Query<Option<bool>>,
        /// Optional date string (YYYY-MM-DD format) to filter bans after a specific date.
        #[oai(validator(pattern = "\\d{4}-\\d{2}-\\d{2}"))]
        since: Query<Option<String>>,
        pool: Data<&MySqlPool>,
    ) -> PlayerBansResponse {
        match get_player_bans(&ckey, permanent.unwrap_or(false), &since, &pool).await {
            Ok(bans) => PlayerBansResponse::Success(Json(bans)),
            Err(e) => match e {
                DatabaseError::PlayerNotFound => PlayerBansResponse::NotFound(e.into()),
                _ => {
                    error!(ckey = *ckey, err = ?e, "error fetching player bans");
                    PlayerBansResponse::InternalError(e.into())
                }
            },
        }
    }

    /// /v3/player/{ckey}/characters
    ///
    /// Retrieves characters associated with the player.
    #[oai(path = "/player/:ckey/characters", method = "get")]
    async fn player_characters(
        &self,
        /// The player's ckey.
        ckey: Path<String>,
        pool: Data<&MySqlPool>,
    ) -> PlayerCharactersResponse {
        match get_player_characters(&ckey, &pool).await {
            Ok(characters) => PlayerCharactersResponse::Success(Json(characters)),
            Err(e) => match e {
                DatabaseError::PlayerNotFound => PlayerCharactersResponse::NotFound(e.into()),
                _ => {
                    error!(ckey = *ckey, err = ?e, "error fetching player characters");
                    PlayerCharactersResponse::InternalError(e.into())
                }
            },
        }
    }
}

#[derive(ApiResponse)]
enum PlayerResponse {
    /// Returns when wlayer data successfully retrieved.
    #[oai(status = 200)]
    Success(Json<Player>),
    /// Returns when player with the specified ckey does not exist.
    #[oai(status = 404)]
    NotFound(PlainText<String>),
    /// Returns when a database error occurred.
    #[oai(status = 500)]
    InternalError(PlainText<String>),
}

#[derive(ApiResponse)]
enum PlayerAchievementsResponse {
    /// Returns when player achievements successfully retrieved.
    #[oai(status = 200)]
    Success(Json<Vec<Achievement>>),
    /// Returns when player with the specified ckey does not exist.
    #[oai(status = 404)]
    NotFound(PlainText<String>),
    /// Returns when a database error occurred.
    #[oai(status = 500)]
    InternalError(PlainText<String>),
}

#[derive(ApiResponse)]
enum PlayerActivityResponse {
    /// Returns when player activity successfully retrieved.
    #[oai(status = 200)]
    Success(Json<Vec<Activity>>),
    /// Returns when player with the specified ckey does not exist.
    #[oai(status = 404)]
    NotFound(PlainText<String>),
    /// Returns when a database error occurred.
    #[oai(status = 500)]
    InternalError(PlainText<String>),
}

#[derive(ApiResponse)]
enum PlayerBansResponse {
    /// Returns when player bans successfully retrieved.
    #[oai(status = 200)]
    Success(Json<Vec<Ban>>),
    /// Returns when player with the specified ckey does not exist.
    #[oai(status = 404)]
    NotFound(PlainText<String>),
    /// Returns when a database error occurred.
    #[oai(status = 500)]
    InternalError(PlainText<String>),
}

#[derive(ApiResponse)]
enum PlayerCharactersResponse {
    /// Returns when player characters successfully retrieved.
    #[oai(status = 200)]
    Success(Json<Vec<Character>>),
    /// Returns when player with the specified ckey does not exist.
    #[oai(status = 404)]
    NotFound(PlainText<String>),
    /// Returns when a database error occurred.
    #[oai(status = 500)]
    InternalError(PlainText<String>),
}
