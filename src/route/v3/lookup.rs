use poem::web::Data;
use poem_openapi::{
    ApiResponse, OpenApi,
    param::Path,
    payload::{Json, PlainText},
};
use sqlx::MySqlPool;
use tracing::error;

use crate::database::{Lookup, lookup_cid, lookup_ip, lookup_player};

pub struct Endpoint;

#[OpenApi]
impl Endpoint {
    /// /v3/lookup/cid/{cid}
    ///
    /// Retrieves lookup information by computer ID
    #[oai(path = "/lookup/cid/:cid", method = "get")]
    async fn lookup_cid(
        &self,
        /// The computer ID to look up
        cid: Path<String>,
        pool: Data<&MySqlPool>,
    ) -> LookupResponse {
        match lookup_cid(&cid, &pool).await {
            Ok(lookup) => LookupResponse::Success(Json(lookup)),
            Err(e) => {
                error!("Error fetching lookup for cid `{}`: {e:?}", *cid);
                LookupResponse::InternalError(e.into())
            }
        }
    }

    /// /v3/lookup/ip/{ip}
    ///
    /// Retrieves lookup information by IP address
    #[oai(path = "/lookup/ip/:ip", method = "get")]
    async fn lookup_ip(
        &self,
        /// The IP address to look up
        #[oai(validator(pattern = "\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}"))]
        ip: Path<String>,
        pool: Data<&MySqlPool>,
    ) -> LookupResponse {
        match lookup_ip(&ip, &pool).await {
            Ok(lookup) => LookupResponse::Success(Json(lookup)),
            Err(e) => {
                error!("Error fetching lookup for IP `{}`: {e:?}", *ip);
                LookupResponse::InternalError(e.into())
            }
        }
    }

    /// /v3/lookup/player/{ckey}
    ///
    /// Retrieves lookup information by player's ckey
    #[oai(path = "/lookup/player/:ckey", method = "get")]
    async fn lookup_player(
        &self,
        /// The player's ckey
        ckey: Path<String>,
        pool: Data<&MySqlPool>,
    ) -> LookupResponse {
        match lookup_player(&ckey, &pool).await {
            Ok(lookup) => LookupResponse::Success(Json(lookup)),
            Err(e) => {
                error!("Error fetching roletimes for player `{}`: {e:?}", *ckey);
                LookupResponse::InternalError(e.into())
            }
        }
    }
}

#[derive(ApiResponse)]
enum LookupResponse {
    /// Returns when lookup successfully retrieved
    #[oai(status = 200)]
    Success(Json<Vec<Lookup>>),
    /// Returns when a database error occurred
    #[oai(status = 500)]
    InternalError(PlainText<String>),
}
