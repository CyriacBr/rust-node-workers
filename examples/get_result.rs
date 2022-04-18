use node_workers::{EmptyPayload, WorkerPool};

fn main() {
  {
    let mut pool = WorkerPool::setup("examples/worker", 2);
    pool.with_debug(true);
    let payloads = vec![10, 20, 30, 40];
    let result = pool.perform::<u64, _>("fib2", payloads).unwrap();
    println!("-----");
    println!("result: {:?}", result);
  }

  {
    // Using serde, results from workers can be deserialized into structs
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, Debug)]
    struct Person {
      name: String,
      age: u8,
      phones: Vec<String>,
    }

    let mut pool = WorkerPool::setup("examples/worker", 2);
    let result = pool
      .perform::<Person, _>("getUser", vec![EmptyPayload::new()])
      .unwrap();
    println!("-----");
    println!("result: {:?}", result);
  }
}
