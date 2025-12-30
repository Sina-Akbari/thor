use std::io::{BufRead, StdoutLock, Write};

use anyhow::Context;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<Payload> {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<Payload>,
}

impl<Payload> Message<Payload> {
    pub fn into_reply(self, id: Option<&mut usize>) -> Self {
        Self {
            src: self.dst,
            dst: self.src,
            body: Body {
                id: id.map(|id| {
                    let mid = *id;
                    *id += 1;
                    mid
                }),
                in_reply_to: self.body.id,
                payload: self.body.payload,
            },
        }
    }

    pub fn send_reply(self, output: &mut impl Write) -> anyhow::Result<()>
    where
        Self: Serialize,
    {
        serde_json::to_writer(&mut *output, &self).context("serialize response")?;

        output.write_all(b"\n").context("write trailing new line")?;

        anyhow::Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Event<Payload, InjectedPayload = ()> {
    Message(Message<Payload>),
    Inject(InjectedPayload),
    EOF,
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

pub trait Node<S, Payload, InjectedPayload = ()> {
    fn from_init(
        state: S,
        init: Init,
        inject: std::sync::mpsc::Sender<Event<Payload, InjectedPayload>>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized;

    fn step(
        &mut self,
        input: Event<Payload, InjectedPayload>,
        stdout: &mut StdoutLock,
    ) -> anyhow::Result<()>;
}

pub fn start_app<S, N, P, IP>(state: S) -> anyhow::Result<()>
where
    P: DeserializeOwned + Send + 'static,
    N: Node<S, P, IP>,
    IP: Send + 'static,
{
    let (tx, rx) = std::sync::mpsc::channel();
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
    let mut node: N =
        Node::from_init(state, init, tx.clone()).context("could not initialize node")?;

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

    reply
        .send_reply(&mut stdout)
        .context("failed to send init_ok reply")?;

    drop(lines);

    let th = std::thread::spawn(move || {
        let stdin = std::io::stdin().lock();
        let lines = stdin.lines();
        for line in lines {
            let line = line.context("Maelstrom input from STDIN could not be deserialized.")?;
            let input: Message<P> = serde_json::from_str(&line)
                .context("Maelstrom input from STDIN could not be deserialized.")?;

            if let Err(_) = tx.send(Event::Message(input)) {
                return Ok::<_, anyhow::Error>(());
            }
        }
        let _ = tx.send(Event::EOF);
        Ok(())
    });

    for input in rx {
        node.step(input, &mut stdout)
            .context("Node step function failed")?;
    }

    th.join()
        .expect("stdin thread panicked!")
        .context("stdin thread encountered an error")?;

    Ok(())
}
