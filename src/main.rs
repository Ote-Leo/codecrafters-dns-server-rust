use std::net::{Ipv4Addr, UdpSocket};

use anyhow::Context;
use dns_starter_rust::message::{
    Label, Message, QuestionClass, QuestionType, ResourceClass, ResourceData, ResourceRecord,
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

                message
                    .ask("codecrafters.io", QuestionType::A, QuestionClass::IN)
                    .context("sending `codecrafters.io` question")?;

                message.answer(ResourceRecord {
                    name: Label::parse_str("codecrafters.io").unwrap(),
                    class: ResourceClass::IN,
                    time_to_live: 60,
                    data: ResourceData::Address(Ipv4Addr::new(8, 8, 8, 8)),
                });

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
