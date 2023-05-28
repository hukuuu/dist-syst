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
    n: usize,
    node_id: String,
}

impl Node<GeneratePayload> for UniqueIdNode {
    fn new(tx: Sender<Message<GeneratePayload>>, init: Init) -> Self {
        Self {
            tx,
            n: 0,
            node_id: init.node_id,
        }
    }

    fn handle_msg(self: &mut Self, msg: Message<GeneratePayload>) {
        let mut reply = msg.into_reply(Some(0));
        if let GeneratePayload::Generate = reply.body.payload {
            reply.body.payload = GeneratePayload::GenerateOk {
                id: format!("{}-{}", self.node_id, self.n),
            };
            self.n += 1;
        };
        self.tx.send(reply).unwrap();
    }
}

fn main() -> Result<()> {
    run::<UniqueIdNode, GeneratePayload>()
}
