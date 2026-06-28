use anyhow::Result;
use tokio::{
    select,
    sync::{mpsc, oneshot},
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
    payload: Payload,
    dest: String,
    in_reply_to: Option<u64>,
    reply_channel: oneshot::Sender<Message>,
}

pub enum TransportPayload {
    Init(String),
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
    // pending: HashMap<u64, oneshot::Sender<Body>>,
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
            //     pending: HashMap::new(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            select! {
                message = self.stdin_rx.recv() => {
                    if let Some(message) = message {
                        self.node_tx.send(message).await?;
                    } else {
                        break;
                    }
                }
                transport_payload = self.transport_rx.recv() => {
                    if let Some(transport_payload) = transport_payload {
                        match transport_payload {
                            TransportPayload::Init(node_id) => self.init(node_id),
                            TransportPayload::Send(data) => {
                                self.send(data).await?;
                            }
                            TransportPayload::RPC(data) => (),
                        }
                    } else {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn send(&mut self, data: SendData) -> Result<()> {
        let body = Body {
            msg_id: self.generate(),
            in_reply_to: data.in_reply_to,
            payload: data.payload,
        };
        let message = self.message(body, &data.dest);
        self.stdout_tx.send(message).await?;
        Ok(())
    }

    pub async fn rpc(
        &mut self,
        payload: Payload,
        dest: &str,
        in_reply_to: Option<u64>,
    ) -> Result<()> {
        let body = Body {
            msg_id: self.generate(),
            in_reply_to: in_reply_to,
            payload,
        };
        let message = self.message(body, dest);
        self.stdout_tx.send(message).await?;
        Ok(())
    }

    pub fn init(&mut self, node: String) {
        let node_id = node
            .chars()
            .skip(1)
            .collect::<String>()
            .parse()
            .expect("Correct node name");
        self.node_info = Some(NodeInfo { node, node_id });
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
