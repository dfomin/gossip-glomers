use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Body {
    Init {
        msg_id: u32,
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {
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
}

impl Body {
    pub fn reply(self) -> Body {
        match self {
            Self::Init { msg_id, .. } => Self::InitOk {
                in_reply_to: msg_id,
            },
            Self::Echo { msg_id, echo } => Self::EchoOk {
                msg_id,
                in_reply_to: msg_id,
                echo,
            },
            _ => panic!("Unsupported enum type {:?}", self),
        }
    }
}
