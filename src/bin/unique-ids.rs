use thor::*;

use std::io::{StdoutLock, Write};

use anyhow::{Context, Ok, bail};
use serde::{Deserialize, Serialize};
use ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Generate,
    GenerateOk {
        #[serde(rename = "id")]
        guid: String,
    },
}

struct UniqueNode {
    id: usize,
}

impl Node<Payload> for UniqueNode {
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
                    .context("serialize response to generate")?;
                stdout.write_all(b"\n").context("write trailing new line")?;
                self.id += 1;
            }
            Payload::Generate => {
                let guid = ulid::Ulid::new().to_string();

                let reply = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: Payload::GenerateOk { guid },
                    },
                };
                serde_json::to_writer(&mut *stdout, &reply)
                    .context("serialize response to init")?;

                stdout.write_all(b"\n").context("write trailing new line")?;
                self.id += 1;
            }
            Payload::InitOk => bail!("received init_ok message"),
            Payload::GenerateOk { .. } => bail!("received generate_ok message"),
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let state = UniqueNode { id: 0 };

    start_app(state)
}
