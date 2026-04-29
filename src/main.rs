mod body;
mod message;
mod node;

use std::io::{BufRead, stdin};

use anyhow::Result;
use tokio::sync::{mpsc, oneshot};

use crate::{message::Message, node::Node};

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel(32);
    let mut node = Node::new(rx);
    tokio::spawn(async move {
        _ = node.process().await;
    });
    let reader = stdin().lock();
    for line in reader.lines() {
        let message: Message = serde_json::from_str(&line?)?;
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        tx.send((message, oneshot_tx)).await?;
        let reply = oneshot_rx.await?;
        println!("{}", serde_json::to_string(&reply)?);
    }
    Ok(())
}
