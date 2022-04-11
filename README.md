# Rust Node Workers
[![CI](https://github.com/CyriacBr/rust-node-workers/actions/workflows/CI.yml/badge.svg)](https://github.com/CyriacBr/rust-node-workers/actions/workflows/CI.yml)
[![crates.io](https://img.shields.io/crates/v/node-workers.svg)](https://crates.io/crates/node-workers)
[![documentation](https://img.shields.io/badge/docs-live-brightgreen)](https://docs.rs/node_workers)

This lets you call node binaries from Rust code using a pool of workers. This is useful for project that are mostly coded in Rust
but need to leverage Node packages for some tasks, like using Typescript's API or performing SSR with a JS framework.  
Using a pool of workers is fundamental to avoid the cost of booting node binaries. Medium to big Node binaries can take about a second to bot (or more) depending on its imports, and using a pool of long-lived processes save you time on subsequent runs. If throughout the usage of your program you need to interact with a node binary multiple times, a reusable and long-lived process will save you a LOT of time.  

This solution differs from calling rust within Node like you'd do with solutions like `napi-rs`. If most of your code is written in Rust, the additional overhead of creating and maintaining node addons won't be worth it.

The pool spawns a long-lived node process and communicate with it using stdin/stdout, which makes this solution cross-platform.  
To communicate with it, `node-workers` provides a bridge that needs to be used when creating the node binary to interact with:

```ts
// worker.js
const { bridge } = require('rust-node-workers');

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
let result = pool.perform::<u32, _>("worker", "ping", vec![100]).unwrap();
println!("result: {:?}", result);
```

## Installation
The npm package need to be installed to setup the bridge:
```sh
yarn add rust-node-workers
```
In your rust project:
```yml
[dependencies]
node-workers = "0.5.1"
```

## Usage

This crate exposes a `WorkerPool` you can instantiate with the length of the pool. When a task needs to be performed, a new worker will be created if needed, up to the maximum amount.
```rust
let pool = WorkerPool::setup(4); // 4 max workers
```
Then, you can call tasks from your worker using `run_worker` or `perform`.

`run_worker` performs a task on a worker in a new thread. Using `get_result` on the thread will wait for the worker to finish and deserialize the result if there is any.
```rust
let mut pool = WorkerPool::setup(2);
pool.run_worker("examples/worker", "fib", 80u32); // on a separate thread

let thread = pool.run_worker("examples/worker", "fib2", 40u32);
// join the thread's handle
let result = thread.get_result::<u32>().unwrap();
println!("run_worker result: {:?}", result);
```

`perform` takes an array of data to process, and run a worker for each of its value.
```rust
let files = /* vector of TypeScript files */;
// execute the command "getInterfaces" on every file
// each executed worker will return an array of interfaces (Vec<Interface>)
let interfaces = pool
  .perform::<Vec<Interface>, _>("examples/worker", "getInterfaces", files)
  .unwrap();

// it may be benefic to send multiple files to each worker instead of just one
let file_chunks = files.chunks(30);
let interfaces = pool
  .perform::<Vec<Interface>, _>("examples/worker", "getInterfacesBulk", file_chunks)
  .unwrap();
```

You can use `EmptyPayload` for tasks that doesn't need any payload.
```rust
pool.run_worker("examples/worker", "ping", EmptyPayload::new());
```

For additional usage, checkout the [documentation](https://docs.rs/node_workers) as well as the [examples in the repo](https://github.com/CyriacBr/rust-node-workers/tree/main/examples).

## Development

Building:
```sh
yarn build
```

Running examples:
```sh
cargo run --example
```

### Publishing

- `release-plz update`
- Adjust package.json version
- `git commit -m "chore: release" && git tag vx.y.z && git push --follow-tags`
- `npm publish --access=public`