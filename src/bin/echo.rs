use thor::*;

use std::io::{StdoutLock, Write};

use anyhow::{Context, Ok, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
}

struct EchoNode {
    id: usize,
}

impl Node<Payload> for EchoNode {
    fn step(&mut self, input: Message<Payload>, stdout: &mut StdoutLock) -> anyhow::Result<()> {
        match input.body.payload {
            Payload::Init { .. } => {
                let reply = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: Payload::InitOk,
                    },
                };
                serde_json::to_writer(&mut *stdout, &reply)
                    .context("serialize response to init")?;

                stdout.write_all(b"\n").context("write trailing new line")?;
                self.id += 1;
            }
            Payload::InitOk => bail!("received init_ok message"),
            Payload::Echo { echo } => {
                let reply = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: Payload::EchoOk { echo },
                    },
                };

                serde_json::to_writer(&mut *stdout, &reply)
                    .context("serialize response to echo")?;

                stdout.write_all(b"\n").context("write trailing new line")?;
                self.id += 1;
            }
            Payload::EchoOk { echo } => {}
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let mut state = EchoNode { id: 0 };

    start_app(state)
}
