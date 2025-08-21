pub mod v3;

use poem::{Route, endpoint::make_sync, web::Html};

const STOPLIGHT_ELEMENTS: &str = include_str!("stoplight-elements.html");

pub(super) fn route() -> Route {
    let service = v3::service();
    let ui_html = STOPLIGHT_ELEMENTS.replace("'{:spec}'", &service.spec());

    Route::new()
        .nest("/v3", service)
        .nest("/", make_sync(move |_| Html(ui_html.clone())))
}
