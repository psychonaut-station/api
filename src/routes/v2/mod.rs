use rocket::{routes, Build, Rocket};

mod autocomplete;
mod byond;
mod common;
mod discord;
mod events;
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
            patreon::patrons,
            player::index,
            player::ban,
            player::characters,
            player::roletime,
            player::activity,
            player::top,
            player::discord,
            player::achievements,
            player::lookup,
            server::index,
            verify::index,
            verify::unverify,
            discord::user,
            discord::member,
            byond::member,
            autocomplete::job,
            autocomplete::ckey,
            autocomplete::ic_name,
            autocomplete::hide_ckey_autocomplete,
            autocomplete::unhide_ckey_autocomplete,
            events::overview,
            events::citations,
            events::crimes,
            events::deaths,
        ],
    )
}
