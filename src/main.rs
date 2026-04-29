mod body;
mod message;

use std::io::{BufRead, stdin};

use anyhow::Result;

use crate::message::Message;

#[tokio::main]
async fn main() -> Result<()> {
    let reader = stdin().lock();
    for line in reader.lines() {
        let message: Message = serde_json::from_str(&line?)?;
        let reply = message.reply();
        println!("{}", serde_json::to_string(&reply)?);
    }
    Ok(())
}
