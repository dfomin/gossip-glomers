use std::{collections::HashMap, time::Duration};

use anyhow::Result;
use futures::{StreamExt, future::BoxFuture, stream::FuturesUnordered};
use tokio::{
    select,
    sync::{mpsc, oneshot},
    time,
};

use crate::{
    body::{Body, Payload},
    message::Message,
};

pub struct SendData {
    pub payload: Payload,
    pub dest: String,
    pub in_reply_to: Option<u64>,
}

pub struct RPCData {
    pub payload: Payload,
    pub dest: String,
    pub reply_tx: oneshot::Sender<Message>,
}

pub enum TransportPayload {
    Init(String, u32),
    Send(SendData),
    RPC(RPCData),
}

struct NodeInfo {
    node: String,
    node_id: u32,
}

pub struct Transport {
    node_info: Option<NodeInfo>,
    last_message_id: u32,
    stdout_tx: mpsc::Sender<Message>,
    stdin_rx: mpsc::Receiver<Message>,
    node_tx: mpsc::Sender<Message>,
    transport_rx: mpsc::Receiver<TransportPayload>,
    pending: HashMap<u64, oneshot::Sender<Message>>,
    futures: FuturesUnordered<BoxFuture<'static, Message>>,
}

impl Transport {
    pub fn new(
        stdout_tx: mpsc::Sender<Message>,
        stdin_rx: mpsc::Receiver<Message>,
        node_tx: mpsc::Sender<Message>,
        transport_rx: mpsc::Receiver<TransportPayload>,
    ) -> Self {
        Self {
            node_info: None,
            last_message_id: 0,
            stdout_tx,
            stdin_rx,
            node_tx,
            transport_rx,
            pending: HashMap::new(),
            futures: FuturesUnordered::new(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            select! {
                Some(message) = self.stdin_rx.recv() => {
                    if let Some(in_reply_to) = message.body.in_reply_to {
                        if let Some(reply_channel) = self.pending.remove(&in_reply_to) {
                            _ = reply_channel.send(message);
                        }
                    } else {
                        self.node_tx.send(message).await?;
                    }
                }
                Some(transport_payload) = self.transport_rx.recv() => {
                    match transport_payload {
                        TransportPayload::Init(node, node_id) => {
                            self.node_info = Some(NodeInfo { node, node_id});
                        }
                        TransportPayload::Send(data) => {
                            self.send(data).await?;
                        }
                        TransportPayload::RPC(data) => {
                            self.rpc(data).await?;
                        }
                    }
                }
                Some(message) = self.futures.next() => {
                    if let Some(msg_id) = message.body.msg_id && self.pending.contains_key(&msg_id) {
                        self.send_retryable(message).await?;
                    }
                }
                else => break,
            }
        }
        Ok(())
    }

    async fn send(&mut self, data: SendData) -> Result<()> {
        let body = Body {
            msg_id: Some(self.generate()),
            in_reply_to: data.in_reply_to,
            payload: data.payload,
        };
        let message = self.message(body, &data.dest);
        self.stdout_tx.send(message).await?;
        Ok(())
    }

    async fn rpc(&mut self, data: RPCData) -> Result<()> {
        let msg_id = self.generate();
        let body = Body {
            msg_id: Some(msg_id),
            in_reply_to: None,
            payload: data.payload,
        };
        self.pending.insert(msg_id, data.reply_tx);
        let message = self.message(body, &data.dest);
        self.send_retryable(message).await?;
        Ok(())
    }

    async fn send_retryable(&mut self, message: Message) -> Result<()> {
        self.stdout_tx.send(message.clone()).await?;
        self.futures.push(Box::pin(async move {
            time::sleep(Duration::from_millis(100)).await;
            message
        }));
        Ok(())
    }

    fn message(&self, body: Body, dest: &str) -> Message {
        let Some(NodeInfo { node, .. }) = self.node_info.as_ref() else {
            panic!("Init should be the first command")
        };
        Message {
            src: node.clone(),
            dest: dest.to_string(),
            body,
        }
    }

    fn generate(&mut self) -> u64 {
        let Some(NodeInfo { node_id, .. }) = self.node_info else {
            panic!("Not initialized");
        };
        self.last_message_id += 1;
        ((node_id as u64) << 32) + self.last_message_id as u64
    }
}
