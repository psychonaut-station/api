mod player;
mod recent_test_merges;
mod server;

use poem_openapi::OpenApiService;

macro_rules! service {
    ($($endpoint:path),*) => {
        pub fn service() -> OpenApiService<($($endpoint,)*), ()> {
            OpenApiService::new(($($endpoint,)*), "Psychonaut Station API", "3.0.0").url_prefix("/v3")
        }
    };
}

service!(
    server::Endpoint,
    player::Endpoint,
    recent_test_merges::Endpoint
);
