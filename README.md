# hyper-boilerplate
Boilerplate code for rust hyper web server. Implements http/https/ws/wss single and multithreaded server with toml config.

Going to add some shared state and websocket benchmarking code.

## How to run

Install nightly rust

$ curl -s https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly

Clone and run

$ git clone https://github.com/ddosoff/hyper-boilerplate.git

$ cd cd hyper-boilerplate

$ cargo run --release
