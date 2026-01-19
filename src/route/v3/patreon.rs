//! Patreon integration API endpoints.
//!
//! Provides endpoints for checking Patreon supporter status and retrieving
//! the list of all supporters.

use poem::web::Data;
use poem_openapi::{
    ApiResponse, OpenApi,
    param::Path,
    payload::{Json, PlainText},
};
use sqlx::MySqlPool;
use tracing::error;

use crate::{
    config::Config,
    database::{get_patrons, is_patron},
};

pub struct Endpoint;

#[OpenApi]
impl Endpoint {
    /// /v3/patreon
    ///
    /// Retrieves the list of our Patreon supporters' ckeys
    #[oai(path = "/patreon", method = "get")]
    async fn patreon(&self, pool: Data<&MySqlPool>, config: Data<&Config>) -> PatreonResponse {
        match get_patrons(&pool, &config).await {
            Ok(patrons) => PatreonResponse::Success(Json(patrons)),
            Err(e) => {
                error!(err = ?e, "error fetching patrons");
                PatreonResponse::InternalError(e.into())
            }
        }
    }

    /// /v3/patreon/{ckey}
    ///
    /// Checks if a given ckey is a Patreon supporter
    #[oai(path = "/patreon/:ckey", method = "get")]
    async fn patreon_status(
        &self,
        ckey: Path<String>,
        pool: Data<&MySqlPool>,
        config: Data<&Config>,
    ) -> PatreonStatusResponse {
        match is_patron(&ckey, &pool, &config).await {
            Ok(is) => PatreonStatusResponse::Success(Json(is)),
            Err(e) => {
                error!(err = ?e, "error checking patron status");
                PatreonStatusResponse::InternalError(e.into())
            }
        }
    }
}

#[derive(ApiResponse)]
enum PatreonResponse {
    /// Returns when Patreon supporters successfully retrieved
    #[oai(status = 200)]
    Success(Json<Vec<String>>),
    /// Returns when a database or HTTP error occurred
    #[oai(status = 500)]
    InternalError(PlainText<String>),
}

#[derive(ApiResponse)]
enum PatreonStatusResponse {
    /// Returns whether the specified ckey is a Patreon supporter
    #[oai(status = 200)]
    Success(Json<bool>),
    /// Returns when a database or HTTP error occurred
    #[oai(status = 500)]
    InternalError(PlainText<String>),
}
