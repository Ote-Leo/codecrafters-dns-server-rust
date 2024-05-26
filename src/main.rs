use std::{
    env::{args, Args},
    net::{Ipv4Addr, SocketAddrV4, UdpSocket},
};

use anyhow::Context;
use dns_starter_rust::message::{
    HeaderError, Message, OperationCode, ResourceClass, ResourceData, ResourceRecord,
};

fn read_resolver(mut args: Args) -> Option<SocketAddrV4> {
    args.next().and_then(|flag| {
        args.next().and_then(|address| {
            if flag == "--resolver" {
                address.parse().ok()
            } else {
                None
            }
        })
    })
}

fn main() -> anyhow::Result<()> {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    let mut args = args();
    args.next();

    let resolver = read_resolver(args);
    eprintln!("resolver: {resolver:?}");

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);
                let message_buf = &buf[..size];
                eprintln!("\nPacket: {:?}\n", message_buf);

                let mut message = match resolver {
                    Some(address) => forward_message(&address, message_buf),
                    None => quick_reply(message_buf),
                }?;

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

fn quick_reply(buf: &[u8]) -> anyhow::Result<Message> {
    let mut message: Message = buf.try_into().context("decoding query message")?;

    match message.header.operation_code {
        OperationCode::StandardQuery => message.header.response = Ok(()),
        _ => message.header.response = Err(HeaderError::NotImplemented),
    }

    let answers = message
        .questions
        .iter()
        .map(|q| ResourceRecord {
            name: q.name.clone(),
            class: ResourceClass::IN,
            time_to_live: 60,
            data: ResourceData::Address(Ipv4Addr::new(8, 8, 8, 8)),
        })
        .collect::<Vec<_>>();

    for answer in answers.into_iter() {
        message.answer(answer);
    }

    Ok(message)
}

fn forward_message(address: &SocketAddrV4, buf: &[u8]) -> anyhow::Result<Message> {
    let mut message: Message = buf.try_into().context("decoding query message")?;

    let header = {
        let mut header = message.header.clone();
        header.question_count = 1;
        header
    };

    let socket = UdpSocket::bind(address)?;
    let mut inner_buf = [0; 512];

    let questions = message.questions.clone();

    for question in questions.into_iter() {
        let question_message = Message {
            header: header.clone(),
            questions: vec![question.clone()],
            answers: vec![],
            authorities: vec![],
            additionals: vec![],
        };

        socket.send(&Vec::from(question_message))?;
        let (size, _) = socket.recv_from(&mut inner_buf)?;
        let mut reply = Message::try_from(&inner_buf[..size])?;
        message.answer(reply.answers.pop().unwrap());
    }

    match message.header.operation_code {
        OperationCode::StandardQuery => message.header.response = Ok(()),
        _ => message.header.response = Err(HeaderError::NotImplemented),
    }

    Ok(message)
}
