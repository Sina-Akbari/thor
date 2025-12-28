use thor::*;

use std::io::{StdoutLock, Write};

use anyhow::{Context, Ok, bail};
use serde::{Deserialize, Serialize};
use ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Generate,
    GenerateOk {
        #[serde(rename = "id")]
        guid: String,
    },
}

struct UniqueNode {
    node: String,
    id: usize,
}

impl Node<(), Payload> for UniqueNode {
    fn from_init(_state: (), init: thor::Init) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(UniqueNode {
            node: init.node_id,
            id: 1,
        })
    }
    fn step(&mut self, input: Message<Payload>, stdout: &mut StdoutLock) -> anyhow::Result<()> {
        match input.body.payload {
            Payload::Generate => {
                let guid = format!("{}-{}", self.node, ulid::Ulid::new().to_string());

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
            Payload::GenerateOk { .. } => bail!("received generate_ok message"),
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    start_app::<_, UniqueNode, _>(())
}
