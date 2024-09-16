use rocket::{routes, Build, Rocket};

mod autocomplete;
mod common;
mod discord;
mod patreon;
mod player;
mod server;
mod verify;

pub use common::*;

pub fn mount(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount(
        "/v2",
        routes![
            patreon::index,
            player::index,
            player::ban,
            player::characters,
            player::roletime,
            player::activity,
            player::top,
            player::discord,
            server::index,
            verify::index,
            verify::unverify,
            discord::user,
            discord::member,
            autocomplete::job,
            autocomplete::ckey,
            autocomplete::ic_name
        ],
    )
}
