use std::net::{Ipv4Addr, UdpSocket};

use anyhow::Context;
use dns_starter_rust::message::{
    HeaderError, Message, OperationCode, ResourceClass, ResourceData, ResourceRecord,
};

fn main() -> anyhow::Result<()> {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);

                let mut message: Message =
                    buf[..size].try_into().context("decoding query message")?;

                let name = message.questions[0].name.clone();

                match message.header.operation_code {
                    OperationCode::StandardQuery => message.header.response = Ok(()),
                    _ => message.header.response = Err(HeaderError::NotImplemented),
                }

                message.answer(ResourceRecord {
                    name,
                    class: ResourceClass::IN,
                    time_to_live: 60,
                    data: ResourceData::Address(Ipv4Addr::new(8, 8, 8, 8)),
                });

                message.respond();

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
