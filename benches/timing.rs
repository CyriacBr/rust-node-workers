use benchman::*;
use rust_node_workers::WorkerPool;
use std::process::Command;

fn main() {
  fn standard_command(worker_name: &str) {
    Command::new("node")
      .arg(&format!("benches/workers/{}", worker_name))
      .arg("30")
      .spawn()
      .unwrap()
      .wait()
      .unwrap();
  }

  let bm = BenchMan::new("timing");
  for worker_name in vec!["fast", "slow"] {
    let std_name = &format!("{}-inner", worker_name);

    let mut pool = WorkerPool::setup(1);
    let mut worker_pool = |worker_name: &str| {
      pool.run_task::<(), _>(
        &format!("benches/workers/{}", worker_name),
        "fib",
        vec![30u32],
      ).unwrap();
    };

    {
      let _sw = bm.get_stopwatch(&format!("[{}] standard command - first run", worker_name));
      standard_command(std_name);
    }
    {
      let _sw = bm.get_stopwatch(&format!(
        "[{}] standard command - subsequent 3 runs",
        worker_name
      ));
      for _ in 0..3 {
        standard_command(std_name);
      }
    }

    {
      let _sw = bm.get_stopwatch(&format!("[{}] worker pool - first run", worker_name));
      worker_pool(worker_name);
    }
    {
      let _sw = bm.get_stopwatch(&format!(
        "[{}] worker pool - subsequent 3 runs",
        worker_name
      ));
      for _ in 0..3 {
        worker_pool(worker_name);
      }
    }
  }
  println!("{}", bm);
}
