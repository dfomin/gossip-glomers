use anyhow::Result;
use gossip_glomers::{run, workload::WorkloadBroadcast};

#[tokio::main]
async fn main() -> Result<()> {
    run(WorkloadBroadcast::default()).await
}
