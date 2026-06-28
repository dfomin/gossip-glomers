use serde::{Deserialize, Serialize};

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum ReadRPC {
//     Stage3 { messages: Vec<u32> },
//     Stage4 { value: u32 },
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(tag = "type", rename_all = "snake_case")]
// pub enum Body {
//     Broadcast {
//         msg_id: u32,
//         message: u32,
//     },
//     BroadcastOk {
//         msg_id: u32,
//         in_reply_to: u32,
//     },
//     Read {
//         msg_id: u32,
//         key: Option<String>,
//     },
//     ReadOk {
//         msg_id: Option<u32>,
//         in_reply_to: u32,
//         #[serde(flatten)]
//         result: ReadRPC,
//     },
//     Topology {
//         msg_id: u32,
//         topology: HashMap<String, Vec<String>>,
//     },
//     TopologyOk {
//         msg_id: u32,
//         in_reply_to: u32,
//     },
//     Add {
//         msg_id: u32,
//         delta: u32,
//     },
//     AddOk {
// j        msg_id: u32,
//         in_reply_to: u32,
//     },
//     Cas {
//         msg_id: u32,
//         key: String,
//         from: u32,
//         to: u32,
//         create_if_not_exists: bool,
//     },
//     CasOk {
//         msg_id: Option<u32>,
//         in_reply_to: u32,
//     },
//     Error {
//         msg_id: Option<u32>,
//         in_reply_to: u32,
//         code: u32,
//         text: String,
//     },
//     Noop,
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
}
