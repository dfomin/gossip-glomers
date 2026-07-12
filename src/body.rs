use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum ReadRPC {
//     Stage3 { messages: Vec<u32> },
//     Stage4 { value: u32 },
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body {
    pub msg_id: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<u64>,

    #[serde(flatten)]
    pub payload: Payload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Payload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
    Generate,
    GenerateOk {
        id: u64,
    },
    Broadcast {
        message: u32,
    },
    BroadcastOk,
    BroadcastBatch {
        messages: Vec<u32>,
    },
    BroadcastBatchOk,
    Read,
    ReadOk {
        messages: HashSet<u32>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}
