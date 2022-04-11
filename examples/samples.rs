use node_workers::{make_payloads, AsPayload, EmptyPayload, WorkerPool};
use serde_json::json;

fn main() {
  {
    // Create a pool of 2 node workers
    let mut pool = WorkerPool::setup(2);
    pool.with_debug(true);
    // Create 4 payloads
    let payloads = vec![10, 20, 30, 40];
    // execute the command "fib" for every payload
    // worker 1 and 2 will be reused as the pool can only create a maximum of 2 workers
    pool
      .perform::<(), _>("examples/worker", "fib", payloads)
      .unwrap();
  }

  {
    // Example using `run_worker` instead of `perform`
    let mut pool = WorkerPool::setup(2);
    let thread = pool.run_worker("examples/worker", "fib2", 40u32);
    let result = thread.get_result::<u32>().unwrap();
    println!("run_worker result: {:?}", result);
  }

  {
    let mut pool = WorkerPool::setup(1);
    // Create 2 empty payloads - the task doesn't need any payload, but we still want to execute it twice
    let payloads = vec![EmptyPayload::new(), EmptyPayload::new()];
    pool
      .perform::<(), _>("examples/worker", "ping", payloads)
      .unwrap();

    // or, use EmptyPayload::bulk
    pool
      .perform::<(), _>("examples/worker", "ping", EmptyPayload::bulk(2))
      .unwrap();
  }

  {
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
