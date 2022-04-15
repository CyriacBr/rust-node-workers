use std::thread::JoinHandle;

use anyhow::{bail, Ok, Result};
use serde::de::DeserializeOwned;

/// Wraps a `std::thread::JoinHandle` for convenience
#[derive(Debug)]
pub struct WorkerThread {
  inner: JoinHandle<Option<String>>,
}
impl WorkerThread {
  /// Create a wrapper arround an existing handle. That handle should return `Option<String>`
  pub fn from_handle(handle: JoinHandle<Option<String>>) -> WorkerThread {
    WorkerThread { inner: handle }
  }

  /// `join()` the inner handle
  pub fn join(self) -> std::thread::Result<Option<String>> {
    self.inner.join()
  }

  /// Join the handle and deserialize it's result.
  /// 
  /// ## Errors
  /// 
  /// Whill return an error variant if the thread panicked during `join()`.
  pub fn get_result<R: DeserializeOwned>(self) -> Result<Option<R>> {
    match self.join() {
      std::thread::Result::Ok(join) => {
        let result = join.map(|x| serde_json::from_str::<R>(x.as_str()).unwrap());
        Ok(result)
      }
      std::thread::Result::Err(_) => bail!("thread panicked"),
    }
  }
}
