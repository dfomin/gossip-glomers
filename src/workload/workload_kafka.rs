use std::collections::HashMap;

use anyhow::{Result, bail};
use tokio::sync::mpsc::Sender;

use crate::{
    body::Payload,
    transport::{SendData, TransportPayload},
    workload::Workload,
};

#[derive(Default)]
pub struct WorkloadKafka {
    log: HashMap<String, Vec<u64>>,
    offsets: HashMap<String, u64>,
}

impl Workload for WorkloadKafka {
    async fn handle(
        &mut self,
        tx: Sender<TransportPayload>,
        payload: Payload,
        dest: String,
        msg_id: Option<u64>,
    ) -> Result<()> {
        match payload {
            Payload::Send { key, msg } => {
                let entry = self.log.entry(key.to_string()).or_default();
                let key_offset = entry.len() as u64;
                entry.push(msg);
                let basic_offset = 0;
                tx.send(TransportPayload::Send(SendData {
                    payload: Payload::SendOk {
                        offset: basic_offset + key_offset,
                    },
                    dest,
                    in_reply_to: msg_id,
                }))
                .await?;
            }
            Payload::Poll { offsets } => {
                let mut messages = HashMap::new();
                for (key, offset) in offsets {
                    let basic_offset = 0;
                    let items = self
                        .log
                        .get(&key)
                        .unwrap_or(&Vec::new())
                        .iter()
                        .enumerate()
                        .skip(offset as usize)
                        .map(|(k, v)| vec![k as u64 + basic_offset, *v])
                        .collect();
                    messages.insert(key, items);
                }
                tx.send(TransportPayload::Send(SendData {
                    payload: Payload::PollOk { msgs: messages },
                    dest,
                    in_reply_to: msg_id,
                }))
                .await?;
            }
            Payload::CommitOffsets { offsets } => {
                for (key, offset) in offsets {
                    let entry = self.offsets.entry(key).or_insert(0);
                    *entry = offset.max(*entry);
                }
                tx.send(TransportPayload::Send(SendData {
                    payload: Payload::CommitOffsetsOk,
                    dest,
                    in_reply_to: msg_id,
                }))
                .await?;
            }
            Payload::ListCommittedOffsets { keys } => {
                let mut offsets = HashMap::new();
                for key in keys {
                    offsets.insert(key.to_string(), *self.offsets.get(&key).unwrap_or(&0));
                }
                tx.send(TransportPayload::Send(SendData {
                    payload: Payload::ListCommittedOffsetsOk { offsets },
                    dest,
                    in_reply_to: msg_id,
                }))
                .await?;
            }
            _ => bail!("Unsupported"),
        }
        Ok(())
    }
}
