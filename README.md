# deboa-ws

[![Crates.io downloads](https://img.shields.io/crates/d/deboa-ws)](https://crates.io/crates/deboa-ws) [![crates.io](https://img.shields.io/crates/v/deboa-ws?style=flat-square)](https://crates.io/crates/deboa-ws) [![Build Status](https://github.com/deboa-client/deboa-ws/actions/workflows/rust.yml/badge.svg?event=push)](https://github.com/deboa-client/deboa-ws/actions/workflows/rust.yml) ![Crates.io MSRV](https://img.shields.io/crates/msrv/deboa-ws) [![Documentation](https://docs.rs/deboa-ws/badge.svg)](https://docs.rs/deboa-ws/latesto/deboa-ws) [![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/deboa-client/deboa-ws/blob/main/LICENSE.md)  [![codecov](https://codecov.io/gh/deboa-client/deboa-ws/graph/badge.svg?token=T0HSBAPVSI)](https://codecov.io/gh/deboa-client/deboa-ws)

## Description

**deboa-ws** is a websockets extension crate for deboa http client.

## Install

Either run from command line:

`cargo add deboa deboa-ws deboa-tokio`

Or add to your `Cargo.toml`:

```toml
deboa = { version = "0.1.4" }
deboa-tokio = { version = "0.1.1" }
deboa-ws = { version = "0.1.1", features = "runtime-tokio" }
```

## Crate features

- runtime-smol
- runtime-tokio

## Usage

```rust, ignore
use deboa::{Client, Result, request::{IntoUrl, DeboaRequestBuilder}};
use deboa_ws::request::{WebsocketRequestBuilder};

let mut client = Client::new();
let request = DeboaRequestBuilder::websocket("ws://example.com").unwrap();
let response = request.send_with(&mut client).await.unwrap();
let ws = response.into_websocket().unwrap();
loop {
    if let Ok(Some(message)) = ws.read_message().await {
        println!("message: {}", message);
    }
}
```

## License

Licensed under either of

- Apache License, Version 2.0
  (LICENSE-APACHE or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  (LICENSE-MIT or <https://opensource.org/licenses/MIT>)

at your option.

## Author

Rogerio Pereira Araujo <rogerio.araujo@gmail.com>
