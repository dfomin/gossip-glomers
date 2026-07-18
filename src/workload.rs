pub mod workload_broadcast;
pub mod workload_echo;
pub mod workload_gcounter;
pub mod workload_generate;
pub mod workload_kafka;

use anyhow::Result;
use tokio::sync::mpsc::Sender;

use crate::{body::Payload, transport::TransportPayload};

#[allow(async_fn_in_trait)]
pub trait Workload {
    fn init(&mut self, _node_id: u32, _node: String) {}

    async fn handle(
        &mut self,
        tx: Sender<TransportPayload>,
        payload: Payload,
        dest: String,
        msg_id: Option<u64>,
    ) -> Result<()>;

    async fn gossip(&mut self, _tx: Sender<TransportPayload>) -> Result<()> {
        Ok(())
    }
}
