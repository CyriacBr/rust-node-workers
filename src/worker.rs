use std::{
  io::{BufRead, BufReader, Write},
  process::{Child, ChildStdin, ChildStdout, Command, Stdio},
  thread,
};

use serde_json::Value;

pub struct Worker {
  pub id: usize,
  pub child: Option<Child>,
  pub stdout: Option<BufReader<ChildStdout>>,
  pub stdin: Option<ChildStdin>,
  pub idle: bool,
  pub ready: bool,
}

impl Worker {
  pub fn new(id: usize) -> Worker {
    Worker {
      id,
      child: None,
      stdout: None,
      stdin: None,
      ready: false,
      idle: true,
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
    println!("[worker {}] child spawned", self.id);
    self.child = Some(child);
  }

  pub fn perform_task(&mut self, payload: Option<Value>) {
    self.idle = false;

    let mut reader = self.stdout.take().unwrap();
    let mut stdin = self.stdin.take().unwrap();

    if !self.ready {
      self.communicate("", "READY", &stdin, &mut reader);
      self.ready = true;
    }

    println!("[worker {}] is ready", self.id);
    if let Some(payload) = payload {
      let payload_str = payload.to_string();
      let chunks = payload_str
        .as_bytes()
        .chunks(1000)
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap();
      for chunk in chunks {
        self.communicate(&format!("PAYLOAD_CHUNK: {}", chunk), "", &stdin, &mut reader)
      }
      self.communicate("PAYLOAD_END", "PAYLOAD_OK", &stdin, &mut reader);
    }
    self.communicate("WORK", "OK", &stdin, &mut reader);

    self.stdout = Some(reader);
    self.stdin = Some(stdin);

    println!("[worker {}] task finished", self.id);
    self.idle = true;
  }

  pub fn communicate(
    &self,
    send: &str,
    wait: &str,
    mut stdin: &ChildStdin,
    reader: &mut BufReader<ChildStdout>,
  ) {
    if !send.is_empty() {
      println!("[worker {}] send {} to child stdin", self.id, send);
      stdin.write_all(format!("{}\n", send).as_bytes()).unwrap();
    }
    if !wait.is_empty() {
      println!("[worker {}] waiting for {}", self.id, wait);
      loop {
        let mut ln = String::new();
        reader.read_line(&mut ln).unwrap();
        println!("[worker {}] (stdout) {}", self.id, ln.clone().trim());
        if ln == format!("{}\n", wait) {
          println!("[worker {}] {} received", self.id, wait);
          break;
        }
      }
    }
  }
}
