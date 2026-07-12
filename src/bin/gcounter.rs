use anyhow::Result;
use gossip_glomers::{run, workload::workload_gcounter::WorkloadGcounter};

#[tokio::main]
async fn main() -> Result<()> {
    run(WorkloadGcounter::default()).await
}
