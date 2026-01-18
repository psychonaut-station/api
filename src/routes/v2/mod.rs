use rocket::{routes, Build, Rocket};

mod autocomplete;
mod byond;
mod common;
mod discord;
mod events;
mod patreon;
mod player;
mod round;
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
            player::fav_character,
            player::tickets,
            player::messages,
            player::notes,
            player::rounds,
            player::friends,
            player::friend_invites,
            player::check_friends,
            player::addfriend,
            player::removefriend,
            player::acceptfriend,
            player::declinefriend,
            player::lookup,
            round::index,
            round::rounds,
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
