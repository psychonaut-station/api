//! Ban-related endpoints for the API.
//!
//! Provides an endpoint for retrieving ban information by its ID.

use poem::web::Data;
use poem_openapi::{
    ApiResponse, OpenApi,
    param::Path,
    payload::{Json, PlainText},
};
use sqlx::MySqlPool;

use crate::database::{Ban, get_ban_by_id};

pub struct Endpoint;

#[OpenApi]
impl Endpoint {
    /// /v3/ban/{id}
    ///
    /// Retrieves ban information by its ID
    #[oai(path = "/ban/:id", method = "get")]
    async fn ban(&self, id: Path<u32>, pool: Data<&MySqlPool>) -> Response {
        match get_ban_by_id(*id, &pool).await {
            Ok(Some(ban)) => Response::Success(Json(ban)),
            Ok(None) => Response::NotFound(PlainText(format!("ban with ID {} not found", *id))),
            Err(e) => Response::InternalError(e.into()),
        }
    }
}

#[derive(ApiResponse)]
enum Response {
    /// Returns when ban information successfully retrieved
    #[oai(status = 200)]
    Success(Json<Ban>),
    /// Returns when the ban with the specified ID was not found
    #[oai(status = 404)]
    NotFound(PlainText<String>),
    /// Returns when a database error occurred
    #[oai(status = 500)]
    InternalError(PlainText<String>),
}
