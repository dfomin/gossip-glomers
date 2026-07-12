use anyhow::Result;
use gossip_glomers::{run, workload::workload_generate::WorkloadGenerate};

#[tokio::main]
async fn main() -> Result<()> {
    run(WorkloadGenerate::default()).await
}
