use crate::{
  as_payload::AsPayload, print_debug, worker_pool_inner::WorkerPoolInner,
  worker_thread::WorkerThread,
};
use anyhow::{bail, Result};
use serde::de::DeserializeOwned;
use std::{
  sync::{Arc, Mutex},
  thread::JoinHandle,
};

/// A pool of nodejs workers.
/// Wraps a inner struct inside `Arc<Mutex<T>>` to be able to invoke it's method within a spawned thread.
/// This is important so that indefinitely blocking methods such as `get_available_workers` can be offloaded.
pub struct WorkerPool {
  inner: Arc<Mutex<WorkerPoolInner>>,
}

impl WorkerPool {
  /// Create a new workers pool with the maximum numbers of workers that can be spawned for the duration of the program
  /// ```
  /// use node_workers::{WorkerPool};
  ///
  /// let nbr_max_workers = 4;
  /// let mut pool = WorkerPool::setup(nbr_max_workers);
  /// ```
  pub fn setup(max_workers: usize) -> Self {
    WorkerPool {
      inner: Arc::new(Mutex::new(WorkerPoolInner::setup(max_workers))),
    }
  }

  /// Configure the binary that's used to run JS workers
  /// This can be usefull configure node or to run JS via another runtime
  /// ```rust
  /// use node_workers::{EmptyPayload, WorkerPool};
  /// # use std::error::Error;
  ///
  /// # fn main() -> Result<(), Box<dyn Error>> {
  /// let mut pool = WorkerPool::setup(4);
  /// pool.set_binary("node -r esbuild-register");
  /// pool.perform::<(), _>("examples/worker.ts", "ping", EmptyPayload::bulk(1))?;
  /// # Ok(())
  /// # }
  /// ```
  pub fn set_binary(&mut self, binary: &str) {
    self.inner.lock().unwrap().set_binary(binary);
  }

  /// Enable or disable logging
  pub fn with_debug(&mut self, debug: bool) {
    self.inner.lock().unwrap().with_debug(debug);
  }

  /// Run a single worker in a thread. This method returns the created thread, not the result of the worker.
  /// Use this if you need more control on the pool.
  /// ```
  /// use node_workers::{WorkerPool};
  ///
  /// let mut pool = WorkerPool::setup(2);
  /// for n in 1..=4 {
  ///   pool.run_worker("examples/worker", "fib", n * 10);
  /// }
  /// println!("not blocking");
  /// ```
  ///
  /// The returned thread optionally holds the serialized result from the worker. This can be deserialized using serde_json in order to
  /// get a proper result. This is done under the hood for you.
  /// ```
  /// use node_workers::{WorkerPool};
  /// # use std::error::Error;
  ///
  /// # fn main() -> Result<(), Box<dyn Error>> {
  /// let mut pool = WorkerPool::setup(2);
  /// let thread = pool.run_worker("examples/worker", "fib2", 40u32);
  /// let result = thread.get_result::<u32>()?;
  /// println!("run_worker result: {:#?}", result);
  /// # Ok(())
  /// # }
  /// ```
  pub fn run_worker<P: AsPayload>(
    &mut self,
    file_path: &str,
    cmd: &str,
    payload: P,
  ) -> WorkerThread {
    let payload = payload.to_payload();
    let file_path = file_path.to_string();
    let cmd = cmd.to_string();
    let inner = self.inner.clone();

    // spawn a thread so that inner.get_available_worker() doesn't block
    let handle = std::thread::spawn(move || {
      let inner = inner.clone();
      let mut pool = inner.lock().unwrap();
      let res = pool.run_worker(file_path, cmd, payload);
      drop(pool);
      res.join().unwrap()
    });
    WorkerThread::from_handle(handle)
  }

  /// Dispatch a task between available workers with a set of payloads.
  /// This mobilize a worker for each payload. As soon as a worker is free, it'll be assigned right away a new task until all payloads have been processed.
  /// Contrarily to `run_worker`, this method is blocking and directly return the result from all workers.
  /// ```
  /// use node_workers::{WorkerPool};
  /// # use std::error::Error;
  ///
  /// # fn main() -> Result<(), Box<dyn Error>> {
  /// let mut pool = WorkerPool::setup(2);
  /// pool.with_debug(true);
  /// let payloads = vec![10, 20, 30, 40];
  /// let result = pool.perform::<u64, _>("examples/worker", "fib2", payloads).unwrap();
  /// println!("result: {:#?}", result);
  /// # Ok(())
  /// # }
  /// ```
  /// ## Errors
  ///
  /// Each worker is run in a thread, and `perform()` will return an error variant if one of them panick.
  pub fn perform<T: DeserializeOwned, P: AsPayload>(
    &mut self,
    file_path: &str,
    cmd: &str,
    payloads: Vec<P>,
  ) -> Result<Vec<Option<T>>> {
    let debug = self.inner.lock().unwrap().debug;
    print_debug!(debug, "[pool] running tasks");
    let mut handles = Vec::new();
    for (n, payload) in payloads.into_iter().map(|x| x.to_payload()).enumerate() {
      print_debug!(debug, "[pool] (task {}) start of iteration", n);
      let handle =
        self
          .inner
          .lock()
          .unwrap()
          .run_worker(file_path.to_string(), cmd.to_string(), payload);
      handles.push(handle);
      print_debug!(debug, "[pool] (task {}) end of iteration", n);
    }

    handles
      .into_iter()
      .enumerate()
      .map(|(n, x)| {
        print_debug!(debug, "[pool] (thread {}) joined", n);
        let res = x.get_result::<T>();
        if let Ok(res) = res {
          Ok(res)
        } else {
          bail!("failed to join thread")
        }
      })
      .collect::<Result<Vec<_>, _>>()
  }

  /// Boot a maximum of *n* workers, making them ready to take on a task right away.
  /// ```rust
  /// use node_workers::{WorkerPool};
  ///
  /// let mut pool = WorkerPool::setup(2);
  /// let handle = pool.warmup(2, "examples/worker");
  ///
  /// //... some intensive task on the main thread
  ///
  /// handle.join().expect("Couldn't warmup workers");
  /// //... task workers
  /// ```
  pub fn warmup(&self, nbr_workers: usize, file_path: &str) -> JoinHandle<()> {
    let inner = self.inner.clone();
    let file_path = file_path.to_string();
    std::thread::spawn(move || {
      inner
        .lock()
        .unwrap()
        .warmup(nbr_workers, &file_path)
        .unwrap()
    })
  }
}

#[cfg(test)]
mod tests {
  use crate::worker_pool::WorkerPool;

  #[test]
  pub fn create_worker_when_needed() {
    let pool = WorkerPool::setup(1);
    assert_eq!(pool.inner.lock().unwrap().workers.len(), 0);

    pool.inner.lock().unwrap().get_available_worker();
    assert_eq!(pool.inner.lock().unwrap().workers.len(), 1);
  }

  #[test]
  pub fn same_idle_worker() {
    let pool = WorkerPool::setup(1);
    let worker = pool.inner.lock().unwrap().get_available_worker();
    worker.lock().unwrap().idle = true;
    let worker_id = worker.lock().unwrap().id;
    let other_worker_id = pool
      .inner
      .lock()
      .unwrap()
      .get_available_worker()
      .lock()
      .unwrap()
      .id;
    assert_eq!(worker_id, other_worker_id);
  }

  #[test]
  pub fn create_new_worker_when_busy() {
    let pool = WorkerPool::setup(2);
    pool
      .inner
      .lock()
      .unwrap()
      .run_worker("examples/worker".into(), "fib2".into(), 40);

    let worker_id = pool
      .inner
      .lock()
      .unwrap()
      .get_available_worker()
      .lock()
      .unwrap()
      .id;
    println!("got worker_id");
    assert_eq!(worker_id, 2);
  }

  #[test]
  pub fn reuse_worker_when_full() {
    let pool = WorkerPool::setup(1);
    pool
      .inner
      .lock()
      .unwrap()
      .run_worker("examples/worker".into(), "fib2".into(), 40);

    let worker_id = pool
      .inner
      .lock()
      .unwrap()
      .get_available_worker()
      .lock()
      .unwrap()
      .id;
    assert_eq!(worker_id, 1);
  }

  #[test]
  pub fn warmup() {
    let mut pool = WorkerPool::setup(2);
    pool.with_debug(true);
    pool.warmup(2, "examples/worker").join().unwrap();

    let workers = pool.inner.lock().unwrap().workers.clone();
    for worker in workers {
      assert_eq!(worker.lock().unwrap().ready, true);
    }
  }

  #[test]
  pub fn error_invalid_command() {
    {
      let mut pool = WorkerPool::setup(1);
      let res = pool.run_worker("foo", "fib2", 40).join();
      println!("{:?}", res);
      assert_eq!(true, matches!(res, Err(_)));
    }

    {
      let mut pool = WorkerPool::setup(1);
      let res = pool.perform::<(), _>("foo", "fib2", vec![40]);
      assert_eq!(true, matches!(res, Err(_)));
    }

    {
      let pool = WorkerPool::setup(1);
      let res = pool.warmup(1, "foo").join();
      assert_eq!(true, matches!(res, Err(_)));
    }
  }

  #[test]
  pub fn error_task_throws() {
    {
      let mut pool = WorkerPool::setup(1);
      let res = pool.run_worker("examples/worker", "error", 40).join();
      assert_eq!(true, matches!(res, Err(_)));
    }

    {
      let mut pool = WorkerPool::setup(1);
      let res = pool.perform::<(), _>("examples/worker", "error", vec![40]);
      assert_eq!(true, matches!(res, Err(_)));
    }
  }

  #[test]
  pub fn error_task_not_found() {
    {
      let mut pool = WorkerPool::setup(1);
      let res = pool.run_worker("examples/worker", "no", 40).join();
      assert_eq!(true, matches!(res, Err(_)));
    }

    {
      let mut pool = WorkerPool::setup(1);
      let res = pool.perform::<(), _>("examples/worker", "no", vec![40]);
      assert_eq!(true, matches!(res, Err(_)));
    }
  }
}
