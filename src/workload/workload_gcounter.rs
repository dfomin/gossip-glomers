use anyhow::{Result, bail};
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    oneshot,
};

use crate::{
    body::Payload,
    transport::{RPCData, SendData, TransportPayload},
    workload::Workload,
};

struct CasActorPayload {
    tx: Sender<crate::transport::TransportPayload>,
    delta: u32,
    dest: String,
    msg_id: Option<u64>,
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
            WorkloadGcounter::add(payload.tx.clone(), payload.delta).await?;
            _ = payload
                .tx
                .send(TransportPayload::Send(SendData {
                    payload: Payload::AddOk,
                    dest: payload.dest,
                    in_reply_to: payload.msg_id,
                }))
                .await;
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct WorkloadGcounter {
    cas_tx: Option<Sender<CasActorPayload>>,
}

impl WorkloadGcounter {
    async fn add(tx: Sender<crate::transport::TransportPayload>, delta: u32) -> Result<()> {
        loop {
            let value = WorkloadGcounter::read(tx.clone()).await?;
            let (reply_tx, reply_rx) = oneshot::channel();
            _ = tx
                .send(TransportPayload::RPC(RPCData {
                    payload: Payload::Cas {
                        key: "g-counter".to_string(),
                        from: value,
                        to: value + delta,
                        create_if_not_exists: true,
                    },
                    dest: "seq-kv".to_string(),
                    reply_tx,
                }))
                .await;
            let reply_message = reply_rx.await?;
            match reply_message.body.payload {
                Payload::CasOk => {
                    return Ok(());
                }
                Payload::Error { code, .. } => match code {
                    22 => (),
                    _ => bail!("Unexpected seq-kv error"),
                },
                _ => panic!("Unexpected"),
            }
        }
    }

    async fn read(tx: Sender<crate::transport::TransportPayload>) -> Result<u32> {
        let (reply_tx, reply_rx) = oneshot::channel();
        _ = tx
            .send(TransportPayload::RPC(RPCData {
                payload: Payload::Read {
                    key: Some("g-counter".to_string()),
                },
                dest: "seq-kv".to_string(),
                reply_tx,
            }))
            .await;
        let reply_message = reply_rx.await?;
        match reply_message.body.payload {
            Payload::ReadOk {
                result: crate::body::ReadRPC::Gcounter { value },
            } => Ok(value),
            Payload::Error { code, .. } => match code {
                20 => Ok(0),
                _ => bail!("Unexpected seq-kv error"),
            },
            _ => bail!("Unexpected"),
        }
    }
}

impl Workload for WorkloadGcounter {
    fn init(&mut self, _node_id: u32, _node: String) {
        let (tx, rx) = mpsc::channel(1024);
        self.cas_tx = Some(tx);
        tokio::spawn(async move { CasActor::new(rx).run().await });
    }

    async fn handle(
        &mut self,
        tx: Sender<crate::transport::TransportPayload>,
        payload: Payload,
        dest: String,
        msg_id: Option<u64>,
    ) -> anyhow::Result<()> {
        match payload {
            Payload::Add { delta } => {
                if let Some(cas_tx) = &self.cas_tx {
                    cas_tx
                        .send(CasActorPayload {
                            tx: tx.clone(),
                            delta,
                            dest,
                            msg_id,
                        })
                        .await?;
                }
            }
            Payload::Read { .. } => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    _ = WorkloadGcounter::add(tx.clone(), 0).await;
                    if let Ok(value) = WorkloadGcounter::read(tx.clone()).await {
                        _ = tx
                            .send(TransportPayload::Send(SendData {
                                payload: Payload::ReadOk {
                                    result: crate::body::ReadRPC::Gcounter { value },
                                },
                                dest,
                                in_reply_to: msg_id,
                            }))
                            .await;
                    }
                });
            }
            _ => bail!("Unsupported"),
        }
        Ok(())
    }
}
