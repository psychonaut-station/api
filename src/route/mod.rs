mod v3;

use poem::Route;

pub(super) fn route() -> Route {
    let service = v3::service();
    let ui = service.stoplight_elements();

    Route::new().nest("/v3", service).nest("/", ui)
}
