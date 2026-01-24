//! Connection lookup API endpoints.
//!
//! Provides REST endpoints for looking up related connections by computer ID,
//! IP address, or player ckey.

use poem::web::Data;
use poem_openapi::{
    param::Path,
    payload::{Json, PlainText},
};
use sqlx::MySqlPool;
use tracing::error;

use crate::{
    database::{Lookup, lookup_cid, lookup_ip, lookup_player},
    endpoint,
};

use super::KeyGuard;

#[endpoint]
mod __ {
    /// /v3/lookup/cid/{cid}
    ///
    /// Retrieves lookup information by computer ID.
    #[oai(path = "/lookup/cid/:cid", method = "get")]
    async fn lookup_cid(
        &self,
        /// The computer ID to look up.
        cid: Path<String>,
        pool: Data<&MySqlPool>,
        _api_key: KeyGuard<1>,
    ) -> Response {
        match lookup_cid(&cid, &pool).await {
            Ok(lookup) => Response::Success(Json(lookup)),
            Err(e) => {
                error!(cid = *cid, err = ?e, "error fetching cid lookup");
                Response::InternalError(e.into())
            }
        }
    }

    /// /v3/lookup/ip/{ip}
    ///
    /// Retrieves lookup information by IP address.
    #[oai(path = "/lookup/ip/:ip", method = "get")]
    async fn lookup_ip(
        &self,
        /// The IP address to look up.
        #[oai(validator(pattern = "\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}"))]
        ip: Path<String>,
        pool: Data<&MySqlPool>,
        _api_key: KeyGuard<1>,
    ) -> Response {
        match lookup_ip(&ip, &pool).await {
            Ok(lookup) => Response::Success(Json(lookup)),
            Err(e) => {
                error!(ip = *ip, err = ?e, "error fetching ip lookup");
                Response::InternalError(e.into())
            }
        }
    }

    /// /v3/lookup/player/{ckey}
    ///
    /// Retrieves lookup information by player's ckey.
    #[oai(path = "/lookup/player/:ckey", method = "get")]
    async fn lookup_player(
        &self,
        /// The player's ckey
        ckey: Path<String>,
        pool: Data<&MySqlPool>,
        _api_key: KeyGuard<1>,
    ) -> Response {
        match lookup_player(&ckey, &pool).await {
            Ok(lookup) => Response::Success(Json(lookup)),
            Err(e) => {
                error!(ckey = *ckey, err = ?e, "error fetching ckey lookup");
                Response::InternalError(e.into())
            }
        }
    }

    #[response]
    enum Response {
        /// Returns when lookup successfully retrieved.
        #[oai(status = 200)]
        Success(Json<Vec<Lookup>>),
        /// Returns when a database error occurred.
        #[oai(status = 500)]
        InternalError(PlainText<String>),
    }
}
