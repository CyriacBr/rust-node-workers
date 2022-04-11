use crate::{as_payload::AsPayload, print_debug, worker::Worker};
use anyhow::{bail, Result};
use serde::de::DeserializeOwned;
use std::{
  sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
  },
  thread::JoinHandle,
};

/// A pool of nodejs workers
pub struct WorkerPool {
  workers: Vec<Arc<Mutex<Worker>>>,
  max_workers: usize,
  busy_counter: Arc<AtomicUsize>,
  debug: bool,
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
      workers: Vec::new(),
      max_workers,
      busy_counter: Arc::new(AtomicUsize::new(0)),
      debug: false,
    }
  }

  /// Enable or disable logging
  pub fn with_debug(&mut self, debug: bool) {
    self.debug = debug;
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
  /// get a proper result.
  /// ```
  /// use node_workers::{WorkerPool};
  ///
  /// let mut pool = WorkerPool::setup(2);
  /// let handle = pool.run_worker("examples/worker", "fib2", 40u32);
  /// let result = handle
  ///   .join()
  ///   .unwrap()
  ///   .map(|x| serde_json::from_str::<u32>(x.as_str()).unwrap())
  ///   .unwrap();
  /// println!("run_worker result: {}", result);
  /// ```
  pub fn run_worker<P: AsPayload>(
    &mut self,
    file_path: &str,
    cmd: &str,
    payload: P,
  ) -> JoinHandle<Option<String>> {
    let worker = self.get_available_worker();
    self.busy_counter.fetch_add(1, Ordering::SeqCst);
    print_debug!(
      self.debug,
      "[pool] got worker {}",
      worker.lock().unwrap().id
    );
    let file_path = String::from(file_path);
    let waiting = self.busy_counter.clone();
    let cmd = cmd.to_string();
    let debug = self.debug;
    let payload = payload.to_payload();

    std::thread::spawn(move || {
      let worker = worker.clone();
      worker.lock().unwrap().init(file_path.as_str()).unwrap();
      let res = worker
        .lock()
        .unwrap()
        .perform_task(cmd, payload)
        .expect("perform task");
      print_debug!(
        debug,
        "[pool] performed task on worker {}",
        worker.lock().unwrap().id
      );
      waiting.fetch_sub(1, Ordering::SeqCst);
      res
    })
  }

  /// Dispatch a task between available workers with a set of payloads.
  /// The length of the payloads defines how many workers are mobilised. But this also depends on the maximum number of
  /// allowed workers. As soon as a worker is free, it'll be assigned right away a new task untill all payloads have been sent.
  /// Contrarly to `run_worker`, this method is blocking and directly return the result from all workers.
  /// ```
  /// use node_workers::{WorkerPool};
  /// let mut pool = WorkerPool::setup(2);
  /// pool.with_debug(true);
  /// let payloads = vec![10, 20, 30, 40];
  /// let result = pool.run_task::<u64, _>("examples/worker", "fib2", payloads).unwrap();
  /// println!("result: {:?}", result);
  /// ```
  pub fn run_task<T: DeserializeOwned, P: AsPayload>(
    &mut self,
    file_path: &str,
    cmd: &str,
    payloads: Vec<P>,
  ) -> Result<Vec<Option<T>>> {
    print_debug!(self.debug, "[pool] running tasks");
    let mut handles = Vec::new();
    for (n, payload) in payloads.into_iter().map(|x| x.to_payload()).enumerate() {
      print_debug!(self.debug, "[pool] (task {}) start of iteration", n);
      let handle = self.run_worker(file_path, cmd, payload);
      handles.push(handle);
      print_debug!(self.debug, "[pool] (task {}) end of iteration", n);
    }

    handles
      .into_iter()
      .enumerate()
      .map(|(n, x)| {
        let str = x.join();
        if let Ok(str) = str {
          print_debug!(self.debug, "[pool] (thread {}) result: {:?}", n, str);
          Ok(str.map(|x| serde_json::from_str(&x).unwrap()))
        } else {
          bail!("failed to join thread")
        }
      })
      .collect::<Result<Vec<_>, _>>()
  }

  fn get_available_worker(&mut self) -> Arc<Mutex<Worker>> {
    let idle_worker = self.workers.iter().find(|w| {
      if let Ok(w) = w.try_lock() {
        return w.idle;
      }
      false
    });
    if let Some(idle_worker) = idle_worker {
      idle_worker.lock().unwrap().idle = false;
      print_debug!(self.debug, "[pool] found idle worker");
      return idle_worker.clone();
    }
    if self.workers.len() < self.max_workers {
      let mut worker = Worker::new(self.workers.len() + 1, self.debug);
      worker.idle = false;
      self.workers.push(Arc::new(Mutex::new(worker)));
      print_debug!(self.debug, "[pool] created new worker");
      return self.workers.last().unwrap().clone();
    }
    print_debug!(self.debug, "[pool] waiting for worker to be free");
    loop {
      if self.busy_counter.load(Ordering::SeqCst) == 0 {
        print_debug!(self.debug, "[pool] pool is free");
        break;
      }
    }
    self.get_available_worker()
  }
}

#[cfg(test)]
mod tests {
  use crate::worker_pool::WorkerPool;

  #[test]
  pub fn create_worker_when_needed() {
    let mut pool = WorkerPool::setup(1);
    assert_eq!(pool.workers.len(), 0);

    pool.get_available_worker();
    assert_eq!(pool.workers.len(), 1);
  }

  #[test]
  pub fn same_idle_worker() {
    let mut pool = WorkerPool::setup(1);
    let worker = pool.get_available_worker();
    worker.lock().unwrap().idle = true;
    let worker_id = worker.lock().unwrap().id;
    let other_worker_id = pool.get_available_worker().lock().unwrap().id;
    assert_eq!(worker_id, other_worker_id);
  }

  #[test]
  pub fn create_new_worker_when_busy() {
    let mut pool = WorkerPool::setup(2);
    pool.run_worker("examples/worker", "fib2", 40);

    let worker_id = pool.get_available_worker().lock().unwrap().id;
    assert_eq!(worker_id, 2);
  }

  #[test]
  pub fn reuse_worker_when_full() {
    let mut pool = WorkerPool::setup(1);
    pool.run_worker("examples/worker", "fib2", 40);

    let worker_id = pool.get_available_worker().lock().unwrap().id;
    assert_eq!(worker_id, 1);
  }

  #[test]
  pub fn error_invalid_command() {
    {
      let mut pool = WorkerPool::setup(1);
      let res = pool.run_worker("foo", "fib2", 40).join();
      assert_eq!(true, matches!(res, Err(_)));
    }

    {
      let mut pool = WorkerPool::setup(1);
      let res = pool.run_task::<(), _>("foo", "fib2", vec![40]);
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
      let res = pool.run_task::<(), _>("examples/worker", "error", vec![40]);
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
      let res = pool.run_task::<(), _>("examples/worker", "no", vec![40]);
      assert_eq!(true, matches!(res, Err(_)));
    }
  }
}
