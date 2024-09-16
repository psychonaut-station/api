use std::io::Cursor;

use rocket::{
    http::{ContentType, Status},
    request::{FromRequest, Outcome},
    response::{self, Responder, Response},
    Request,
};
use serde::Serialize;

use crate::config::Config;

#[derive(Debug, Serialize)]
pub enum Json<R> {
    Ok(R),
    Conflict(R),
}

impl<R: Serialize> Responder<'_, 'static> for Json<R> {
    fn respond_to(self, _: &Request) -> response::Result<'static> {
        let (status, body) = match self {
            Json::Ok(r) => (Status::Ok, r),
            Json::Conflict(r) => (Status::Conflict, r),
        };

        let Ok(body) = serde_json::to_string(&body) else {
            return Err(Status::InternalServerError);
        };

        Response::build()
            .status(status)
            .header(ContentType::JSON)
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}

pub struct ApiKey;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let Some(config) = request.rocket().state::<Config>() else {
            return Outcome::Error((Status::InternalServerError, ()));
        };

        if request.headers().get_one("X-API-KEY") == Some(&config.secret) {
            return Outcome::Success(ApiKey);
        }

        if request.headers().get_one("X-DEV-KEY") == Some(&config.dev_secret) {
            if let Some(route) = request.route() {
                if config.dev_routes.contains(route.uri.origin.path().as_str()) {
                    return Outcome::Success(ApiKey);
                }
            }
        }

        if request.headers().get_one("X-EXP-KEY") == Some(&config.exposed_secret) {
            if let Some(route) = request.route() {
                if config
                    .exposed_routes
                    .contains(route.uri.origin.path().as_str())
                {
                    return Outcome::Success(ApiKey);
                }
            }
        }

        Outcome::Error((Status::Unauthorized, ()))
    }
}
