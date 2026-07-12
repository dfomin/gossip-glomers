use anyhow::{Result, bail};
use tokio::sync::mpsc::Sender;

use crate::{
    body::Payload,
    transport::{SendData, TransportPayload},
    workload::Workload,
};

#[derive(Default)]
pub struct WorkloadEcho {}

impl Workload for WorkloadEcho {
    async fn handle(
        &mut self,
        tx: Sender<TransportPayload>,
        payload: Payload,
        dest: String,
        msg_id: Option<u64>,
    ) -> Result<()> {
        match payload {
            Payload::Echo { echo } => {
                tx.send(TransportPayload::Send(SendData {
                    payload: Payload::EchoOk { echo },
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
