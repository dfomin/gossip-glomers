mod body;
mod message;

use std::io::{BufRead, stdin};

use anyhow::Result;

use crate::{body::Body, message::Message};

#[tokio::main]
async fn main() -> Result<()> {
    let reader = stdin().lock();
    for line in reader.lines() {
        let message: Message = serde_json::from_str(&line?)?;
        let reply = match message.body {
            Body::Init { msg_id, .. } => Message {
                src: message.dest,
                dest: message.src,
                body: Body::InitOk {
                    in_reply_to: msg_id,
                },
            },
            Body::Echo { msg_id, echo } => Message {
                src: message.dest,
                dest: message.src,
                body: Body::EchoOk {
                    msg_id,
                    in_reply_to: msg_id,
                    echo,
                },
            },
            _ => panic!("Unsupported command"),
        };
        println!("{}", serde_json::to_string(&reply)?);
    }
    Ok(())
}
