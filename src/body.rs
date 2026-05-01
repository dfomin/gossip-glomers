use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ReadRPC {
    Stage3 { messages: Vec<u32> },
    Stage4 { value: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Body {
    Init {
        msg_id: u32,
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {
        msg_id: u32,
        in_reply_to: u32,
    },
    Echo {
        msg_id: u32,
        echo: String,
    },
    EchoOk {
        msg_id: u32,
        in_reply_to: u32,
        echo: String,
    },
    Generate {
        msg_id: u32,
    },
    GenerateOk {
        msg_id: u32,
        in_reply_to: u32,
        id: u32,
    },
    Broadcast {
        msg_id: u32,
        message: u32,
    },
    BroadcastOk {
        msg_id: u32,
        in_reply_to: u32,
    },
    Read {
        msg_id: u32,
        key: Option<String>,
    },
    ReadOk {
        msg_id: Option<u32>,
        in_reply_to: u32,
        #[serde(flatten)]
        result: ReadRPC,
    },
    Topology {
        msg_id: u32,
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk {
        msg_id: u32,
        in_reply_to: u32,
    },
    Add {
        msg_id: u32,
        delta: u32,
    },
    AddOk {
        msg_id: u32,
        in_reply_to: u32,
    },
    Cas {
        msg_id: u32,
        key: String,
        from: u32,
        to: u32,
        create_if_not_exists: bool,
    },
    CasOk {
        msg_id: Option<u32>,
        in_reply_to: u32,
    },
    Error {
        msg_id: Option<u32>,
        in_reply_to: u32,
        code: u32,
        text: String,
    },
    Noop,
}
