use node_workers::{make_payloads, AsPayload, EmptyPayload, WorkerPool};
use serde_json::json;

fn main() {
  {
    // Create a pool of 2 node workers
    let mut pool = WorkerPool::setup(2);
    pool.with_debug(true);
    // Create 4 payloads
    let payloads = vec![10, 20, 30, 40];
    // execute the command "fib" on the worker for every payload
    pool
      .perform::<(), _>("examples/worker", "fib", payloads)
      .unwrap();
  }

  {
    let mut pool = WorkerPool::setup(2);
    let handle = pool.run_worker("examples/worker", "fib2", 40u32);
    let result = handle
      .join()
      .unwrap()
      .map(|x| serde_json::from_str::<u32>(x.as_str()).unwrap())
      .unwrap();
    println!("run_worker result: {}", result);
  }

  {
    // Create a pool of 1 node workers
    let mut pool = WorkerPool::setup(1);
    // Create 2 empty payloads - the task doesn't need any
    let payloads = vec![EmptyPayload::new(), EmptyPayload::new()];
    pool
      .perform::<(), _>("examples/worker", "ping", payloads)
      .unwrap();
    // or
    pool
      .perform::<(), _>("examples/worker", "ping", EmptyPayload::bulk(2))
      .unwrap();
  }

  {
    // Create a pool of 1 node workers
    let mut pool = WorkerPool::setup(1);
    // Create 3 payloads of different types
    let payloads = make_payloads!(EmptyPayload::new(), 20, "test");
    pool
      .perform::<(), _>("examples/worker", "ping", payloads)
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
      .perform::<(), _>("examples/worker", "ping", payloads)
      .unwrap();
  }
}