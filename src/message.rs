use serde::{Deserialize, Serialize};

use crate::body::Body;

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: Body,
}
