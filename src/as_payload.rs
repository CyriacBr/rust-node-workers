use serde_json::{json, Value};

pub struct EmptyPayload {}
impl EmptyPayload {
  pub fn new() -> EmptyPayload {
    EmptyPayload {}
  }
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
