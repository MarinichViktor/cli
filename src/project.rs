use std::{sync::Mutex, time::Instant, vec};
use crate::result::{Result};
use std::io::{BufReader};
use std::sync::Arc;
use std::io::BufRead;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use std::process::{Command, Stdio};
use serde::{Deserialize, Serialize};

static PROCESS_DELAY: u64 = 100;
static OUTPUT_SIZE_THRESHOLD: usize = 3000;

// todo: to be refactored
pub struct Cmd {
  pub descriptor: CmdDescriptor,
  pub status: Arc<Mutex<Status>>,
  pub output: Arc<Mutex<CmdOutput>>,
  pub unsubscribe: Option<Box<dyn FnOnce() -> ()>>
}

impl<'a> From<CmdDescriptor> for Cmd {
  fn from(descriptor: CmdDescriptor) -> Self {
    Cmd::new(descriptor)
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CmdDescriptor {
  pub name: String,
  pub executable: String,
  pub workdir: String,
}

#[derive(Default)]
pub struct CmdOutput {
  pub data: Vec<String>,
  pub data_cache: (Vec<String>, usize),
  pub offset: i32
}

impl CmdOutput {
  pub fn clear(&mut self) {
    self.data.clear();
    self.data_cache.0.clear();
    self.offset = 0;
  }
}

pub struct Status {
  pub started_at: Option<Instant>,
  pub is_running: bool,
}

impl Cmd {
  pub fn new(descriptor: CmdDescriptor) -> Self {
    Cmd {
      descriptor,
      output: Arc::new(Mutex::new(CmdOutput::default())),
      unsubscribe: None,
      status: Arc::new(
        Mutex::new(
          Status {
            is_running: false,
            started_at: None
          }
        )
      )
    }
  }

  pub fn run(&mut self) -> Result<()> {
    let mut status = self.status.lock().unwrap();

    if status.is_running {
      return Ok(());
    }

    status.is_running = true;
    status.started_at = Some(Instant::now());

    let (data_stream, stop_cmd, close_stream) = self.spawn()?;
    self.unsubscribe = Some(Box::new(stop_cmd));
    let status = self.status.clone();
    let output = self.output.clone();

    std::thread::spawn(move || {
      loop {
        if !status.lock().unwrap().is_running {
          break;
        }

        for _ in close_stream.try_iter() {
          break;
        }

        for buff in data_stream.try_iter() {
          let mut data = output.lock().unwrap();
          data.data.append(&mut buff.clone());
          let cache_width = data.data_cache.1;

          if cache_width > 0 {
            data.data_cache.0.append(&mut Cmd::build_lines(&buff, cache_width));
          }

          let output_len = data.data.len();
          let cache_len = data.data_cache.0.len();

          if output_len > OUTPUT_SIZE_THRESHOLD {
            data.data = data.data.split_off(output_len - OUTPUT_SIZE_THRESHOLD)
          }

          if cache_len > OUTPUT_SIZE_THRESHOLD {
            data.data_cache.0 = data.data_cache.0.split_off(cache_len - OUTPUT_SIZE_THRESHOLD)
          }
        }

        std::thread::sleep(Duration::from_millis(PROCESS_DELAY));
      }
    });

    Ok(())
  }

  fn spawn(&self) -> Result<(Receiver<Vec<String>>, impl FnOnce() -> (), Receiver<()>)> {
    let mut child = Command::new("/bin/bash")
      .arg("-c")
      .arg(self.descriptor.executable.as_str())
      .current_dir(self.descriptor.workdir.as_str())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .stdin(Stdio::null())
      .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let (tx, rx) = channel();
    let stdout_tx = tx.clone();
    let (ctx, crx) = channel();

    std::thread::spawn(move || {
      let reader = BufReader::new(stdout);

      for line in reader.lines() {
        stdout_tx.send(line.unwrap()).unwrap();
      }

      let _ = ctx.send(());
    });

    let stderr_tx = tx.clone();
    std::thread::spawn(move || {
      let reader = BufReader::new(stderr);

      for line in reader.lines() {
        stderr_tx.send(line.unwrap()).unwrap();
      }
    });

    let (tx2, rx2) = channel();
    std::thread::spawn(move || {
      loop {
        let mut buff = vec![];

        for line in rx.try_iter() {
          buff.push(line);
        }

        if !buff.is_empty() {
          let _ = tx2.send(buff);
        }

        std::thread::sleep(Duration::from_millis(PROCESS_DELAY));
      }
    });

    let stop = move || {
      match child.try_wait() {
        Ok(_) => {},
        Err(_) => {
          child.kill().unwrap();
          child.wait().unwrap();
        }
      }
    };

    Ok((rx2, stop, crx))
  }

  pub fn stop(&mut self) -> Result<bool>  {
    let mut status = self.status.lock().unwrap();

    if !status.is_running {
      return Ok(false);
    }

    if let Some(unsubscriber) = self.unsubscribe.take() {
      status.is_running = false;
      self.output.lock().unwrap().clear();
      unsubscriber();
      Ok(true)
    } else {
      Ok(false)
    }
  }

  pub fn render(&self, w: usize, h: usize) -> Vec<String> {
    let mut output = self.output.lock().unwrap();

    if output.data_cache.1 != w {
      let cached_lines = Cmd::build_lines(&output.data, w);

      output.data_cache.0 = cached_lines;
      output.data_cache.1 = w;
    }

    if output.data_cache.0.len() > h {
      let start_index = ((output.data_cache.0.len() as i32)  - (h as i32) - output.offset).max(0).min((output.data_cache.0.len() - h) as i32) as usize;
      output.data_cache.0[start_index..(start_index + h)].to_vec()
    } else {
      output.data_cache.0.clone()
    }
  }

  fn build_lines(data: &[String], w: usize) -> Vec<String>{
    data.iter()
      .flat_map(|line| {
        let mut chars: Vec<char> = line.chars().collect();

        if chars.is_empty() {
          chars.push('\n');
        }

        chars.chunks(w)
            .map(|ch| {
              ch.into_iter().collect::<String>()
            })
            .collect::<Vec<String>>()

      })
      .collect::<Vec<String>>()
  }
}

#[cfg(test)]
mod project_tests {

}
