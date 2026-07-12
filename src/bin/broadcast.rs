use anyhow::Result;
use gossip_glomers::{run, workload::workload_broadcast::WorkloadBroadcast};

#[tokio::main]
async fn main() -> Result<()> {
    run(WorkloadBroadcast::default()).await
}
