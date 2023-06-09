use std::sync::mpsc::Sender;

use fly_dist_sys::*;
use anyhow::{Result};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum EchoPayload {
    Echo { echo: String },
    EchoOk { echo: String },
}

pub struct EchoNode {
    tx: Sender<Message<EchoPayload>>,
    id: IdGenerator
}

impl Node<EchoPayload> for EchoNode {
    fn new(tx: Sender<Message<EchoPayload>>, init: Init) -> Self {
        Self { tx, id: IdGenerator::new() }
    }

    fn handle_msg(self: &mut Self, msg: Message<EchoPayload>) {
        let mut reply = msg.into_reply(Some(self.id.next_id()));
        if let EchoPayload::Echo { echo } = reply.body.payload {
            reply.body.payload = EchoPayload::EchoOk { echo };
        };
        self.tx.send(reply).unwrap();
    }
}


fn main() -> Result<()> {
    run::<EchoNode, EchoPayload>()
}