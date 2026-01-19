//! Server status API endpoint.
//!
//! Provides an endpoint for querying the status of all configured game servers.

use poem::web::Data;
use poem_openapi::{ApiResponse, Object, OpenApi, Union, payload::Json};
use tracing::error;

use crate::{
    byond::{
        self, Error as ByondError,
        status::{GameState, SecurityLevel},
    },
    cache::Cache,
    config::Config,
};

pub struct Endpoint;

#[OpenApi]
impl Endpoint {
    /// /v3/server
    ///
    /// Retrieves the status of the game servers
    #[oai(path = "/server", method = "get")]
    async fn server(&self, cache: Data<&Cache>, config: Data<&Config>) -> Response {
        if let Some(cached) = cache.get_server_status().await {
            return Response::Success(Json(cached));
        }

        let mut response = Vec::with_capacity(config.servers.len());

        for server in config.servers.iter() {
            let status = match byond::status(server.address).await {
                Ok(status) => Some(status),
                Err(e) => match e {
                    ByondError::Elapsed(_) => None,
                    // ByondError::Io(e) if e.kind() == io::ErrorKind::TimedOut => None,
                    _ => {
                        error!(server = server.name, err = ?e, "error fetching server status");
                        None
                    }
                },
            };

            response.push(match status {
                Some(status) => Server::Online(Online {
                    name: server.name.clone(),
                    address: server.connection_address.clone(),
                    round_id: status.round_id,
                    players: status.players,
                    map: status.map_name,
                    security_level: status.security_level,
                    round_duration: status.round_duration,
                    gamestate: status.gamestate,
                }),
                None => Server::Offline(Offline {
                    name: server.name.clone(),
                    address: server.connection_address.clone(),
                    error_message: server.error_message.clone(),
                }),
            });
        }

        cache.set_server_status(response.clone()).await;

        Response::Success(Json(response))
    }
}

#[derive(ApiResponse)]
enum Response {
    /// Returns when server status successfully retrieved
    #[oai(status = 200)]
    Success(Json<Vec<Server>>),
}

#[derive(Union, Clone)]
pub enum Server {
    /// The server is online and provides its status
    Online(Online),
    /// The server is offline or unreachable
    Offline(Offline),
}

#[derive(Object, Clone)]
#[oai(rename = "ServerOnline")]
pub struct Online {
    /// The display name of the server
    name: String,
    /// The connection address of the server
    address: String,
    /// The ID of the current round
    round_id: u32,
    /// The number of players currently on the server
    players: u32,
    /// The name of the current map
    map: String,
    /// The security level in the round
    security_level: SecurityLevel,
    /// The duration of the current round in seconds
    round_duration: u32,
    /// The state of the round
    gamestate: GameState,
}

#[derive(Object, Clone)]
#[oai(rename = "ServerOffline")]
pub struct Offline {
    /// The display name of the server
    name: String,
    /// The connection address of the server
    address: String,
    /// The error message to display when the server is offline
    error_message: String,
}
