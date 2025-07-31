mod server;

use poem::Route;

use super::RouteExt as _;

use self::server::Server;

pub(super) fn route() -> Route {
    Route::new().route::<Server>()
}
