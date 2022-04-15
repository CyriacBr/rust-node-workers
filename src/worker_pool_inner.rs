use anyhow::{bail, Result};

use crate::{print_debug, worker::Worker, worker_thread::WorkerThread, AsPayload};
use std::sync::{
  atomic::{AtomicUsize, Ordering},
  Arc, Mutex,
};

/// Struct responsible of the inner working of the pool
/// Needs to be wrapped in a Arc<Mutex<T>> for manipulations within different threads
pub struct WorkerPoolInner {
  pub binary_args: Vec<String>,
  pub workers: Vec<Arc<Mutex<Worker>>>,
  pub max_workers: usize,
  pub busy_counter: Arc<AtomicUsize>,
  pub debug: bool,
}

impl WorkerPoolInner {
  /// Create a new pool with some parameters
  pub fn setup(max_workers: usize) -> Self {
    WorkerPoolInner {
      binary_args: vec!["node".into()],
      workers: Vec::new(),
      max_workers,
      busy_counter: Arc::new(AtomicUsize::new(0)),
      debug: false,
    }
  }

  /// Refers to `WorkerPool::set_binary` for documentation
  pub fn set_binary(&mut self, binary: &str) {
    self.binary_args = shell_words::split(binary).expect("couldn't parse binary");
  }

  /// Refers to `WorkerPool::with_debug` for documentation
  pub fn with_debug(&mut self, debug: bool) {
    self.debug = debug;
  }

  /// Run a worker in a new thread. However, `get_available_worker` is executed on the main thread
  /// and therefor can block if the pool is waiting for an idle worker.
  pub fn run_worker<P: AsPayload>(
    &mut self,
    file_path: String,
    cmd: String,
    payload: P,
  ) -> WorkerThread {
    let worker = self.get_available_worker();
    self.busy_counter.fetch_add(1, Ordering::SeqCst);

    print_debug!(
      self.debug,
      "[pool] got worker {}",
      worker.lock().unwrap().id
    );
    let waiting = self.busy_counter.clone();
    let debug = self.debug;
    let binary_args = self.binary_args.clone();
    let payload = payload.to_payload();

    let handle = std::thread::spawn(move || {
      let worker = worker.clone();
      let mut worker = worker.lock().unwrap();
      worker.init(binary_args, file_path.as_str()).unwrap();
      let res = worker.perform_task(cmd, payload).expect("perform task");
      print_debug!(debug, "[pool] performed task on worker {}", worker.id);
      drop(worker);

      waiting.fetch_sub(1, Ordering::SeqCst);
      res
    });
    WorkerThread::from_handle(handle)
  }

  /// Find an idle worker that can take on a task.
  /// If no worker is free, and the capacity of the pool is not reached yet, a new worker is created.
  /// However, if the capacity is reached, this method will wait (and block) until a worker is idle.
  pub fn get_available_worker(&mut self) -> Arc<Mutex<Worker>> {
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

  pub fn warmup(&mut self, nbr_workers: usize, file_path: &str) -> Result<()> {
    let n = nbr_workers.clamp(0, self.max_workers - self.workers.len());
    let debug = self.debug;
    let ln = self.workers.len();
    let mut handles = Vec::new();
    for n in 0..n {
      let id = ln + n + 1;
      let worker = Worker::new(id, debug);
      let mutex = Arc::new(Mutex::new(worker));
      self.workers.push(mutex.clone());
      print_debug!(debug, "[pool] (warmup) created new worker");

      let binary_args = self.binary_args.clone();
      let file_path = file_path.to_string();
      let handle = std::thread::spawn(move || {
        let worker = mutex.clone();
        let mut worker = worker.lock().unwrap();
        worker.init(binary_args, &file_path).unwrap();
        worker.wait_for_ready().unwrap();
        print_debug!(debug, "[pool] (warmup) worker {} initialized", id);
      });
      handles.push(handle);
    }
    for handle in handles {
      if let std::thread::Result::Err(_) = handle.join() {
        bail!("thread panicked")
      }
    }
    Ok(())
  }
}
