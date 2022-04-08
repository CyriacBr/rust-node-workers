use std::{
  io::{BufRead, BufReader, Write},
  process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use serde_json::Value;

use crate::print_debug;

pub struct Worker {
  pub id: usize,
  pub child: Option<Child>,
  pub stdout: Option<BufReader<ChildStdout>>,
  pub stdin: Option<ChildStdin>,
  pub idle: bool,
  pub ready: bool,
  pub debug: bool,
}

// TODO: handle cmd error
impl Worker {
  pub fn new(id: usize, debug: bool) -> Worker {
    Worker {
      id,
      child: None,
      stdout: None,
      stdin: None,
      ready: false,
      idle: true,
      debug,
    }
  }

  pub fn init(&mut self, file_path: &str) {
    if self.child.is_some() {
      return;
    }
    let mut child = Command::new("node")
      .arg(file_path)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .spawn()
      .expect("failed to execute process");
    self.stdin = Some(child.stdin.take().unwrap());
    self.stdout = Some(BufReader::new(child.stdout.take().unwrap()));
    print_debug!(self.debug, "[worker {}] child spawned", self.id);
    self.child = Some(child);
  }

  pub fn perform_task(&mut self, cmd: String, payload: Value) -> Option<String> {
    self.idle = false;

    let mut reader = self.stdout.take().unwrap();
    let stdin = self.stdin.take().unwrap();

    if !self.ready {
      self.communicate("", "READY", &stdin, &mut reader);
      self.ready = true;
    }

    print_debug!(self.debug, "[worker {}] is ready", self.id);
    if !payload.is_null() {
      let payload_str = payload.to_string();
      let chunks = payload_str
        .as_bytes()
        .chunks(1000)
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap();
      for chunk in chunks {
        self.communicate(
          &format!("PAYLOAD_CHUNK: {}", chunk),
          "",
          &stdin,
          &mut reader,
        );
      }
      self.communicate("PAYLOAD_END", "PAYLOAD_OK", &stdin, &mut reader);
    }
    let result_str = self.communicate(&format!("CMD: {}", cmd), "OK", &stdin, &mut reader);

    self.stdout = Some(reader);
    self.stdin = Some(stdin);

    print_debug!(self.debug, "[worker {}] task finished", self.id);
    self.idle = true;

    result_str
  }

  pub fn communicate(
    &self,
    send: &str,
    wait: &str,
    mut stdin: &ChildStdin,
    reader: &mut BufReader<ChildStdout>,
  ) -> Option<String> {
    if !send.is_empty() {
      print_debug!(
        self.debug,
        "[worker {}] send {} to child stdin",
        self.id,
        send
      );
      stdin.write_all(format!("{}\n", send).as_bytes()).unwrap();
    }
    if !wait.is_empty() {
      print_debug!(self.debug, "[worker {}] waiting for {}", self.id, wait);
      let mut payload_str = String::new();
      loop {
        let mut ln = String::new();
        reader.read_line(&mut ln).unwrap();
        if ln.trim().is_empty() {
          continue;
        }
        print_debug!(
          self.debug,
          "[worker {}] (stdout) {}",
          self.id,
          ln.clone().trim()
        );
        if ln == format!("{}\n", wait) {
          print_debug!(self.debug, "[worker {}] {} received", self.id, wait);
          return if payload_str.is_empty() {
            None
          } else {
            Some(payload_str)
          };
        } else if ln.starts_with("RESULT_CHUNK:") {
          print_debug!(self.debug, "[worker {}] received result chunk", self.id);
          payload_str += ln.replace("RESULT_CHUNK:", "").trim();
        }
      }
    }
    None
  }
}
