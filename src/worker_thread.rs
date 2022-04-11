use std::thread::JoinHandle;

use anyhow::{bail, Ok, Result};
use serde::de::DeserializeOwned;

pub struct WorkerThread {
  inner: JoinHandle<Option<String>>,
}
impl WorkerThread {
  pub fn from_handle(handle: JoinHandle<Option<String>>) -> WorkerThread {
    WorkerThread { inner: handle }
  }

  pub fn join(self) -> std::thread::Result<Option<String>> {
    self.inner.join()
  }

  pub fn get_result<R: DeserializeOwned>(self) -> Result<Option<R>> {
    match self.join() {
      std::thread::Result::Ok(join) => {
        let result = join.map(|x| serde_json::from_str::<R>(x.as_str()).unwrap());
        Ok(result)
      },
      std::thread::Result::Err(_) => bail!("thread panicked")
    }
  }
}
