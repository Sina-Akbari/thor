use thor::*;

use std::{collections::HashMap, io::StdoutLock};

use anyhow::{Context, Ok};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Vec<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

struct BroadcastNode {
    id: usize,
    messages: Vec<usize>,
}

impl Node<(), Payload> for BroadcastNode {
    fn from_init(_state: (), _init: thor::Init) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            id: 1,
            messages: Vec::new(),
        })
    }
    fn step(&mut self, input: Message<Payload>, stdout: &mut StdoutLock) -> anyhow::Result<()> {
        let mut reply = input.into_reply(Some(&mut self.id));

        match reply.body.payload {
            Payload::Broadcast { message } => {
                self.messages.push(message);
                reply.body.payload = Payload::BroadcastOk;

                reply
                    .send_reply(&mut *stdout)
                    .context("reply with broadcast_ok")?;
            }
            Payload::Read {} => {
                reply.body.payload = Payload::ReadOk {
                    messages: self.messages.clone(),
                };

                reply
                    .send_reply(&mut *stdout)
                    .context("reply with read_ok")?;
            }
            Payload::Topology { .. } => {
                reply.body.payload = Payload::TopologyOk;

                reply
                    .send_reply(&mut *stdout)
                    .context("reply with topology_ok")?;
            }
            Payload::BroadcastOk {} | Payload::ReadOk { .. } | Payload::TopologyOk => {}
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    start_app::<_, BroadcastNode, _>(())
}
