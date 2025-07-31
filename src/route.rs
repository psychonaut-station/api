mod index;
mod recent_test_merges;
mod v3;

use poem::{IntoEndpoint, Route, RouteMethod, http::Method};

use self::{index::Index, recent_test_merges::RecentTestMerges};

pub(super) fn route() -> Route {
    Route::new()
        .route::<Index>()
        .route::<RecentTestMerges>()
        .nest("/v3", v3::route())
}

trait RouteExt {
    fn route<T: Router>(self) -> Self;
}

impl RouteExt for Route {
    fn route<T: Router>(self) -> Self {
        self.at(
            T::path(),
            RouteMethod::new().method(T::method(), T::handler()),
        )
    }
}

trait Router {
    fn path() -> &'static str;
    fn handler() -> impl IntoEndpoint + 'static;
    fn method() -> Method;
}
