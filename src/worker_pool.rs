use crate::{as_payload::AsPayload, print_debug, worker::Worker};
use anyhow::{Result, Context, bail};
use serde::de::DeserializeOwned;
use std::{
  sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
  },
  thread::JoinHandle,
};

pub struct WorkerPool {
  pub workers: Vec<Arc<Mutex<Worker>>>,
  pub max_workers: usize,
  pub busy_counter: Arc<AtomicUsize>,
  pub debug: bool,
}

impl WorkerPool {
  pub fn setup(max_workers: usize) -> Self {
    WorkerPool {
      workers: Vec::new(),
      max_workers,
      busy_counter: Arc::new(AtomicUsize::new(0)),
      debug: false,
    }
  }

  pub fn with_debug(&mut self, debug: bool) {
    self.debug = debug;
  }

  // fn watch_for_error(&self) {
  //   let workers = self.workers.clone();
  //   std::thread::spawn(|| {
  //     loop {
  //       let workers = workers.lock().unwrap();
  //       for w in workers.iter() {
  //         let child = w.lock().unwrap().child;
  //         if let Some(child) = child {

  //         }
  //       }
  //     }
  //   });
  // }

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

mod tests {
  use super::WorkerPool;

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
