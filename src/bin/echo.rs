use anyhow::Result;
use gossip_glomers::{run, workload::workload_echo::WorkloadEcho};

#[tokio::main]
async fn main() -> Result<()> {
    run(WorkloadEcho::default()).await
}
