use core::panic;
use std::{
    collections::{HashMap, HashSet},
    sync::mpsc::Sender,
};

use anyhow::{Context, Result};
use fly_dist_sys::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum BroadcastPayload {
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
    Broadcast {
        message: u32,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Vec<u32>,
    },
}

pub struct BroadcastNode {
    tx: Sender<Message<BroadcastPayload>>,
    id: IdGenerator,
    node_id: String,
    neighbors: Vec<String>,
    values: HashSet<u32>,
}

impl Node<BroadcastPayload> for BroadcastNode {
    fn new(tx: Sender<Message<BroadcastPayload>>, init: Init) -> Self {
        Self {
            tx,
            id: IdGenerator::new(),
            node_id: init.node_id,
            neighbors: Vec::new(),
            values: HashSet::new(),
        }
    }

    fn handle_msg(self: &mut Self, msg: Message<BroadcastPayload>) {
        eprintln!("recv: {:?}", &msg);
        match msg.clone().body.payload {
            BroadcastPayload::Topology { mut topology } => {
                let Some(neighbors) = topology.get_mut(&self.node_id) else { panic!("no neighbors"); };
                self.neighbors = neighbors.to_vec();

                let mut reply = msg.into_reply(Some(self.id.next_id()));
                reply.body.payload = BroadcastPayload::TopologyOk;
                self.tx.send(reply).context("Sending topology ok").unwrap();
            }
            BroadcastPayload::Broadcast { message } => {
                if !self.values.contains(&message) {
                    self.values.insert(message);

                    for neighbor in &self.neighbors {
                        if neighbor.eq(&msg.src) {
                            continue;
                        }
                        let mut msg = msg.clone();
                        msg.src = self.node_id.clone();
                        msg.dest = neighbor.clone();
                        self.tx
                            .send(msg)
                            .context(format!("Broadcasting to neighbor {}", neighbor))
                            .unwrap();
                    }
                }

                //only reply to clients, nodes dont need ack
                if msg.src.starts_with("c") {
                    let mut reply = msg.into_reply(Some(self.id.next_id()));
                    reply.body.payload = BroadcastPayload::BroadcastOk;
                    self.tx
                        .send(reply)
                        .context("Message already seen, BroadcastOk")
                        .unwrap();
                }

                ()
            }
            BroadcastPayload::Read => {
                let mut reply = msg.into_reply(Some(self.id.next_id()));
                reply.body.payload = BroadcastPayload::ReadOk {
                    messages: self.values.iter().cloned().collect(),
                };
                self.tx
                    .send(reply)
                    .context("Sending all accumulated values")
                    .unwrap();
            }

            BroadcastPayload::BroadcastOk => {}
            BroadcastPayload::TopologyOk => unreachable!(),
            BroadcastPayload::ReadOk { messages: _ } => unreachable!(),
        }
    }
}

fn main() -> Result<()> {
    run::<BroadcastNode, BroadcastPayload>()
}

#[cfg(test)]
mod tests {
    use std::{
        sync::mpsc::{channel, Receiver},
        vec,
    };

    use super::*;

    fn read(rx: &Receiver<Message<BroadcastPayload>>) {
        if let Ok(msg) = rx.recv() {
            println!("resp: {}", serde_json::to_string(&msg).unwrap());
        }
    }

    #[test]
    fn it_works() {
        let init = Init {
            node_id: String::from("n1"),
            node_ids: vec!["n2", "n3", "n4"]
                .into_iter()
                .map(String::from)
                .collect(),
        };

        let (tx, rx) = channel();
        let mut node = BroadcastNode::new(tx, init);
        let mut topology: HashMap<String, Vec<String>> = HashMap::new();
        topology.insert("n1".to_string(), vec!["n2".to_string(), "n3".to_string()]);
        topology.insert("n2".to_string(), vec!["n3".to_string()]);
        topology.insert("n3".to_string(), vec!["n1".to_string()]);
        let topology = Message {
            src: "c1".to_string(),
            dest: "n1".to_string(),
            body: Body {
                in_reply_to: None,
                msg_id: Some(0),
                payload: BroadcastPayload::Topology { topology },
            },
        };
        println!("before {:?}", node.neighbors);
        node.handle_msg(topology);
        println!("after {:?}", node.neighbors);

        read(&rx);

        let broadcast = Message {
            src: "c1".to_string(),
            dest: "n1".to_string(),
            body: Body {
                in_reply_to: None,
                msg_id: Some(0),
                payload: BroadcastPayload::Broadcast {
                    message: serde_json::from_str("{\"foo\": 3}").unwrap(),
                },
            },
        };

        //first message, expect to be broadcast
        node.handle_msg(broadcast.clone());
        println!("values: {:?}", node.values);
        read(&rx);
        read(&rx);
        read(&rx);

        //same message, expect to be ignored
        node.handle_msg(broadcast);
        println!("values: {:?}", node.values);
        read(&rx);

        //try read values
        let msg = Message {
            src: "c1".to_string(),
            dest: "n1".to_string(),
            body: Body {
                in_reply_to: None,
                msg_id: Some(0),
                payload: BroadcastPayload::Read,
            },
        };

        //first message, expect to be broadcast
        node.handle_msg(msg);
        read(&rx);
    }
}
