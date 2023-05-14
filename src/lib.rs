use std::{
    io::Write,
    sync::mpsc::{channel, Sender},
    thread,
};

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message<Payload> {
    pub src: String,
    pub dest: String,
    pub body: Body<Payload>,
}

impl<Payload> Message<Payload> {
    pub fn into_reply(self, msg_id: Option<&mut u32>) -> Self {
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
pub struct Body<Payload> {
    pub in_reply_to: Option<u32>,
    pub msg_id: Option<u32>,
    #[serde(flatten)]
    pub payload: Payload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum InitPayload {
    Init(Init),
    InitOk,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

pub trait Node<Payload> {
    fn new(tx: Sender<Message<Payload>>) -> Self;
    fn handle_msg(self: &mut Self, msg: Message<Payload>);
}


pub fn run<N, P>() -> Result<()>
where
    N: Node<P>,
    P: Serialize + DeserializeOwned + Send + 'static,
{
    let mut stdin = std::io::stdin().lines();
    let mut stdout = std::io::stdout().lock();

    let init_msg: Message<InitPayload> = serde_json::from_str(&stdin.next().expect("ni raboti")?)?;
    let mut reply = init_msg.into_reply(Some(&mut 0));
    reply.body.payload = InitPayload::InitOk;
    send(&mut stdout, reply)?;

    let (tx, rx) = channel();
    let mut node = N::new(tx);

    drop(stdout);
    let jh = thread::spawn(move || {
        let mut stdout = std::io::stdout().lock();
        while let Ok(msg) = rx.recv() {
            send(&mut stdout, msg).unwrap();
        }
    });

    for msg in stdin {
        let msg = msg?;
        let msg: Message<P> = serde_json::from_str(&msg)?;
        node.handle_msg(msg);
    }

    jh.join().unwrap();
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
