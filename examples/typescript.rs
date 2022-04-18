use node_workers::{EmptyPayload, WorkerPool};

fn main() {
  // Create a pool of 4 node workers
  let mut pool = WorkerPool::setup("examples/worker.ts", 4);
  pool.set_binary("node -r esbuild-register");
  pool.with_debug(true);

  pool
    .perform::<(), _>("ping", EmptyPayload::bulk(1))
    .unwrap();
}
