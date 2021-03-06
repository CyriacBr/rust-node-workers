use std::path::PathBuf;

use serde_json::{json, Value};

/// Represents an empty payload that can be sent to a node worker
/// ```
/// use node_workers::{EmptyPayload, WorkerPool};
/// # use std::error::Error;
///
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let mut pool = WorkerPool::setup("examples/worker", 1);
/// let payloads = vec![EmptyPayload::new(), EmptyPayload::new()];
/// pool.perform::<(), _>("ping", payloads)?;
/// # Ok(())
/// # }
/// ```
pub struct EmptyPayload {}
impl EmptyPayload {
  pub fn new() -> EmptyPayload {
    EmptyPayload {}
  }
  /// Convenient method to create an array of empty payload
  /// ```
  /// use node_workers::{EmptyPayload, WorkerPool};
  /// # use std::error::Error;
  ///
  /// # fn main() -> Result<(), Box<dyn Error>> {
  /// let mut pool = WorkerPool::setup("examples/worker", 1);
  /// let payloads = EmptyPayload::bulk(2);
  /// pool.perform::<(), _>("ping", payloads)?;
  /// # Ok(())
  /// # }
  /// ```
  pub fn bulk(n: u32) -> Vec<EmptyPayload> {
    (0..n).into_iter().map(|_| EmptyPayload::new()).collect()
  }
}
impl Default for EmptyPayload {
  fn default() -> Self {
    Self::new()
  }
}
impl AsPayload for EmptyPayload {
  fn to_payload(self) -> Value {
    Value::Null
  }
}

/// Represent a data that can be sent to a node worker.
/// Under the hood, node worker can only receive and transfer back serde_json::Value.
/// This trait is mainly for convenience as it is already implemented for all primitive types, and lets you
/// send all kinds of data to a node worker without boilerplate.
pub trait AsPayload {
  fn to_payload(self) -> Value;
}

impl AsPayload for Value {
  fn to_payload(self) -> Value {
    self
  }
}

impl<T: AsPayload> AsPayload for Option<T> {
  fn to_payload(self) -> Value {
    if let Some(val) = self {
      val.to_payload()
    } else {
      Value::Null
    }
  }
}

impl AsPayload for PathBuf {
  fn to_payload(self) -> Value {
    self.to_str().unwrap().to_payload()
  }
}

macro_rules! impl_all {
    ($($ty: ty),*) => {
        $(
            impl AsPayload for $ty {
                fn to_payload(self) -> Value {
                    json!({ "_inner_payload": self})
                }
            }
        )*
    }
}

/// A macro to quickly create an array of payload. This is usefull for running a task with payloads of different types.
/// ```
/// use node_workers::{EmptyPayload, WorkerPool, AsPayload, make_payloads};
/// # use std::error::Error;
///
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let mut pool = WorkerPool::setup("examples/worker", 1);
/// let payloads = make_payloads!(EmptyPayload::new(), 20, "test");
/// pool.perform::<(), _>("ping", payloads)?;
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! make_payloads {
    ( $( $a:expr ),* ) => {
      {
        let mut vec: Vec<serde_json::Value> = Vec::new();
        $( vec.push($a.to_payload()); )*
        vec
      }
    };
  }

impl_all!(usize, isize, u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, f32, f64, &str, String);
