use std::sync::mpsc::Sender;

use anyhow::Result;
use fly_dist_sys::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum GeneratePayload {
    Generate,
    GenerateOk { id: String },
}

pub struct UniqueIdNode {
    tx: Sender<Message<GeneratePayload>>,
    id: IdGenerator,
    node_id: String,
}

impl Node<GeneratePayload> for UniqueIdNode {
    fn new(tx: Sender<Message<GeneratePayload>>, init: Init) -> Self {
        Self {
            tx,
            id: IdGenerator::new(),
            node_id: init.node_id,
        }
    }

    fn handle_msg(self: &mut Self, msg: Message<GeneratePayload>) {
        let mut reply = msg.into_reply(Some(self.id.next_id()));
        if let GeneratePayload::Generate = reply.body.payload {
            reply.body.payload = GeneratePayload::GenerateOk {
                id: format!("{}-{}", self.node_id, self.id.next_id()),
            };
        };
        self.tx.send(reply).unwrap();
    }
}

fn main() -> Result<()> {
    run::<UniqueIdNode, GeneratePayload>()
}
