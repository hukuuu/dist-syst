use std::{io::{self, Write}};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    src: String,
    dest: String,
    body: Body
}

#[derive(Serialize, Deserialize, Debug)]
struct Body {
    r#type: String,
    msg_id: u32,
    in_reply_to: Option<u32>,
    echo: Option<String>
}


fn main() -> io::Result<()> {
    let mut id = 0;

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();

    for line in stdin.lines() {
        let line = line.unwrap();
        let mut msg: Message = serde_json::from_str(&line)?;

        let tmp = msg.src;
        msg.src = msg.dest;
        msg.dest = tmp;
        msg.body.in_reply_to = Some(msg.body.msg_id);
        msg.body.msg_id = id; 
        id += 1;

        if msg.body.r#type == "init" {
            msg.body.r#type = "init_ok".to_string();
        }

        if msg.body.r#type == "echo" {
            msg.body.r#type = "echo_ok".to_string();
    
        }

        let mut resp = serde_json::to_string(&msg)?;
        resp.push_str("\r\n");
        stdout.write_all(resp.as_bytes())?;

    }

    Ok(())
}
