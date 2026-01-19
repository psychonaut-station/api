//! Version 3 API endpoints.
//!
//! Contains all v3 API endpoint implementations.

mod lookup;
mod patreon;
mod player;
mod recent_test_merges;
mod roletime;
pub mod server;

/// Macro for generating an [OpenAPI service](poem_openapi::OpenApiService) with multiple endpoint modules.
///
/// This macro takes a list of endpoint modules and creates an OpenAPI service
/// that includes all of them.
macro_rules! service {
    ($($endpoint:ident),* $(,)?) => {
        pub fn service() -> poem_openapi::OpenApiService<($($endpoint::Endpoint,)*), ()> {
            poem_openapi::OpenApiService::new(($($endpoint::Endpoint,)*), "Psychonaut Station API", "3.0.0").url_prefix("/v3")
        }
    };
}

service!(
    lookup,
    player,
    recent_test_merges,
    roletime,
    server,
    patreon,
);
