use std::net::UdpSocket;

use anyhow::Context;
use dns_starter_rust::message::{Message, QuestionClass, QuestionType};

fn main() -> anyhow::Result<()> {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);
                let id = u16::from_be_bytes(buf[..2].try_into()?);

                let mut message = Message::new(id);
                message
                    .ask("codecrafters.io", QuestionType::A, QuestionClass::IN)
                    .context("sending `codecrafters.io` question")?;

                let response: Vec<u8> = message.into();
                udp_socket
                    .send_to(&response, source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }

    Ok(())
}
