# Gossip Glomers - Distributed Systems Challenge

This repository contains my solutions to the [Fly.io Distributed Systems Challenge](https://fly.io/dist-sys/), implemented in Rust.

## About

The challenge is built on [Maelstrom](https://github.com/jepsen-io/maelstrom), a testing platform that simulates distributed systems by routing messages between nodes. Maelstrom injects failures and verifies consistency guarantees, helping you build robust distributed systems.

## Running Tests

Build the Rust binary and run Maelstrom test cases against it:

```bash
# Build the binary
cargo build --release

# Run a test (example: echo challenge)
~/maelstrom/maelstrom test -w echo --bin ./target/release/thor --node-count 1 --time-limit 10
```

Check the `store/` directory for test results and detailed logs after each run.

## Acknowledgements

Inspired by [Jon Gjengset’s Maelstrom walkthrough](https://www.youtube.com/watch?v=gboGyccRVXI).
