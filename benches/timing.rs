use std::process::Command;

use rust_node_workers::worker_pool::WorkerPool;

fn main() {
  fn standard_command() {
    Command::new("node")
      .arg("benches/workers/fast-inner")
      .arg("30")
      .spawn()
      .unwrap()
      .wait()
      .unwrap();
  }

  let mut pool = WorkerPool::setup(1);
  let mut worker_pool = || {
    pool.run_task::<(), _>("benches/workers/fast", "fib", vec![30]);
  };

  {
    println!("==== standard command");
    let timer = std::time::Instant::now();
    standard_command();
    println!("first run: {:#?}ms", timer.elapsed().as_millis());
    let timer = std::time::Instant::now();
    for _ in 0..3 {
      standard_command();
    }
    println!("subsequent 3 runs: {:#?}ms", timer.elapsed().as_millis());
  }

  {
    println!("==== worker pool");
    let timer = std::time::Instant::now();
    worker_pool();
    println!("first run: {:#?}ms", timer.elapsed().as_millis());
    let timer = std::time::Instant::now();
    for _ in 0..3 {
        worker_pool();
    }
    println!("subsequent 3 runs: {:#?}ms", timer.elapsed().as_millis());
  }
}
