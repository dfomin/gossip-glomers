use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ReadRPC {
    Broadcast { messages: HashSet<u32> },
    Gcounter { value: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body {
    pub msg_id: Option<u64>,

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
    Read {
        key: Option<String>,
    },
    ReadOk {
        #[serde(flatten)]
        result: ReadRPC,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
    Add {
        delta: u32,
    },
    AddOk,
    Cas {
        key: String,
        from: u32,
        to: u32,
        create_if_not_exists: bool,
    },
    CasOk,
    Error {
        code: u32,
        text: String,
    },
    Send {
        key: String,
        msg: u64,
    },
    SendOk {
        offset: u64,
    },
    Poll {
        offsets: HashMap<String, u64>,
    },
    PollOk {
        msgs: HashMap<String, Vec<Vec<u64>>>,
    },
    CommitOffsets {
        offsets: HashMap<String, u64>,
    },
    CommitOffsetsOk,
    ListCommittedOffsets {
        keys: Vec<String>,
    },
    ListCommittedOffsetsOk {
        offsets: HashMap<String, u64>,
    },
}
