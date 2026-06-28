use anyhow::Result;
use gossip_glomers::{run, workload::WorkloadEcho};

#[tokio::main]
async fn main() -> Result<()> {
    run(WorkloadEcho::default()).await
}
