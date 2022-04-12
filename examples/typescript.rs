use node_workers::{EmptyPayload, WorkerPool};

fn main() {
  // Create a pool of 4 node workers
  let mut pool = WorkerPool::setup(4);
  pool.set_binary("node -r esbuild-register");
  pool.with_debug(true);

  pool
    .perform::<(), _>("examples/worker.ts", "ping", EmptyPayload::bulk(1))
    .unwrap();
}
