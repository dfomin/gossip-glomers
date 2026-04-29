use anyhow::{Result, bail};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{body::Body, message::Message};

pub struct Node {
    id: Option<u32>,
    last_message_id: u64,
    messages: Vec<u32>,

    rx: Receiver<(Message, Sender<Message>)>,
}

impl Node {
    pub fn new(rx: Receiver<(Message, Sender<Message>)>) -> Self {
        Self {
            id: None,
            last_message_id: 0,
            messages: Vec::new(),
            rx,
        }
    }

    pub async fn process(&mut self) -> Result<()> {
        while let Some((message, tx)) = self.rx.recv().await {
            let reply_body = match &message.body {
                Body::Init {
                    msg_id, node_id, ..
                } => {
                    let id = node_id.chars().skip(1).collect::<String>().parse()?;
                    self.init(id)?;
                    Body::InitOk {
                        in_reply_to: *msg_id,
                    }
                }
                Body::Echo { msg_id, echo } => Body::EchoOk {
                    msg_id: *msg_id,
                    in_reply_to: *msg_id,
                    echo: echo.clone(),
                },
                Body::Generate { msg_id } => {
                    let new_id = self.generate()?;
                    Body::GenerateOk {
                        msg_id: *msg_id,
                        in_reply_to: *msg_id,
                        id: new_id,
                    }
                }
                Body::Broadcast { msg_id, message } => {
                    self.add_message(*message);
                    Body::BroadcastOk {
                        msg_id: *msg_id,
                        in_reply_to: *msg_id,
                    }
                }
                Body::Read { msg_id } => Body::ReadOk {
                    msg_id: *msg_id,
                    in_reply_to: *msg_id,
                    messages: self.messages.clone(),
                },
                Body::Topology { msg_id, .. } => Body::TopologyOk {
                    msg_id: *msg_id,
                    in_reply_to: *msg_id,
                },
                _ => panic!("Unsupported enum type {:?}", message.body),
            };
            let reply = message.reply(reply_body);
            tx.send(reply).await?;
        }
        Ok(())
    }

    fn init(&mut self, id: u32) -> Result<()> {
        match self.id {
            Some(current_id) => bail!(
                "Cannot initialize with id {}, already initialized with id {}",
                id,
                current_id,
            ),

            None => {
                self.id = Some(id);
                Ok(())
            }
        }
    }

    fn generate(&mut self) -> Result<u64> {
        let Some(id) = self.id else {
            bail!("Node is not initialized");
        };
        self.last_message_id += 1;
        Ok(((id as u64) << 32) + self.last_message_id as u64)
    }

    fn add_message(&mut self, message: u32) {
        self.messages.push(message);
    }
}
