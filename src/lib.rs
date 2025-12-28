use std::io::{BufRead, StdoutLock, Write};

use anyhow::{Context, Ok};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<Payload> {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<Payload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body<Payload> {
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    pub in_reply_to: Option<usize>,

    #[serde(flatten)]
    pub payload: Payload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum InitPayload {
    Init(Init),
    InitOk,
}

pub trait Node<S, Payload> {
    fn from_init(state: S, init: Init) -> anyhow::Result<Self>
    where
        Self: Sized;

    fn step(&mut self, input: Message<Payload>, stdout: &mut StdoutLock) -> anyhow::Result<()>;
}

pub fn start_app<S, N, P>(state: S) -> anyhow::Result<()>
where
    P: DeserializeOwned + Serialize,
    N: Node<S, P>,
{
    let stdin = std::io::stdin().lock();
    let mut lines = stdin.lines();

    let init_msg: Message<InitPayload> = serde_json::from_str(
        &lines
            .next()
            .expect("no init message received!")
            .context("failed to read init message from stdin")?,
    )
    .expect("init could not be deserialized!");

    let InitPayload::Init(init) = init_msg.body.payload else {
        panic!("first message should be init");
    };
    let mut node: N = Node::from_init(state, init).context("could not initialize node")?;

    let mut stdout = std::io::stdout().lock();

    let reply = Message {
        src: init_msg.dst,
        dst: init_msg.src,
        body: Body {
            id: Some(0),
            in_reply_to: init_msg.body.id,
            payload: InitPayload::InitOk,
        },
    };

    serde_json::to_writer(&mut stdout, &reply).context("serialize response to generate")?;
    stdout.write_all(b"\n").context("write trailing new line")?;

    for line in lines {
        let input = serde_json::from_str(
            &line.expect("Maelstrom input from STDIN could not be deserialized."),
        )
        .context("Maelstrom input from STDIN could not be deserialized.")?;

        node.step(input, &mut stdout)
            .context("Node step function failed")?;
    }

    Ok(())
}
