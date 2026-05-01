use serde::{Deserialize, Serialize};

use crate::body::Body;

#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: Body,
}

impl Message {
    pub fn reply(&self, reply_body: Body) -> Self {
        Message {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body: reply_body,
        }
    }
}
