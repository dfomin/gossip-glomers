use std::collections::HashMap;

use anyhow::{Result, bail};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    body::{Body, ReadRPC},
    message::Message,
    stage::Stage,
};

pub struct Node {
    id: Option<u32>,
    last_message_id: u32,
    messages: Vec<u32>,
    value: u32,
    message_queue: HashMap<u32, Message>,

    stage: Stage,
    rx: Receiver<Message>,
    tx: Sender<Message>,
}

impl Node {
    pub fn new(rx: Receiver<Message>, tx: Sender<Message>, stage: Stage) -> Self {
        Self {
            id: None,
            last_message_id: 0,
            messages: Vec::new(),
            value: 0,
            message_queue: HashMap::new(),
            stage,
            rx,
            tx,
        }
    }

    pub async fn process(&mut self) -> Result<()> {
        while let Some(message) = self.rx.recv().await {
            let reply_body = match &message.body {
                Body::Init {
                    msg_id, node_id, ..
                } => {
                    let id = node_id.chars().skip(1).collect::<String>().parse()?;
                    self.init(id)?;
                    Body::InitOk {
                        msg_id: self.generate()?,
                        in_reply_to: *msg_id,
                    }
                }
                Body::Echo { msg_id, echo } => Body::EchoOk {
                    msg_id: self.generate()?,
                    in_reply_to: *msg_id,
                    echo: echo.clone(),
                },
                Body::Generate { msg_id } => {
                    let new_id = self.generate()?;
                    Body::GenerateOk {
                        msg_id: self.generate()?,
                        in_reply_to: *msg_id,
                        id: new_id,
                    }
                }
                Body::Broadcast { msg_id, message } => {
                    self.add_message(*message);
                    Body::BroadcastOk {
                        msg_id: self.generate()?,
                        in_reply_to: *msg_id,
                    }
                }
                Body::Read { msg_id, .. } => match self.stage {
                    Stage::Stage3 => Body::ReadOk {
                        msg_id: Some(self.generate()?),
                        in_reply_to: *msg_id,
                        result: ReadRPC::Stage3 {
                            messages: self.messages.clone(),
                        },
                    },
                    Stage::Stage4 => {
                        self.read(message.clone()).await?;
                        Body::Noop
                    }
                },
                Body::ReadOk {
                    in_reply_to,
                    result: ReadRPC::Stage4 { value },
                    ..
                } => {
                    self.value = *value;
                    let saved_message = self.message_queue.remove(in_reply_to).unwrap();
                    let body = match &saved_message.body {
                        Body::Read { msg_id, .. } => Body::ReadOk {
                            msg_id: Some(self.generate()?),
                            in_reply_to: *msg_id,
                            result: ReadRPC::Stage4 { value: *value },
                        },
                        Body::Add { delta, .. } => {
                            self.add(*delta, saved_message.clone()).await?;
                            Body::Noop
                        }
                        _ => panic!("Unsupported enum type {:?}", saved_message.body),
                    };
                    if !matches!(body, Body::Noop) {
                        let reply = saved_message.reply(body);
                        self.tx.send(reply).await?;
                    }
                    Body::Noop
                }
                Body::Topology { msg_id, .. } => Body::TopologyOk {
                    msg_id: self.generate()?,
                    in_reply_to: *msg_id,
                },
                Body::Add { delta, .. } => {
                    self.add(*delta, message.clone()).await?;
                    Body::Noop
                }
                Body::CasOk { in_reply_to, .. } => {
                    let saved_message = self.message_queue.remove(in_reply_to).unwrap();
                    let body = match &saved_message.body {
                        Body::Add { msg_id, .. } => Body::AddOk {
                            msg_id: self.generate()?,
                            in_reply_to: *msg_id,
                        },
                        _ => panic!("Unsupported enum type {:?}", saved_message.body),
                    };
                    let reply = saved_message.reply(body);
                    self.tx.send(reply).await?;
                    Body::Noop
                }
                Body::Error {
                    in_reply_to,
                    code,
                    text,
                    ..
                } => {
                    let saved_message = self.message_queue.remove(in_reply_to).unwrap();
                    let body = match &saved_message.body {
                        Body::Read { msg_id, .. } => {
                            if *code == 20 {
                                Body::ReadOk {
                                    msg_id: Some(self.generate()?),
                                    in_reply_to: *msg_id,
                                    result: ReadRPC::Stage4 { value: 0 },
                                }
                            } else {
                                Body::Error {
                                    msg_id: Some(self.generate()?),
                                    in_reply_to: *msg_id,
                                    code: *code,
                                    text: text.clone(),
                                }
                            }
                        }
                        Body::Add { msg_id, .. } => {
                            if *code == 22 {
                                self.read(saved_message.clone()).await?;
                                Body::Noop
                            } else {
                                Body::Error {
                                    msg_id: Some(self.generate()?),
                                    in_reply_to: *msg_id,
                                    code: *code,
                                    text: text.clone(),
                                }
                            }
                        }
                        _ => panic!("Unsupported enum type {:?}", saved_message.body),
                    };
                    if !matches!(body, Body::Noop) {
                        let reply = saved_message.reply(body);
                        self.tx.send(reply).await?;
                    }
                    Body::Noop
                }
                _ => panic!("Unsupported enum type {:?}", message.body),
            };

            if !matches!(reply_body, Body::Noop) {
                let reply = message.reply(reply_body);
                self.tx.send(reply).await?;
            }
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

    fn generate(&mut self) -> Result<u32> {
        let Some(id) = self.id else {
            bail!("Node is not initialized");
        };
        self.last_message_id += 1;
        Ok((id << 16) + self.last_message_id)
    }

    fn add_message(&mut self, message: u32) {
        self.messages.push(message);
    }

    async fn add(&mut self, delta: u32, message: Message) -> Result<()> {
        let msg_id = self.generate()?;
        let request = Message {
            src: format!("n{}", self.id.unwrap()),
            dest: "seq-kv".to_string(),
            body: Body::Cas {
                msg_id,
                key: "g-counter".to_string(),
                from: self.value,
                to: self.value + delta,
                create_if_not_exists: true,
            },
        };
        self.value += delta;
        self.message_queue.insert(msg_id, message);
        self.tx.send(request).await?;
        Ok(())
    }

    async fn read(&mut self, message: Message) -> Result<()> {
        let msg_id = self.generate()?;
        let request = Message {
            src: format!("n{}", self.id.unwrap()),
            dest: "seq-kv".to_string(),
            body: Body::Read {
                msg_id: msg_id,
                key: Some("g-counter".to_string()),
            },
        };
        self.message_queue.insert(msg_id, message);
        self.tx.send(request).await?;
        Ok(())
    }
}
