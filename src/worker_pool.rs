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
use crossbeam::thread;

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

  pub fn run_task(&mut self, file_path: &str, times: usize) {
    println!("[pool] running tasks");
    let mut handles = Vec::new();
    for _n in 0..times {
      println!("[pool] (task {}) start of iteration", _n);
      let worker = self.get_available_worker();
      println!("[pool] (task {}) got worker {}", _n, worker.lock().unwrap().id);
      let file_path = String::from(file_path);
      // let (sender, receiver) = mpsc::channel();
      let waiting = self.waiting.clone();

      let handle = std::thread::spawn(move || {
        let worker = worker.clone();
        worker.lock().unwrap().init(file_path.as_str());
        // sender.send(1);
        *waiting.lock().unwrap().get_mut() += 1;
        worker.lock().unwrap().perform_task();
        println!("[pool] performed task on worker {}", worker.lock().unwrap().id);
        *waiting.lock().unwrap().get_mut() -= 1;
        // sender.send(-1);
      });

      handles.push(handle);
      println!("[pool] (task {}) end of iteration", _n);
    }
    for handle in handles {
      handle.join().unwrap();
    }
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
