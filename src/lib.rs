mod body;
mod message;
mod node;
mod transport;
pub mod workload;

use anyhow::Result;
use tokio::{
    io::{AsyncBufReadExt, BufReader, stdin},
    sync::mpsc,
};

use crate::{message::Message, node::Node, transport::Transport, workload::Workload};

pub async fn run<W: Workload>(workload: W) -> Result<()> {
    let (stdout_tx, mut stdout_rx) = mpsc::channel(32);
    let (stdin_tx, stdin_rx) = mpsc::channel(32);
    let (node_tx, node_rx) = mpsc::channel(32);
    let (transport_tx, transport_rx) = mpsc::channel(32);

    let stdout_handle = tokio::spawn(async move {
        while let Some(reply) = stdout_rx.recv().await {
            println!("{}", serde_json::to_string(&reply)?);
        }
        Ok::<(), anyhow::Error>(())
    });

    let stdin_handle = tokio::spawn(async move {
        let mut lines = BufReader::new(stdin()).lines();
        while let Some(line) = lines.next_line().await? {
            let message: Message = serde_json::from_str(&line)?;
            stdin_tx.send(message).await?;
        }
        Ok::<(), anyhow::Error>(())
    });

    let mut transport = Transport::new(stdout_tx, stdin_rx, node_tx, transport_rx);
    let transport_handle = tokio::spawn(async move { transport.run().await });

    let mut node = Node::new(node_rx, transport_tx, workload);
    node.run().await?;

    stdout_handle.await??;
    stdin_handle.await??;
    transport_handle.await??;

    Ok(())
}
