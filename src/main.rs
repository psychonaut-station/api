mod byond;
mod route;

use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use poem::{Server, listener::TcpListener};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = tracing_subscriber::fmt().finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 3000);

    let app = route::route();
    let server = Server::new(TcpListener::bind(socket));

    server.run(app).await?;

    Ok(())
}
