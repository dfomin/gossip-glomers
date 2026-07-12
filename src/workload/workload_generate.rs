use anyhow::{Result, bail};
use tokio::sync::mpsc::Sender;

use crate::{
    body::Payload,
    transport::{SendData, TransportPayload},
    workload::Workload,
};

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
        tx: Sender<TransportPayload>,
        payload: Payload,
        dest: String,
        msg_id: Option<u64>,
    ) -> Result<()> {
        match payload {
            Payload::Generate => {
                let send_payload = Payload::GenerateOk {
                    id: self.generate(),
                };
                tx.send(TransportPayload::Send(SendData {
                    payload: send_payload,
                    dest,
                    in_reply_to: msg_id,
                }))
                .await?
            }
            _ => bail!("Unsupported"),
        }
        Ok(())
    }
}
