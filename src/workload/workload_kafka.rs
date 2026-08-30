use std::collections::HashMap;

use anyhow::{Result, bail};
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    oneshot,
};

use crate::{
    body::{CasRPC, Payload, ReadRPC},
    transport::{RPCData, SendData, TransportPayload},
    workload::Workload,
};

struct CasActorPayload {
    tx: Sender<TransportPayload>,
    key: String,
    value: u64,
    dest: String,
    msg_id: Option<u64>,
    offset: bool,
}

struct CasActor {
    rx: Receiver<CasActorPayload>,
}

impl CasActor {
    fn new(rx: Receiver<CasActorPayload>) -> Self {
        Self { rx }
    }

    async fn run(&mut self) -> Result<()> {
        while let Some(payload) = self.rx.recv().await {
            if !payload.offset {
                let offset =
                    WorkloadKafka::add(payload.tx.clone(), &payload.key, payload.value).await?;
                _ = payload
                    .tx
                    .send(TransportPayload::Send(SendData {
                        payload: Payload::SendOk { offset },
                        dest: payload.dest,
                        in_reply_to: payload.msg_id,
                    }))
                    .await;
            } else {
                WorkloadKafka::commit_offsets(payload.tx.clone(), &payload.key, payload.value)
                    .await?;
            }
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct WorkloadKafka {
    cas_tx: Option<Sender<CasActorPayload>>,
}

impl WorkloadKafka {
    async fn add(tx: Sender<TransportPayload>, key: &str, value: u64) -> Result<u64> {
        loop {
            let values = WorkloadKafka::read(tx.clone(), key).await?;
            let mut new_values = values.clone();
            let offset = new_values.len() as u64;
            new_values.push(value);
            let (reply_tx, reply_rx) = oneshot::channel();
            _ = tx
                .send(TransportPayload::RPC(RPCData {
                    payload: Payload::Cas {
                        data: CasRPC::Kafka {
                            key: "value_".to_string() + key,
                            from: values,
                            to: new_values,
                            create_if_not_exists: true,
                        },
                    },
                    dest: "lin-kv".to_string(),
                    reply_tx,
                }))
                .await;
            let reply_message = reply_rx.await?;
            match reply_message.body.payload {
                Payload::CasOk => {
                    return Ok(offset);
                }
                Payload::Error { code, .. } => match code {
                    22 => (),
                    _ => bail!("Unexpected lin-kv error"),
                },
                _ => panic!("Unexpected"),
            }
        }
    }

    async fn read(tx: Sender<TransportPayload>, key: &str) -> Result<Vec<u64>> {
        let (reply_tx, reply_rx) = oneshot::channel();
        _ = tx
            .send(TransportPayload::RPC(RPCData {
                payload: Payload::Read {
                    key: Some("value_".to_string() + key),
                },
                dest: "lin-kv".to_string(),
                reply_tx,
            }))
            .await;
        let reply_message = reply_rx.await?;
        match reply_message.body.payload {
            Payload::ReadOk {
                result: ReadRPC::Kafka { value },
            } => Ok(value),
            Payload::Error { code, .. } => match code {
                20 => Ok(vec![]),
                _ => bail!("Unexpected lin-kv error"),
            },
            _ => bail!("Unexpected"),
        }
    }

    async fn commit_offsets(tx: Sender<TransportPayload>, key: &str, value: u64) -> Result<u64> {
        loop {
            let offset = WorkloadKafka::read_offsets(tx.clone(), key).await?;
            if offset >= value {
                return Ok(offset);
            }
            let (reply_tx, reply_rx) = oneshot::channel();
            _ = tx
                .send(TransportPayload::RPC(RPCData {
                    payload: Payload::Cas {
                        data: CasRPC::Gcounter {
                            key: "offset_".to_string() + key,
                            from: offset,
                            to: value,
                            create_if_not_exists: true,
                        },
                    },
                    dest: "lin-kv".to_string(),
                    reply_tx,
                }))
                .await;
            let reply_message = reply_rx.await?;
            match reply_message.body.payload {
                Payload::CasOk => {
                    return Ok(value);
                }
                Payload::Error { code, .. } => match code {
                    22 => (),
                    _ => bail!("Unexpected lin-kv error"),
                },
                _ => panic!("Unexpected"),
            }
        }
    }

    async fn read_offsets(tx: Sender<TransportPayload>, key: &str) -> Result<u64> {
        let (reply_tx, reply_rx) = oneshot::channel();
        _ = tx
            .send(TransportPayload::RPC(RPCData {
                payload: Payload::Read {
                    key: Some("offset_".to_string() + key),
                },
                dest: "lin-kv".to_string(),
                reply_tx,
            }))
            .await;
        let reply_message = reply_rx.await?;
        match reply_message.body.payload {
            Payload::ReadOk {
                result: ReadRPC::Gcounter { value },
            } => Ok(value),
            Payload::Error { code, .. } => match code {
                20 => Ok(0),
                _ => bail!("Unexpected lin-kv error"),
            },
            _ => bail!("Unexpected"),
        }
    }
}

impl Workload for WorkloadKafka {
    fn init(&mut self, _node_id: u32, _node: String) {
        let (tx, rx) = mpsc::channel(1024);
        self.cas_tx = Some(tx);
        tokio::spawn(async move { CasActor::new(rx).run().await });
    }

    async fn handle(
        &mut self,
        tx: Sender<TransportPayload>,
        payload: Payload,
        dest: String,
        msg_id: Option<u64>,
    ) -> Result<()> {
        match payload {
            Payload::Send { key, msg } => {
                if let Some(cas_tx) = &self.cas_tx {
                    cas_tx
                        .send(CasActorPayload {
                            tx: tx.clone(),
                            key: key.clone(),
                            value: msg,
                            dest: dest.clone(),
                            msg_id,
                            offset: false,
                        })
                        .await?;
                }
            }
            Payload::Poll { offsets } => {
                let mut messages = HashMap::new();
                for (key, offset) in offsets {
                    let items = WorkloadKafka::read(tx.clone(), &key).await?;
                    let items = items
                        .iter()
                        .enumerate()
                        .skip(offset as usize)
                        .map(|(k, v)| vec![k as u64, *v])
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
                    WorkloadKafka::commit_offsets(tx.clone(), &key, offset).await?;
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
                    let offset = WorkloadKafka::read_offsets(tx.clone(), &key).await?;
                    offsets.insert(key.to_string(), offset);
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
