# Rust Node Workers
[![CI](https://github.com/CyriacBr/rust-node-workers/actions/workflows/CI.yml/badge.svg)](https://github.com/CyriacBr/rust-node-workers/actions/workflows/CI.yml)
[![crates.io](https://img.shields.io/crates/v/node-workers.svg)](https://crates.io/crates/node-workers)
[![documentation](https://img.shields.io/badge/docs-live-brightgreen)](https://docs.rs/node_workers)

This lets you call node binaries from Rust code using a pool of workers. This is useful for project that are mostly coded in Rust
but need to leverage Node packages for some tasks, like using Typescript's API or performing SSR with a JS framework.  
Using a pool of workers is fundamental to avoid the cost of booting node binaries. Medium to big Node binaries can take about a second to bot (or more) depending on the packages it use, and using a pool of long-lived processes save you time on subsequent runs. If throughout the usage of your program you need to interact with a node binary multiple times, a reusable and long-lived process will save you a LOT of time.  

This solution differs from calling rust within Node like you'd do with `napi-rs`. If most of your code is written in Rust, the additional overhead of creating and maintaining node addons won't be worth it.

The pool spawn a long-lived node process and communicate with it using stdin/stdout, which makes this solution cross-platform.  
To communicate with it, `node-workers` provides a bridge that need to be used when making the node binary to interact with:

```ts
// worker.js
const { bridge } = require('node-workers');

bridge({
  ping: (payload) => {
    console.log(`pong at ${new Date()}`);
    return payload * 2;
  }
});
```

Then this worker can be tasked from your Rust code:
```rust
use node_workers::{WorkerPool};

let mut pool = WorkerPool::setup(4); // 4 max workers
let result = pool.run_task::<u32, _>("worker", "ping", vec![100]).unwrap();
println!("result: {:?}", result);
```

## Installation
The npm package need to be installed to setup the bridge:
```sh
yarn add node-workers
```
In your rust project:
```yml
[dependencies]
node-workers = "0.5.1"
```

## Usage

Checkout the [documentation](https://docs.rs/node_workers) as well as the [examples in the repo](https://github.com/CyriacBr/rust-node-workers/tree/main/examples).

## Development

### Publishing

- `release-plz update`
- Adjust package.json version
- `git commit -m "chore: release" && git tag vx.y.z && git push --follow-tags`
- `npm publish --access=public`