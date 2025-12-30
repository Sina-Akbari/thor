use thor::*;

use std::io::StdoutLock;

use anyhow::{Context, Ok};
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
    fn from_init(
        _state: (),
        init: thor::Init,
        _tx: std::sync::mpsc::Sender<Event<Payload>>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            node: init.node_id,
            id: 1,
        })
    }
    fn step(&mut self, input: Event<Payload>, stdout: &mut StdoutLock) -> anyhow::Result<()> {
        match input {
            Event::Message(input) => {
                let mut reply = input.into_reply(Some(&mut self.id));

                match reply.body.payload {
                    Payload::Generate => {
                        let guid = format!("{}-{}", self.node, ulid::Ulid::new().to_string());
                        reply.body.payload = Payload::GenerateOk { guid };

                        reply
                            .send_reply(&mut *stdout)
                            .context("reply with generate_ok")?;
                    }
                    Payload::GenerateOk { .. } => {}
                }

                Ok(())
            }
            Event::EOF | Event::Inject(..) => {
                panic!("unsupported event received");
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    start_app::<_, UniqueNode, _, _>(())
}
