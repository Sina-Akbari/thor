use thor::*;

use std::{
    collections::{HashMap, HashSet},
    io::StdoutLock,
    time::Duration,
};

use anyhow::{Context, Ok};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
enum InjectedPayload {
    Gossip,
}

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
        messages: HashSet<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
    Gossip {
        seen: HashSet<usize>,
    },
}

struct BroadcastNode {
    id: usize,
    node: String,
    messages: HashSet<usize>,

    known: HashMap<String, HashSet<usize>>,
    neighbours: Vec<String>,
}

impl Node<(), Payload, InjectedPayload> for BroadcastNode {
    fn from_init(
        _state: (),
        init: thor::Init,
        tx: std::sync::mpsc::Sender<Event<Payload, InjectedPayload>>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        std::thread::spawn(move || {
            // TODO: Handle EOF
            loop {
                std::thread::sleep(Duration::from_millis(300));
                if let Err(_) = tx.send(Event::Inject(InjectedPayload::Gossip)) {
                    break;
                }
            }
        });

        Ok(Self {
            id: 1,
            node: init.node_id,
            messages: HashSet::new(),
            known: init
                .node_ids
                .into_iter()
                .map(|x| (x, HashSet::new()))
                .collect(),
            neighbours: Vec::new(),
        })
    }
    fn step(
        &mut self,
        input: Event<Payload, InjectedPayload>,
        stdout: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        match input {
            Event::EOF => {}
            Event::Inject(payload) => match payload {
                InjectedPayload::Gossip => {
                    for n in &self.neighbours {
                        let known_to_n = &self.known[n];

                        let message = Message {
                            src: self.node.clone(),
                            dst: n.clone(),
                            body: Body {
                                id: None,
                                in_reply_to: None,
                                payload: Payload::Gossip {
                                    seen: self
                                        .messages
                                        .iter()
                                        .copied()
                                        .filter(|x| !known_to_n.contains(x))
                                        .collect(),
                                },
                            },
                        };

                        message
                            .send_reply(&mut *stdout)
                            .with_context(|| format!("gossip to {}", n))?;
                    }
                }
            },
            Event::Message(input) => {
                let mut reply = input.into_reply(Some(&mut self.id));

                match reply.body.payload {
                    Payload::Gossip { seen } => {
                        self.messages.extend(seen);
                    }
                    Payload::Broadcast { message } => {
                        self.messages.insert(message);
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
                    Payload::Topology { mut topology } => {
                        reply.body.payload = Payload::TopologyOk;

                        self.neighbours = topology
                            .remove(&self.node)
                            .expect(&format!("no topology given for node {}", self.node));

                        reply
                            .send_reply(&mut *stdout)
                            .context("reply with topology_ok")?;
                    }
                    Payload::BroadcastOk {} | Payload::ReadOk { .. } | Payload::TopologyOk => {}
                }
            }
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    start_app::<_, BroadcastNode, _, _>(())
}
