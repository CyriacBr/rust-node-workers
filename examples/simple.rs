use rust_node_workers::{
  as_payload::{AsPayload, EmptyPayload},
  make_payloads,
  worker_pool::WorkerPool,
};
use serde_json::json;

fn main() {
  {
    // Create a pool of 2 node workers
    let mut pool = WorkerPool::setup(2);
    pool.with_debug(true);
    // Create 4 payloads
    let payloads = vec![10, 20, 30, 40];
    pool
      .run_task::<(), _>("examples/worker", "fib", payloads)
      .unwrap();
  }

  {
    // Create a pool of 1 node workers
    let mut pool = WorkerPool::setup(1);
    // Create 2 empty payloads - the task doesn't need any
    let payloads = vec![EmptyPayload::new(), EmptyPayload::new()];
    pool
      .run_task::<(), _>("examples/worker", "ping", payloads)
      .unwrap();
    // or
    pool
      .run_task::<(), _>("examples/worker", "ping", EmptyPayload::bulk(2))
      .unwrap();
  }

  {
    // Create a pool of 1 node workers
    let mut pool = WorkerPool::setup(1);
    // Create 3 payloads of different types
    let payloads = make_payloads!(EmptyPayload::new(), 20, "test");
    pool
      .run_task::<(), _>("examples/worker", "ping", payloads)
      .unwrap();
  }

  {
    // to send structs as payload, you need to convert them to serde_json::Value
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, Debug)]
    struct Point {
      pub x: u32,
      pub y: u32,
    }

    // Create a pool of 1 node workers
    let mut pool = WorkerPool::setup(1);
    let payloads = vec![Point { x: 5, y: 1 }, Point { x: 2, y: 4 }]
      .into_iter()
      .map(|x| json!(x))
      .collect();
    pool
      .run_task::<(), _>("examples/worker", "ping", payloads)
      .unwrap();
  }
}
