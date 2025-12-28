use thor::*;

use std::{
    collections::HashMap,
    io::{StdoutLock, Write},
};

use anyhow::{Context, Ok, bail};
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
                serde_json::to_writer(&mut *stdout, &reply)
                    .context("serialize response to broadcast_ok")?;

                stdout.write_all(b"\n").context("write trailing new line")?;
            }
            Payload::Read {} => {
                reply.body.payload = Payload::ReadOk {
                    messages: self.messages.clone(),
                };

                serde_json::to_writer(&mut *stdout, &reply)
                    .context("serialize response to read_ok")?;

                stdout.write_all(b"\n").context("write trailing new line")?;
            }
            Payload::Topology { .. } => {
                reply.body.payload = Payload::TopologyOk;

                serde_json::to_writer(&mut *stdout, &reply)
                    .context("serialize response to topology_ok")?;

                stdout.write_all(b"\n").context("write trailing new line")?;
            }
            Payload::BroadcastOk {} | Payload::ReadOk { .. } | Payload::TopologyOk => {}
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    start_app::<_, BroadcastNode, _>(())
}
