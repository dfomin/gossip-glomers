use std::collections::{HashMap, HashSet};

use anyhow::{Result, bail};
use rand::{rng, seq::IndexedRandom};
use tokio::sync::{mpsc::Sender, oneshot};

use crate::{
    body::{Payload, ReadRPC},
    transport::{RPCData, SendData, TransportPayload},
    workload::Workload,
};

#[derive(Default)]
pub struct WorkloadBroadcast {
    node: String,
    topology: HashMap<String, Vec<String>>,
    gossip_targets: Vec<String>,
    messages: HashSet<u32>,
    to_send: HashMap<u32, HashSet<String>>,
}

impl WorkloadBroadcast {}

impl Workload for WorkloadBroadcast {
    fn init(&mut self, _node_id: u32, node: String) {
        self.node = node;
    }

    async fn handle(
        &mut self,
        tx: Sender<TransportPayload>,
        payload: Payload,
        dest: String,
        msg_id: Option<u64>,
    ) -> Result<()> {
        match payload {
            Payload::Broadcast { message } => {
                let is_new = self.messages.insert(message);
                if is_new {
                    self.to_send
                        .entry(message)
                        .or_default()
                        .insert(dest.clone());
                }

                tx.send(TransportPayload::Send(SendData {
                    payload: Payload::BroadcastOk,
                    dest,
                    in_reply_to: msg_id,
                }))
                .await?;
            }
            Payload::BroadcastBatch { messages } => {
                let mut new_values = HashSet::new();
                for message in messages {
                    if self.messages.insert(message) {
                        new_values.insert(message);
                    }
                }

                for value in new_values {
                    self.to_send.entry(value).or_default().insert(dest.clone());
                }

                tx.send(TransportPayload::Send(SendData {
                    payload: Payload::BroadcastBatchOk,
                    dest,
                    in_reply_to: msg_id,
                }))
                .await?;
            }
            Payload::BroadcastBatchOk => (),
            Payload::Read { .. } => {
                tx.send(TransportPayload::Send(SendData {
                    payload: Payload::ReadOk {
                        result: ReadRPC::Broadcast {
                            messages: self.messages.clone(),
                        },
                    },
                    dest,
                    in_reply_to: msg_id,
                }))
                .await?
            }
            Payload::Topology { topology } => {
                self.topology = topology;
                let mut rng = rng();
                let targets_count = (2 * self.topology.len()).isqrt().min(self.topology.len());
                self.gossip_targets = self
                    .topology
                    .keys()
                    .cloned()
                    .filter(|node| node != &self.node)
                    .collect::<Vec<_>>()
                    .sample(&mut rng, targets_count)
                    .cloned()
                    .collect();
                tx.send(TransportPayload::Send(SendData {
                    payload: Payload::TopologyOk,
                    dest,
                    in_reply_to: msg_id,
                }))
                .await?
            }
            _ => bail!("Unsupported"),
        }
        Ok(())
    }

    async fn gossip(&mut self, tx: Sender<TransportPayload>) -> Result<()> {
        for neighbor in &self.gossip_targets {
            let tx = tx.clone();
            let neighbor = neighbor.to_string();
            let to_send = self
                .to_send
                .iter()
                .filter_map(|(k, v)| {
                    if !v.contains(&neighbor) {
                        Some(*k)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            if !to_send.is_empty() {
                tokio::spawn(async move {
                    let (reply_tx, reply_rx) = oneshot::channel();
                    _ = tx
                        .send(TransportPayload::RPC(RPCData {
                            payload: Payload::BroadcastBatch { messages: to_send },
                            dest: neighbor,
                            reply_tx,
                        }))
                        .await;
                    let _reply_message = reply_rx.await;
                });
            }
        }

        self.to_send.clear();

        Ok(())
    }
}
