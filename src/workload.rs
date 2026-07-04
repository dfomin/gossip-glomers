use std::collections::{HashMap, HashSet};

use anyhow::{Result, bail};
use tokio::sync::mpsc;

use crate::{
    body::Payload,
    transport::{SendData, TransportPayload},
};

#[allow(async_fn_in_trait)]
pub trait Workload {
    fn init(&mut self, _node_id: u32, _node: String) {}

    async fn handle(
        &mut self,
        tx: mpsc::Sender<TransportPayload>,
        payload: Payload,
        dest: String,
        msg_id: u64,
    ) -> Result<()>;
}

#[derive(Default)]
pub struct WorkloadEcho {}

impl Workload for WorkloadEcho {
    async fn handle(
        &mut self,
        tx: mpsc::Sender<TransportPayload>,
        payload: Payload,
        dest: String,
        msg_id: u64,
    ) -> Result<()> {
        match payload {
            Payload::Echo { echo } => {
                tx.send(TransportPayload::Send(SendData {
                    payload: Payload::EchoOk { echo },
                    dest,
                    in_reply_to: Some(msg_id),
                }))
                .await?;
            }
            _ => bail!("Unsupported"),
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct WorkloadGenerate {
    node_id: u32,
    last_message_id: u32,
}

impl WorkloadGenerate {
    fn generate(&mut self) -> u64 {
        self.last_message_id += 1;
        ((self.node_id as u64) << 32) + self.last_message_id as u64
    }
}

impl Workload for WorkloadGenerate {
    fn init(&mut self, node_id: u32, _node: String) {
        self.node_id = node_id;
    }

    async fn handle(
        &mut self,
        tx: mpsc::Sender<TransportPayload>,
        payload: Payload,
        dest: String,
        msg_id: u64,
    ) -> Result<()> {
        match payload {
            Payload::Generate => {
                let send_payload = Payload::GenerateOk {
                    id: self.generate(),
                };
                tx.send(TransportPayload::Send(SendData {
                    payload: send_payload,
                    dest,
                    in_reply_to: Some(msg_id),
                }))
                .await?
            }
            _ => bail!("Unsupported"),
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct WorkloadBroadcast {
    node: String,
    topology: HashMap<String, Vec<String>>,
    messages: HashSet<u32>,
}

impl Workload for WorkloadBroadcast {
    fn init(&mut self, _node_id: u32, node: String) {
        self.node = node;
    }

    async fn handle(
        &mut self,
        tx: mpsc::Sender<TransportPayload>,
        payload: Payload,
        dest: String,
        msg_id: u64,
    ) -> Result<()> {
        match payload {
            Payload::Broadcast { message } => {
                let is_new = self.messages.insert(message);
                tx.send(TransportPayload::Send(SendData {
                    payload: Payload::BroadcastOk,
                    dest,
                    in_reply_to: Some(msg_id),
                }))
                .await?;

                if is_new {
                    for neighbor in self.topology.get(&self.node).into_iter().flatten() {
                        tx.send(TransportPayload::Send(SendData {
                            payload: Payload::Broadcast { message },
                            dest: neighbor.to_string(),
                            in_reply_to: None,
                        }))
                        .await?;
                    }
                }
            }
            Payload::BroadcastOk => (),
            Payload::Read => {
                tx.send(TransportPayload::Send(SendData {
                    payload: Payload::ReadOk {
                        messages: self.messages.clone(),
                    },
                    dest,
                    in_reply_to: Some(msg_id),
                }))
                .await?
            }
            Payload::Topology { topology } => {
                self.topology = topology;
                tx.send(TransportPayload::Send(SendData {
                    payload: Payload::TopologyOk,
                    dest,
                    in_reply_to: Some(msg_id),
                }))
                .await?
            }
            _ => bail!("Unsupported"),
        }
        Ok(())
    }
}
