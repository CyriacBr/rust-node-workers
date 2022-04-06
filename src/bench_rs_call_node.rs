use std::{
  io::{self, BufRead, BufReader, Read, Write},
  iter,
  process::Command,
  sync::{Arc, Mutex},
  thread,
};

use napi::{
  threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
  JsFunction, Result,
};
use serde_json::json;

use crate::worker_pool::WorkerPool;

#[napi]
pub fn do_rs_task_from_js_cb(task_cb: JsFunction) -> Result<()> {
  let tsfn: ThreadsafeFunction<u32, ErrorStrategy::CalleeHandled> = task_cb
    .create_threadsafe_function(0, |ctx| {
      ctx.env.create_uint32(ctx.value + 1).map(|v| vec![v])
    })?;
  let items: Vec<_> = iter::repeat(0).take(4).collect();

  let threads: Vec<_> = items
    .into_iter()
    .map(|n| {
      let tsfn = tsfn.clone();
      thread::spawn(move || {
        tsfn.call(Ok(n), ThreadsafeFunctionCallMode::Blocking);
      })
    })
    .collect();

  for handle in threads {
    handle.join().unwrap()
  }

  Ok(())
}

#[napi]
pub fn do_rs_task_from_cmd() {
  for i in 0..2 {
    let items: Vec<_> = iter::repeat(0).take(4).collect();

    let threads: Vec<_> = items
      .into_iter()
      .map(|n| {
        thread::spawn(move || {
          Command::new("node")
            .arg("task-inner.js")
            .spawn()
            .expect("command failed to start")
            .wait()
            .unwrap();
        })
      })
      .collect();

    for handle in threads {
      handle.join().unwrap()
    }
  }
}

#[napi]
pub fn do_rs_task_from_workers() {
  let mut pool = WorkerPool::setup(4);
  pool.run_task(
    "task-worker",
    vec![Some(json!({
        "value": 40,
    }))],
  );
}
