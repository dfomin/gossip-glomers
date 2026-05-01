mod body;
mod message;
mod node;
mod stage;

use anyhow::Result;
use tokio::{
    io::{AsyncBufReadExt, BufReader, stdin},
    sync::mpsc,
};

use crate::{message::Message, node::Node, stage::Stage};

#[tokio::main]
async fn main() -> Result<()> {
    let (stdout_tx, mut stdout_rx) = mpsc::channel(32);
    let stdout_handle = tokio::spawn(async move {
        while let Some(reply) = stdout_rx.recv().await {
            println!("{}", serde_json::to_string(&reply)?);
        }
        Ok::<(), anyhow::Error>(())
    });

    let (tx, rx) = mpsc::channel(32);
    let mut node = Node::new(rx, stdout_tx, Stage::Stage4);
    let node_handle = tokio::spawn(async move {
        node.process().await?;
        Ok::<(), anyhow::Error>(())
    });

    let stdin_handle = tokio::spawn(async move {
        let mut lines = BufReader::new(stdin()).lines();
        while let Some(line) = lines.next_line().await? {
            let message: Message = serde_json::from_str(&line)?;
            tx.send(message).await?;
        }
        Ok::<(), anyhow::Error>(())
    });

    node_handle.await??;
    stdout_handle.await??;
    stdin_handle.await??;

    Ok(())
}
