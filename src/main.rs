mod body;
mod message;
mod node;
mod stage;
mod transport;

use anyhow::Result;
use tokio::{
    io::{AsyncBufReadExt, BufReader, stdin},
    sync::mpsc,
};

use crate::{message::Message, node::Node, stage::Stage, transport::Transport};

#[tokio::main]
async fn main() -> Result<()> {
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

    let mut transport = Transport::new(
        stdout_tx,
        stdin_rx,
        node_tx,
        transport_tx.clone(),
        transport_rx,
    );
    let transport_handle = tokio::spawn(async move { transport.run().await });

    let mut node = Node::new(node_rx, transport_tx, Stage::Stage4);
    let node_handle = tokio::spawn(async move { node.run().await });

    stdout_handle.await??;
    stdin_handle.await??;
    node_handle.await??;
    transport_handle.await?;

    Ok(())
}
