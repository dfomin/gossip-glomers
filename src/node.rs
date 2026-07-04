use anyhow::Result;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    body::Payload,
    message::Message,
    transport::{SendData, TransportPayload},
    workload::Workload,
};

pub struct Node<W: Workload> {
    workload: W,
    rx: Receiver<Message>,
    tx: Sender<TransportPayload>,
}

impl<W: Workload> Node<W> {
    pub fn new(rx: Receiver<Message>, tx: Sender<TransportPayload>, workload: W) -> Self {
        Self { workload, rx, tx }
    }

    pub async fn run(&mut self) -> Result<()> {
        while let Some(message) = self.rx.recv().await {
            let dest = message.src;
            let msg_id = message.body.msg_id;
            match message.body.payload {
                Payload::Init { node_id, node_ids } => {
                    let id = node_id
                        .chars()
                        .skip(1)
                        .collect::<String>()
                        .parse()
                        .expect("Correct node name");
                    self.workload.init(id, node_id.to_string());
                    self.tx.send(TransportPayload::Init(node_id, id)).await?;
                    self.tx
                        .send(TransportPayload::Send(SendData {
                            payload: Payload::InitOk,
                            dest,
                            in_reply_to: Some(msg_id),
                        }))
                        .await?;
                }
                payload => {
                    self.workload
                        .handle(self.tx.clone(), payload, dest, msg_id)
                        .await?
                }
            };
        }
        Ok(())
    }
}
