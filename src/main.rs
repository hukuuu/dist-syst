use std::io::{self, Write};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Message<Payload> {
    src: String,
    dest: String,
    body: Body<Payload>,
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

fn main() -> io::Result<()> {
    let mut stdin = std::io::stdin().lines();
    let mut stdout = std::io::stdout().lock();

    let init_msg: Message<InitPayload> = serde_json::from_str(&stdin.next().expect("ni raboti")?)?;
    let InitPayload::Init(init) = init_msg.body.payload else {
        panic!("first message should be init");
    };

    let reply = Message {
        src: init_msg.dest,
        dest: init_msg.src,
        body: Body {
            msg_id: Some(0),
            in_reply_to: init_msg.body.msg_id,
            payload: InitPayload::InitOk,
        },
    };
    serde_json::to_writer(&mut stdout, &reply).expect("eee");
    stdout.write_all(b"\n").expect("ddd");

    eprintln!("{:?}", init);

    for msg in stdin {
        let msg = msg.unwrap();
        let msg: Message<EchoPayload> = serde_json::from_str(&msg)?;

        let EchoPayload::Echo { echo } = msg.body.payload else {
            panic!("invalid echo message");
        };

        let reply = Message {
            src: msg.dest,
            dest: msg.src,
            body: Body {
                msg_id: Some(0),
                in_reply_to: init_msg.body.msg_id,
                payload: EchoPayload::EchoOk { echo },
            },
        };

        serde_json::to_writer(&mut stdout, &reply).expect("eee");
        stdout.write_all(b"\n").expect("ddd");
    }

    Ok(())
}
