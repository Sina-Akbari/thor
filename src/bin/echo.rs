use thor::*;

use std::io::StdoutLock;

use anyhow::{Context, Ok};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Echo { echo: String },
    EchoOk { echo: String },
}

struct EchoNode {
    id: usize,
}

impl Node<(), Payload> for EchoNode {
    fn from_init(
        _state: (),
        _init: thor::Init,
        _tx: std::sync::mpsc::Sender<Event<Payload>>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(EchoNode { id: 1 })
    }

    fn step(&mut self, input: Event<Payload>, stdout: &mut StdoutLock) -> anyhow::Result<()> {
        match input {
            Event::Message(input) => {
                let mut reply = input.into_reply(Some(&mut self.id));
                match reply.body.payload {
                    Payload::Echo { echo } => {
                        reply.body.payload = Payload::EchoOk { echo };

                        reply
                            .send_reply(&mut *stdout)
                            .context("reply with echo_ok")?;
                    }
                    Payload::EchoOk { .. } => {}
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
    start_app::<_, EchoNode, _, _>(())
}
