use anyhow::{bail, Context, Ok, Result};
use serde_json::Value;
use std::{
  io::{BufRead, BufReader, Write},
  process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

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

  pub fn init(&mut self, binary_args: Vec<String>, file_path: &str) -> Result<()> {
    if self.child.is_some() {
      return Ok(());
    }
    let bin = &binary_args[0];
    let mut args = binary_args[1..].to_vec();
    args.push(file_path.to_string());
    let mut child = Command::new(bin)
      .args(args)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .spawn()
      .context("execute process")?;
    self.stdin = Some(child.stdin.take().context("get process stdin")?);
    self.stdout = Some(BufReader::new(
      child.stdout.take().context("take process stdout")?,
    ));
    print_debug!(self.debug, "[worker {}] child spawned", self.id);
    self.child = Some(child);
    Ok(())
  }

  pub fn perform_task(&mut self, cmd: String, payload: Value) -> Result<Option<String>> {
    self.idle = false;

    let mut reader = self.stdout.take().unwrap();
    let stdin = self.stdin.take().unwrap();
    let mut child = self.child.take().unwrap();

    if !self.ready {
      self
        .communicate("", "READY", &stdin, &mut reader, &mut child)
        .context("communicating with process")?;
      self.ready = true;
    }

    print_debug!(self.debug, "[worker {}] is ready", self.id);
    if !payload.is_null() {
      let payload_str = payload.to_string();
      let chunks = payload_str
        .as_bytes()
        .chunks(1000)
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()?;
      for chunk in chunks {
        self
          .communicate(
            &format!("PAYLOAD_CHUNK: {}", chunk),
            "",
            &stdin,
            &mut reader,
            &mut child,
          )
          .context("communicating with process")?;
      }
      self
        .communicate("PAYLOAD_END", "PAYLOAD_OK", &stdin, &mut reader, &mut child)
        .context("communicating with process")?;
    }
    let result_str = self
      .communicate(
        &format!("CMD: {}", cmd),
        "OK",
        &stdin,
        &mut reader,
        &mut child,
      )
      .context("communicating with process")?;

    self.stdout = Some(reader);
    self.stdin = Some(stdin);
    self.child = Some(child);

    print_debug!(self.debug, "[worker {}] task finished", self.id);
    self.idle = true;

    Ok(result_str)
  }

  pub fn communicate(
    &self,
    send: &str,
    wait: &str,
    mut stdin: &ChildStdin,
    reader: &mut BufReader<ChildStdout>,
    child: &mut Child,
  ) -> Result<Option<String>> {
    let status = child.try_wait()?;
    if status.is_some() {
      bail!("process no longer running");
    }
    if !send.is_empty() {
      print_debug!(
        self.debug,
        "[worker {}] send {} to child stdin",
        self.id,
        send
      );
      stdin
        .write_all(format!("{}\n", send).as_bytes())
        .context("writing to process stdin")?;
    }
    if !wait.is_empty() {
      print_debug!(self.debug, "[worker {}] waiting for {}", self.id, wait);
      let mut payload_str = String::new();
      loop {
        let status = child.try_wait()?;
        if status.is_some() {
          bail!("process exited");
        }
        let mut ln = String::new();
        reader
          .read_line(&mut ln)
          .context("reading to process stdout")?;
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
            Ok(None)
          } else {
            Ok(Some(payload_str))
          };
        } else if ln.starts_with("RESULT_CHUNK:") {
          print_debug!(self.debug, "[worker {}] received result chunk", self.id);
          payload_str += ln.replace("RESULT_CHUNK:", "").trim();
        }
      }
    }
    Ok(None)
  }
}
