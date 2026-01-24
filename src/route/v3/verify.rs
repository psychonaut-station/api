//! Discord account verification endpoints.
//!
//! Provides endpoints for linking Discord accounts to
//! BYOND accounts using one-time tokens.

use poem::web::Data;
use poem_openapi::{
    ApiResponse, OpenApi,
    param::Path,
    payload::{Json, PlainText},
};
use sqlx::MySqlPool;
use tracing::error;

use crate::database::{self, verify_with_otp};

use super::KeyGuard;

pub struct Endpoint;

#[OpenApi]
impl Endpoint {
    /// /v3/verify/{id}/otp/{otp}
    ///
    /// Verifies a Discord account using a one-time token.
    ///
    /// Links a Discord account to a BYOND account by validating
    /// the provided token. Returns the associated ckey on success.
    #[oai(path = "/verify/:id/otp/:otp", method = "post")]
    async fn verify_otp(
        &self,
        /// The Discord user ID.
        id: Path<i64>,
        /// The one-time token (valid for 4 hours).
        otp: Path<String>,
        pool: Data<&MySqlPool>,
        _api_key: KeyGuard<1>,
    ) -> Response {
        match verify_with_otp(*id, &otp, &pool).await {
            Ok(ckey) => Response::Success(Json(ckey)),
            Err(database::Error::DiscordInUse(ckey)) => Response::Conflict(Json(ckey)),
            Err(database::Error::TokenInvalid) => Response::NotFound,
            Err(e) => {
                error!(err = ?e, "error verifying with OTP");
                Response::InternalError(e.into())
            }
        }
    }
}

#[derive(ApiResponse)]
enum Response {
    /// Returns when verification succeeds with the associated ckey.
    #[oai(status = 200)]
    Success(Json<String>),
    /// Returns when the OTP token is invalid or expired.
    #[oai(status = 404)]
    NotFound,
    /// Returns with linked ckey when the Discord account is already in use.
    #[oai(status = 409)]
    Conflict(Json<String>),
    /// Returns when a database error occurred.
    #[oai(status = 500)]
    InternalError(PlainText<String>),
}
