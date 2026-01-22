//! API routing and endpoint configuration.
//!
//! Defines the routing structure for the API, including v3 endpoints and
//! the embedded API documentation viewer.

pub mod v3;

use http::StatusCode;
use poem::{Request, RequestBody, Route, endpoint::make_sync, error::ResponseError, web::Html};
use poem_openapi::{
    ApiExtractor, ApiExtractorType, ExtractParamOptions,
    error::AuthorizationError,
    registry::{MetaSecurityScheme, Registry},
};
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

/// Request guard for API key authentication.
///
/// Either allows or denies access to an endpoint based on the
/// provided API key and the required access tier.
pub struct KeyGuard<const TIER: usize>;

impl<const T: usize> KeyGuard<T> {
    /// Name of the security scheme for this tier.
    pub const NAME: &'static str = match T {
        1 => "API Key (full access)",
        2 => "API Key (read-only access)",
        _ => unimplemented!(),
    };
    /// Description of the security scheme for this tier.
    pub const DESCRIPTION: Option<&'static str> = match T {
        1 => Some("API Key with full access"),
        2 => Some("API Key with read-only access"),
        _ => None,
    };
}

impl<const T: usize> ApiExtractor<'_> for KeyGuard<T> {
    const TYPES: &'static [ApiExtractorType] = &[ApiExtractorType::SecurityScheme];

    type ParamType = ();
    type ParamRawType = ();

    fn register(registry: &mut Registry) {
        registry.create_security_scheme(
            KeyGuard::<T>::NAME,
            MetaSecurityScheme {
                ty: "apiKey",
                description: KeyGuard::<T>::DESCRIPTION,
                name: Option::Some("X-API-Key"),
                key_in: Option::Some("header"),
                scheme: Option::None,
                bearer_format: Option::None,
                flows: Option::None,
                openid_connect_url: Option::None,
            },
        );
    }

    fn security_schemes() -> Vec<&'static str> {
        vec![KeyGuard::<T>::NAME]
    }

    async fn from_request(
        req: &Request,
        _: &mut RequestBody,
        _: ExtractParamOptions<Self::ParamType>,
    ) -> poem::Result<Self> {
        let Some(api_key) = req.headers().get("X-API-Key").and_then(|h| h.to_str().ok()) else {
            return Err(AuthorizationError.into());
        };
        let Some(keys) = req.data::<Config>().map(|c| &c.keys) else {
            return Err(InternalServerError("server misconfiguration").into());
        };

        for key in keys.iter().take(T) {
            if key.as_bytes().ct_eq(api_key.as_bytes()).unwrap_u8() == 1 {
                return Ok(Self);
            }
        }

        Err(AuthorizationError.into())
    }
}

/// Wrapper for internal server errors with a static str.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct InternalServerError(&'static str);

impl ResponseError for InternalServerError {
    fn status(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
