use std::io::Write;

use anyhow::{Context, Ok, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Message<Payload> {
    src: String,
    dest: String,
    body: Body<Payload>,
}

impl<Payload> Message<Payload> {
    fn into_reply(self, msg_id: Option<&mut u32>) -> Self {
        Self {
            src: self.dest,
            dest: self.src,
            body: Body {
                in_reply_to: self.body.msg_id,
                msg_id: msg_id.map(|id| {
                    let mid = *id;
                    *id += 1;
                    mid
                }),
                payload: self.body.payload,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Body<Payload> {
    in_reply_to: Option<u32>,
    msg_id: Option<u32>,
    #[serde(flatten)]
    payload: Payload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum EchoPayload {
    Echo { echo: String },
    EchoOk { echo: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum InitPayload {
    Init(Init),
    InitOk,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Init {
    node_id: String,
    node_ids: Vec<String>,
}

trait Node<Payload> {
    fn new() -> Self;
    fn handle_msg(self: &mut Self, msg: Message<Payload>) -> Message<Payload>;
}

struct EchoNode;
impl Node<EchoPayload> for EchoNode {
    fn new() -> Self {
        Self {}
    }

    fn handle_msg(self: &mut Self, msg: Message<EchoPayload>) -> Message<EchoPayload> {
        let mut reply = msg.into_reply(Some(&mut 0));
        if let EchoPayload::Echo { echo } = reply.body.payload {
            reply.body.payload = EchoPayload::EchoOk { echo };
        };
        reply
    }
}

fn main() -> Result<()> {
    let mut stdin = std::io::stdin().lines();
    let mut stdout = std::io::stdout().lock();

    let init_msg: Message<InitPayload> = serde_json::from_str(&stdin.next().expect("ni raboti")?)?;
    let mut reply = init_msg.into_reply(Some(&mut 0));
    reply.body.payload = InitPayload::InitOk;
    send(&mut stdout, reply)?;

    let mut echo_node = EchoNode::new();

    for msg in stdin {
        let msg = msg.unwrap();
        let msg: Message<EchoPayload> = serde_json::from_str(&msg)?;
        let reply = echo_node.handle_msg(msg);
        send(&mut stdout, reply)?;
    }

    Ok(())
}

fn send<P>(mut out: &mut impl Write, msg: Message<P>) -> Result<()>
where
    P: Serialize,
{
    serde_json::to_writer(&mut out, &msg).context("write message to out")?;
    out.write_all(b"\n").context("write newline to out")?;
    Ok(())
}
