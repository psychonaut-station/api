use std::{error::Error, str::FromStr};

use serde::Serialize;
use serde_repr::Serialize_repr;

use super::topic::{ByondResponse, topic};

#[derive(Debug, Default, Serialize_repr)]
#[repr(u8)]
pub enum GameState {
    #[default]
    Startup = 0,
    Pregame = 1,
    SettingUp = 2,
    Playing = 3,
    Finished = 4,
}

impl FromStr for GameState {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(GameState::Startup),
            "1" => Ok(GameState::Pregame),
            "2" => Ok(GameState::SettingUp),
            "3" => Ok(GameState::Playing),
            "4" => Ok(GameState::Finished),
            _ => Err(format!("Unknown game state: {s}").into()),
        }
    }
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityLevel {
    #[default]
    Green,
    Blue,
    Red,
    Delta,
}

impl FromStr for SecurityLevel {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "green" => Ok(SecurityLevel::Green),
            "blue" => Ok(SecurityLevel::Blue),
            "red" => Ok(SecurityLevel::Red),
            "delta" => Ok(SecurityLevel::Delta),
            _ => Err(format!("Unknown security level: {s}").into()),
        }
    }
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "snake_case")]
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
    #[serde(rename = "endgame: game over")]
    Endgame,
    Recharging,
    Landing,
}

impl FromStr for ShuttleMode {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
            _ => Err(format!("Unknown shuttle mode: {s}").into()),
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct ServerStatus {
    pub version: String,
    pub respawn: bool,
    pub enter: bool,
    pub ai: bool,
    pub host: String,
    pub round_id: u32,
    pub players: u32,
    pub revision: String,
    pub revision_data: String,
    pub hub: bool,
    pub identifier: bool,
    pub admins: u32,
    pub gamestate: GameState,
    pub map_name: String,
    pub security_level: SecurityLevel,
    pub round_duration: u32,
    pub time_dilation_current: f32,
    pub time_dilation_avg: f32,
    pub time_dilation_avg_slow: f32,
    pub time_dilation_avg_fast: f32,
    pub soft_popcap: u32,
    pub hard_popcap: u32,
    pub extreme_popcap: u32,
    pub popcap: bool,
    pub bunkered: bool,
    pub interviews: bool,
    pub shuttle_mode: ShuttleMode,
    pub shuttle_timer: u32,
}

pub async fn status(address: &str) -> Result<ServerStatus, Box<dyn Error>> {
    match topic(address, "?status").await? {
        ByondResponse::String(response) => {
            let mut status = ServerStatus::default();

            for params in response.split('&') {
                let mut split = params.splitn(2, '=');

                let key = split.next().ok_or("invalid response")?;
                let value = split.next().unwrap_or("");

                match key {
                    "version" => status.version = value.to_string(),
                    "respawn" => status.respawn = value == "1",
                    "enter" => status.enter = value == "1",
                    "ai" => status.ai = value == "1",
                    "host" => status.host = value.to_string(),
                    "round_id" => status.round_id = value.parse()?,
                    "players" => status.players = value.parse()?,
                    "revision" => status.revision = value.to_string(),
                    "revision_date" => status.revision_data = value.to_string(),
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
                    "hard_popcap" => status.extreme_popcap = value.parse()?,
                    "extreme_popcap" => status.extreme_popcap = value.parse()?,
                    "popcap" => status.popcap = value == "1",
                    "bunkered" => status.bunkered = value == "1",
                    "interviews" => status.interviews = value == "1",
                    "shuttle_mode" => status.shuttle_mode = value.parse()?,
                    "shuttle_timer" => status.shuttle_timer = value.parse()?,
                    _ => {
                        #[cfg(debug_assertions)]
                        tracing::warn!(
                            "Status topic responsed with unknown param: {key} = {value} ({address})"
                        );
                    }
                }
            }

            Ok(status)
        }
        res => Err(format!("Unexpected response: {res:?}").into()),
    }
}
