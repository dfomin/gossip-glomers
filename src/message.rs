use serde::{Deserialize, Serialize};

use crate::body::Body;

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: Body,
}

impl Message {
    pub fn reply(self) -> Self {
        Message {
            src: self.dest,
            dest: self.src,
            body: self.body.reply(),
        }
    }
}
