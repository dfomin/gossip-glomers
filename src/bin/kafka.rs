use anyhow::Result;
use gossip_glomers::{run, workload::workload_kafka::WorkloadKafka};

#[tokio::main]
async fn main() -> Result<()> {
    run(WorkloadKafka::default()).await
}
