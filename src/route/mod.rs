//! API routing and endpoint configuration.
//!
//! Defines the routing structure for the API, including v3 endpoints and
//! the embedded API documentation viewer.

pub mod v3;

use poem::{Request, Route, endpoint::make_sync, web::Html};
use poem_openapi::{SecurityScheme, auth::ApiKey};
use subtle::ConstantTimeEq;

use crate::config::Config;

/// Embedded HTML content for the API documentation viewer.
const STOPLIGHT_ELEMENTS: &str = include_str!("stoplight-elements.html");

/// Creates the main routing structure for the API.
///
/// Sets up routes for the v3 API endpoints and the documentation viewer.
///
/// # Returns
///
/// A configured `Route` with all API endpoints and UI.
pub(super) fn route() -> Route {
    let service = v3::service();
    let ui_html = STOPLIGHT_ELEMENTS.replace("'{:spec}'", &service.spec());

    Route::new()
        .nest("/v3", service)
        .nest("/", make_sync(move |_| Html(ui_html.clone())))
}

/// Security scheme/guard for API key authentication.
///
/// Validates requests using an `X-API-Key` header.
#[derive(SecurityScheme)]
#[oai(
    ty = "api_key",
    key_name = "X-API-Key",
    key_in = "header",
    checker = "key_checker"
)]
pub struct KeyGuard(());

/// Validates the API key in constant time to prevent timing attacks.
///
/// Compares the provided API key against the configured key using constant-time
/// comparison from the [`subtle`] crate. This prevents attackers from using timing
/// information to guess the API key character by character.
// TODO: Should we log failed authentication attempts for security monitoring?
async fn key_checker(req: &Request, api_key: ApiKey) -> Option<()> {
    let key = req.data::<Config>()?.key.as_bytes();
    match key.ct_eq(api_key.key.as_bytes()).unwrap_u8() {
        1 => Some(()),
        _ => None,
    }
}
