#![deny(clippy::all)]

//! # Node Workers
//! This crate lets you call node binaries from Rust code using a pool of workers. This is useful for project that are mostly coded in Rust
//! but need to leverage Node packages for some tasks, like using Typescript's API or performing SSR with a JS framework.  
//! Using a pool of workers is fundamental to avoid the cost of booting node binaries on multiple calls.
//! Medium to big Node binaries can take about a second to bot (or more) depending on its imports, and using a pool of long-lived processes
//! save you time on subsequent runs. If throughout the usage of your program you need to interact with a node binary multiple times,
//! a reusable and long-lived process will save you a LOT of time.  
//!
//! ## Usage example
//!
//! ```rust
//! use node_workers::WorkerPool;
//! use serde::Deserialize;
//! use std::path::Path;
//! # use std::error::Error;
//!
//! #[derive(Deserialize, Debug)]
//! struct Property {
//!   pub key: String,
//!   #[serde(alias = "type")]
//!   pub propType: String,
//! }
//! #[derive(Deserialize, Debug)]
//! struct Interface {
//!   pub name: String,
//!   pub props: Vec<Property>,
//! }
//!
//! # fn main() -> Result<(), Box<dyn Error>> {
//! // Create a pool of 4 node workers
//! let mut pool = WorkerPool::setup(4);
//! pool.with_debug(true);
//!
//! // Payloads
//! let files = vec![
//!   Path::new("examples/user-files/user.ts").canonicalize()?,
//!   Path::new("examples/user-files/pet.ts").canonicalize()?,
//! ];
//!
//! // execute the command "getInterfaces" on every file
//! // each executed worker will return an array of interfaces (Vec<Interface>)
//! let interfaces = pool.perform::<Vec<Interface>, _>("examples/worker", "getInterfaces", files)?;
//! let interfaces: Vec<Interface> = interfaces
//!   .into_iter()
//!   .map(|x| x.unwrap())
//!   .flatten()
//!   .collect();
//! println!("interfaces: {:#?}", interfaces);
//! # Ok(())
//! # }
//! ```

mod as_payload;
mod utils;
mod worker;
mod worker_pool;
mod worker_pool_inner;
mod worker_thread;

pub use as_payload::*;
pub use worker_pool::*;
