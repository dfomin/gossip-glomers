use anyhow::Result;
use gossip_glomers::{run, workload::WorkloadGenerate};

#[tokio::main]
async fn main() -> Result<()> {
    run(WorkloadGenerate::default()).await
}
