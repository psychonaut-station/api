use std::{net::SocketAddr, time::Duration};

use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt as _},
    net::TcpStream,
    time::timeout,
};

use super::{Error, Result};

const BYOND_PACKET_HEADER_SIZE: usize = 4;

struct ResponseHeader {
    #[allow(dead_code)]
    r#type: u16,
    size: usize,
}

#[derive(Debug)]
pub enum Response {
    Null,
    #[allow(dead_code)]
    Float(f32),
    String(String),
}

pub async fn topic(address: SocketAddr, data: &str) -> Result<Response> {
    let length = data.len() + 6;

    let mut packet = vec![0x00, 0x83, 0x00, length as u8];
    packet.extend([0x00; 5]);
    packet.extend(data.as_bytes());
    packet.push(0x00);

    let mut stream = timeout(Duration::from_secs(5), TcpStream::connect(address)).await??;
    stream.write_all(&packet).await?;

    let mut response_header = [0; BYOND_PACKET_HEADER_SIZE];
    stream.read_exact(&mut response_header).await?;

    let response_header = ResponseHeader {
        r#type: u16::from_be_bytes([response_header[0], response_header[1]]),
        size: u16::from_be_bytes([response_header[2], response_header[3]]) as usize,
    };

    let mut response = vec![0; response_header.size];
    stream.read_exact(&mut response).await?;

    if response.len() > 2 {
        match response[0] {
            0x0 => return Ok(Response::Null),
            0x2A => {
                let float =
                    f32::from_be_bytes([response[1], response[2], response[3], response[4]]);
                return Ok(Response::Float(float));
            }
            0x6 => {
                let string = String::from_utf8_lossy(&response[1..response.len() - 1]).to_string();
                return Ok(Response::String(string));
            }
            _ => {}
        }
    }

    Err(Error::InvalidResponse)
}
