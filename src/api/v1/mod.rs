use rocket::{routes, Build, Rocket};
use std::sync::Arc;
use tokio::sync::Mutex;

mod common;
mod player;
mod server;

pub use common::*;

pub fn mount(rocket: Rocket<Build>) -> Rocket<Build> {
    let cache = Arc::new(Mutex::new(Cache::default()));

    rocket
        .manage(cache)
        .mount("/v1", routes![server::index, player::index, player::top])
}
