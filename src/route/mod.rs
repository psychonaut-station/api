//! API routing and endpoint configuration.
//!
//! Defines the routing structure for the API, including v3 endpoints and
//! the embedded API documentation viewer.

pub mod v3;

use poem::{Route, endpoint::make_sync, web::Html};

/// Embedded HTML content for the API documentation viewer.
const STOPLIGHT_ELEMENTS: &str = include_str!("stoplight-elements.html");

/// Creates the main routing structure for the API.
///
/// Sets up routes for the v3 API endpoints and the documentation viewer.
///
/// # Returns
///
/// A configured `Route` with all API endpoints and UI
pub(super) fn route() -> Route {
    let service = v3::service();
    let ui_html = STOPLIGHT_ELEMENTS.replace("'{:spec}'", &service.spec());

    Route::new()
        .nest("/v3", service)
        .nest("/", make_sync(move |_| Html(ui_html.clone())))
}
