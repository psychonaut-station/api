use poem_openapi::{Object, OpenApi, Union, payload::Json};

use crate::byond;

pub struct Endpoint;

#[OpenApi]
impl Endpoint {
    #[oai(path = "/server", method = "get")]
    async fn server(&self) -> Json<Vec<Server>> {
        let servers = [Placeholder {
            name: "Server 1".to_string(),
            address: "10.253.0.1:3131".to_string(),
            connection_address: "play.ss13.tr:3131".to_string(),
            error_message: "No error".to_string(),
        }];

        let mut response = Vec::with_capacity(servers.len());

        // remove
        struct Placeholder {
            name: String,
            address: String,
            connection_address: String,
            error_message: String,
        }
        //

        for server in servers.iter() {
            let status = byond::status(&server.address).await.ok();

            response.push(match status {
                Some(status) => Server::Online(ServerOnline {
                    name: server.name.clone(),
                    address: server.address.clone(),
                    connection_address: server.connection_address.clone(),
                    round_id: status.round_id,
                }),
                None => Server::Offline(ServerOffline {
                    name: server.name.clone(),
                    address: server.address.clone(),
                    error_message: server.error_message.clone(),
                }),
            });
        }

        Json(response)
    }
}

#[derive(Union)]
pub enum Server {
    Online(ServerOnline),
    Offline(ServerOffline),
}

#[derive(Object)]
pub struct ServerOnline {
    name: String,
    address: String,
    connection_address: String,
    round_id: u32,
}

#[derive(Object)]
pub struct ServerOffline {
    name: String,
    address: String,
    error_message: String,
}
