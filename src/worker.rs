use std::{
  io::{BufRead, BufReader, Write},
  process::{Child, ChildStdin, ChildStdout, Command, Stdio},
  thread,
};

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

  pub fn perform_task(&mut self) {
    self.idle = false;

    let mut reader = self.stdout.take().unwrap();
    let mut stdin = self.stdin.take().unwrap();

    if self.ready {
      println!("[worker {}] already ready", self.id);
      println!("[worker {}] send WORK to child stdin", self.id);
      stdin.write_all(b"WORK\n").unwrap();
      println!("[worker {}] waiting for OK", self.id);
      loop {
        let mut ln = String::new();
        reader.read_line(&mut ln).unwrap();
        println!("[worker {}] (stdout) {}", self.id, ln.clone().trim());
        if ln == "OK\n" {
          println!("[worker {}] OK received", self.id);
          break;
        }
      }
    } else {
      println!("[worker {}] waiting for ready", self.id);
      loop {
        let mut ln = String::new();
        reader.read_line(&mut ln).unwrap();
        println!("[worker {}] (stdout) {}", self.id, ln.clone().trim());
        if ln == "READY\n" {
          self.ready = true;
          println!("[worker {}] send WORK to child stdin", self.id);
          stdin.write_all(b"WORK\n").unwrap();
          println!("[worker {}] waiting for OK", self.id);
        } else if ln == "OK\n" {
          println!("[worker {}] OK received", self.id);
          break;
        }
      }
    }

    self.stdout = Some(reader);
    self.stdin = Some(stdin);

    println!("[worker {}] task finished", self.id);
    self.idle = true;
  }
}
