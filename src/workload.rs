use anyhow::{Result, bail};
use tokio::sync::mpsc;

use crate::{
    body::Payload,
    transport::{SendData, TransportPayload},
};

pub trait Workload {
    async fn handle(
        &mut self,
        tx: mpsc::Sender<TransportPayload>,
        payload: Payload,
        dest: String,
        msg_id: u64,
    ) -> Result<()>;
}

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
