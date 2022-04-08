use serde::de::DeserializeOwned;
use std::sync::{
  atomic::{AtomicUsize, Ordering},
  Arc, Mutex,
};

use crate::{as_payload::AsPayload, print_debug, worker::Worker};

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

  pub fn run_task<T: DeserializeOwned, P: AsPayload>(
    &mut self,
    file_path: &str,
    cmd: &str,
    payloads: Vec<P>,
  ) -> Vec<Option<T>> {
    print_debug!(self.debug, "[pool] running tasks");
    let mut handles = Vec::new();
    for (n, payload) in payloads.into_iter().map(|x| x.to_payload()).enumerate() {
      print_debug!(self.debug, "[pool] (task {}) start of iteration", n);
      let worker = self.get_available_worker();
      self.busy_counter.fetch_add(1, Ordering::SeqCst);
      print_debug!(
        self.debug,
        "[pool] (task {}) got worker {}",
        n,
        worker.lock().unwrap().id
      );
      let file_path = String::from(file_path);
      let waiting = self.busy_counter.clone();
      let cmd = cmd.to_string();
      let debug = self.debug;

      let handle = std::thread::spawn(move || {
        let worker = worker.clone();
        worker.lock().unwrap().init(file_path.as_str());
        let res = worker.lock().unwrap().perform_task(cmd, payload);
        print_debug!(
          debug,
          "[pool] performed task on worker {}",
          worker.lock().unwrap().id
        );
        waiting.fetch_sub(1, Ordering::SeqCst);
        res
      });

      handles.push(handle);
      print_debug!(self.debug, "[pool] (task {}) end of iteration", n);
    }

    handles
      .into_iter()
      .enumerate()
      .map(|(n, x)| {
        let str = x.join().unwrap();
        print_debug!(self.debug, "[pool] (thread {}) result: {:?}", n, str);
        str.map(|x| serde_json::from_str(&x).unwrap())
      })
      .collect()
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
