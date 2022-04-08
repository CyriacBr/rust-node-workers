use rust_node_workers::{worker_pool::WorkerPool, as_payload::EmptyPayload};

fn main() {
  {
    // Create a pool of 2 node workers
    let mut pool = WorkerPool::setup(2);
    // Create 4 payloads
    let payloads = vec![10, 20, 30, 40];
    let result = pool.run_task::<u64, _>("examples/worker", "fib2", payloads);
    println!("-----");
    println!("result: {:?}", result);
  }
  {
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, Debug)]
    struct Person {
      name: String,
      age: u8,
      phones: Vec<String>,
    }
    // Create a pool of 2 node workers
    let mut pool = WorkerPool::setup(2);
    // Create 4 payloads
    let payloads = vec![EmptyPayload::new()];
    let result = pool.run_task::<Person, _>("examples/worker", "getUser", payloads);
    println!("-----");
    println!("result: {:?}", result);
  }
}
