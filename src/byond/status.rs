//! BYOND server status parsing and types.
//!
//! Parses server status responses from BYOND topic queries
//! and provides structured types.

use std::{net::SocketAddr, str::FromStr};

use poem_openapi::Enum;
use tracing::warn;

use super::{
    Error, Result,
    topic::{Response, topic},
};

/// Queries a BYOND server for its current status.
///
/// Sends a "?status" topic query to the specified server address and parses
/// the response into a structured `Status` object.
///
/// # Arguments
///
/// * `address` - Socket address of the BYOND server.
///
/// # Returns
///
/// A `Status` object containing current server information.
///
/// # Errors
///
/// Returns an error if:
/// - Connection fails or times out
/// - Response is invalid or cannot be parsed
/// - Server returns an unexpected response type
pub async fn status(address: SocketAddr) -> Result<Status> {
    match topic(address, "?status").await? {
        Response::String(response) => {
            let mut status = Status::default();

            for params in response.split('&') {
                let mut split = params.splitn(2, '=');

                let key = split.next().ok_or(Error::InvalidResponse)?;
                let value = split.next().unwrap_or("");

                parse_param(&mut status, key, value).map_err(|e| {
                    Error::ParseParam(key.to_string(), value.to_string(), Box::new(e))
                })?;
            }

            Ok(status)
        }
        res => Err(Error::UnexpectedResponse(res)),
    }
}

/// Parses a single key-value parameter and updates the status object.
///
/// Takes a key-value pair from the status topic response and updates the
/// corresponding field in the `Status` struct. Unknown keys are logged as
/// warnings but don't cause errors.
///
/// # Errors
///
/// Returns an error if the value cannot be parsed into the expected type
/// for the given key (e.g., parsing a non-numeric string as an integer).
fn parse_param(status: &mut Status, key: &str, value: &str) -> Result<()> {
    match key {
        "version" => status.version = value.to_string(),
        "respawn" => status.respawn = value == "1",
        "enter" => status.enter = value == "1",
        "ai" => status.ai = value == "1",
        "host" => status.host = value.to_string(),
        "round_id" => status.round_id = value.parse()?,
        "players" => status.players = value.parse()?,
        "revision" => status.revision = value.to_string(),
        "revision_date" => status.revision_date = value.to_string(),
        "hub" => status.hub = value == "1",
        "identifier" => status.identifier = value == "1",
        "admins" => status.admins = value.parse()?,
        "gamestate" => status.gamestate = value.parse()?,
        "map_name" => status.map_name = value.replace('+', " "),
        "security_level" => status.security_level = value.parse()?,
        "round_duration" => status.round_duration = value.parse()?,
        "time_dilation_current" => status.time_dilation_current = value.parse()?,
        "time_dilation_avg" => status.time_dilation_avg = value.parse()?,
        "time_dilation_avg_slow" => status.time_dilation_avg_slow = value.parse()?,
        "time_dilation_avg_fast" => status.time_dilation_avg_fast = value.parse()?,
        "soft_popcap" => status.soft_popcap = value.parse()?,
        "hard_popcap" => status.hard_popcap = value.parse()?,
        "extreme_popcap" => status.extreme_popcap = value.parse()?,
        "popcap" => status.popcap = value == "1",
        "bunkered" => status.bunkered = value == "1",
        "interviews" => status.interviews = value == "1",
        "shuttle_mode" => status.shuttle_mode = value.parse()?,
        "shuttle_timer" => status.shuttle_timer = value.parse()?,
        "public_address" => status.public_address = value.to_string(),
        _ => warn!(key = value, "status topic responsed with unknown param"),
    }
    Ok(())
}

/// An enum representing the current state of the game round.
#[derive(Default, Enum, Clone, Debug)]
#[oai(rename_all = "lowercase")]
pub enum GameState {
    #[default]
    Startup,
    Pregame,
    SettingUp,
    Playing,
    Finished,
}

impl FromStr for GameState {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "0" => Ok(GameState::Startup),
            "1" => Ok(GameState::Pregame),
            "2" => Ok(GameState::SettingUp),
            "3" => Ok(GameState::Playing),
            "4" => Ok(GameState::Finished),
            _ => Err(Error::GameStateConversion(s.to_string())),
        }
    }
}

/// An enum representing the station's security alert level.
#[derive(Default, Enum, Clone, Debug)]
#[oai(rename_all = "lowercase")]
pub enum SecurityLevel {
    #[default]
    Green,
    Blue,
    Red,
    Delta,
}

impl FromStr for SecurityLevel {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "green" => Ok(SecurityLevel::Green),
            "blue" => Ok(SecurityLevel::Blue),
            "red" => Ok(SecurityLevel::Red),
            "delta" => Ok(SecurityLevel::Delta),
            _ => Err(Error::SecurityLevelConversion(s.to_string())),
        }
    }
}

/// An enum representing the current mode of the emergency shuttle.
#[derive(Default, Enum, Debug)]
#[oai(rename_all = "lowercase")]
pub enum ShuttleMode {
    #[default]
    Idle,
    Igniting,
    Recallled,
    Called,
    Docked,
    Stranded,
    Disabled,
    Escape,
    #[oai(rename = "endgame: game over")]
    Endgame,
    Recharging,
    Landing,
}

impl FromStr for ShuttleMode {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "idle" => Ok(ShuttleMode::Idle),
            "igniting" => Ok(ShuttleMode::Igniting),
            "recallled" => Ok(ShuttleMode::Recallled),
            "called" => Ok(ShuttleMode::Called),
            "docked" => Ok(ShuttleMode::Docked),
            "stranded" => Ok(ShuttleMode::Stranded),
            "disabled" => Ok(ShuttleMode::Disabled),
            "escape" => Ok(ShuttleMode::Escape),
            "endgame%3a+game+over" => Ok(ShuttleMode::Endgame),
            "recharging" => Ok(ShuttleMode::Recharging),
            "landing" => Ok(ShuttleMode::Landing),
            _ => Err(Error::ShuttleModeConversion(s.to_string())),
        }
    }
}

/// Complete status information for a BYOND game server.
#[derive(Default, Debug)]
pub struct Status {
    /// Game version (e.g., "/tg/Station 13").
    pub version: String,
    /// Whether respawning is enabled.
    pub respawn: bool,
    /// Whether new players can join the round.
    pub enter: bool,
    /// Whether AI slot is available.
    pub ai: bool,
    /// Name of the server host.
    pub host: String,
    /// Current round ID.
    pub round_id: u32,
    /// Number of connected players.
    pub players: u32,
    /// Git revision hash.
    pub revision: String,
    /// Revision date.
    pub revision_date: String,
    /// Whether server is visible on BYOND hub.
    pub hub: bool,
    /// Short form server name used for its database.
    pub identifier: bool,
    /// Number of online admins.
    pub admins: u32,
    /// Current game state.
    pub gamestate: GameState,
    /// Current map name.
    pub map_name: String,
    /// Current security alert level.
    pub security_level: SecurityLevel,
    /// Round duration in deciseconds.
    pub round_duration: u32,
    /// Current time dilation (server performance).
    pub time_dilation_current: f32,
    /// Average time dilation.
    pub time_dilation_avg: f32,
    /// Slow average time dilation.
    pub time_dilation_avg_slow: f32,
    /// Fast average time dilation.
    pub time_dilation_avg_fast: f32,
    /// Soft population cap threshold.
    pub soft_popcap: u32,
    /// Hard population cap threshold.
    pub hard_popcap: u32,
    /// Extreme population cap threshold.
    pub extreme_popcap: u32,
    /// Whether population cap is active.
    pub popcap: bool,
    /// Whether server is in bunkered mode.
    pub bunkered: bool,
    /// Whether interviews are required for new players.
    pub interviews: bool,
    /// Current shuttle mode.
    pub shuttle_mode: ShuttleMode,
    /// Shuttle timer in seconds.
    pub shuttle_timer: u32,
    /// Public connection address.
    pub public_address: String,
}
