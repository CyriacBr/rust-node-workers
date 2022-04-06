#![deny(clippy::all)]

pub mod worker;
pub mod worker_pool;
mod bench_rs_call_node;

#[macro_use]
extern crate napi_derive;

