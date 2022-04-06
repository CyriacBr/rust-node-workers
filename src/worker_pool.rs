use serde::{de::DeserializeOwned, Deserialize};
use serde_json::Value;
use std::{
  cell::RefCell,
  rc::Rc,
  sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc::{self, Receiver, Sender},
    Arc, Mutex,
  },
};

use crate::worker::Worker;

pub struct WorkerPool {
  pub workers: Vec<Arc<Mutex<Worker>>>,
  pub max_workers: usize,
  pub waiting: Arc<Mutex<AtomicUsize>>,
}

impl WorkerPool {
  pub fn setup(max_workers: usize) -> Self {
    WorkerPool {
      workers: Vec::new(),
      max_workers,
      waiting: Arc::new(Mutex::new(AtomicUsize::new(0))),
    }
  }

  pub fn run_task<T: DeserializeOwned>(
    &mut self,
    file_path: &str,
    cmd: &str,
    payloads: Vec<Option<Value>>,
  ) -> Vec<Option<T>> {
    println!("[pool] running tasks");
    let mut handles = Vec::new();
    for (n, payload) in payloads.into_iter().enumerate() {
      println!("[pool] (task {}) start of iteration", n);
      let worker = self.get_available_worker();
      println!(
        "[pool] (task {}) got worker {}",
        n,
        worker.lock().unwrap().id
      );
      let file_path = String::from(file_path);
      // let (sender, receiver) = mpsc::channel();
      let waiting = self.waiting.clone();
      let cmd = cmd.to_string();

      let handle = std::thread::spawn(move || {
        let worker = worker.clone();
        worker.lock().unwrap().init(file_path.as_str());
        // sender.send(1);
        *waiting.lock().unwrap().get_mut() += 1;
        let res = worker.lock().unwrap().perform_task(cmd, payload);
        println!(
          "[pool] performed task on worker {}",
          worker.lock().unwrap().id
        );
        *waiting.lock().unwrap().get_mut() -= 1;
        // sender.send(-1);
        return res;
      });

      handles.push(handle);
      println!("[pool] (task {}) end of iteration", n);
    }

    handles
      .into_iter()
      .enumerate()
      .map(|(n, x)| {
        let str = x.join().unwrap();
        println!("[pool] (thread {}) result: {:?}", n, str);
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
      println!("[pool] found idle worker");
      return idle_worker.clone();
    }
    if self.workers.len() < self.max_workers {
      let mut worker = Worker::new(self.workers.len() + 1);
      worker.idle = false;
      self.workers.push(Arc::new(Mutex::new(worker)));
      println!("[pool] created new worker");
      return self.workers.last().unwrap().clone();
    }
    println!("[pool] waiting for worker to be free");
    loop {
      if self.waiting.lock().unwrap().load(Ordering::SeqCst) == 0 {
        println!("[pool] pool is free");
        break;
      }
    }
    self.get_available_worker()
  }
}

// pub struct WorkerPool {
//   inner: Arc<WorkerPool>,
// }

// impl WorkerPool {
//   pub fn setup() -> WorkerPool {
//     WorkerPool {
//       inner: Arc::new(WorkerPool::setup()),
//     }
//   }

//   pub fn run_tasks(&mut self, file_path: &str) {
//     thread::scope(|s| {
//       for _n in 0..=4 {
//         let pool = self.inner.clone();
//         s.spawn(move |_| {
//           pool.run_task(file_path);
//         });
//       }
//     });
//   }
// }
