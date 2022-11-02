use std::{sync::Mutex, time::Instant, vec, process::Child};
use crate::result::{Result, bail};
use std::io::{BufReader};
use std::sync::Arc;
use std::io::BufRead;
use std::sync::mpsc::{channel};
use std::time::Duration;
use std::process::{Command, Stdio};

static PROCESS_DELAY: u64 = 200;

pub struct Project {
  pub name: String,
  pub executable: String,
  pub workdir: String,
  pub output: Arc<Mutex<String>>,
  pub child: Option<Child>,
  pub offset: Mutex<i32>,
  pub status: Arc<Mutex<ProcessStatus>>,
}

pub struct ProcessStatus {
  pub started_at: Option<Instant>,
  pub is_running: bool,
}

impl Project {
  pub fn new(name: String, executable: String, workdir: String) -> Self {
    Project {
      name,
      executable,
      workdir,
      output: Arc::new(Mutex::new("".to_string())),
      child: None,
      offset: Mutex::new(0),
      status: Arc::new(
        Mutex::new(
          ProcessStatus {
            is_running: false,
            started_at: None
          }
        )
      )
    }
  }

  pub fn stop(&mut self) -> Result<bool>  {
    let mut status = self.status.lock().unwrap();

    if !status.is_running {
      return Ok(false);
    }

    if let Some(mut child) = self.child.take() {
      status.is_running = false;
      self.output.lock().unwrap().clear();
      child.kill()?;
      child.wait()?;

      Ok(true)
    } else {
      Ok(false)
    }
  }

  pub fn run(&mut self) -> Result<()> {
    let mut status = self.status.lock().unwrap();

    if status.is_running {
      bail!("Project already running");
    }

    status.is_running = true;
    status.started_at = Some(Instant::now());
    let mut child = Command::new("/bin/bash")
      .arg("-c")
      .arg(self.executable.as_str())
      .current_dir(self.workdir.as_str())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .stdin(Stdio::null())
      .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let (sender, receiver) = channel();
    let stdout_sender = sender.clone();

    // TODO: push data into temp vector and update it each n seconds
    std::thread::spawn(move || {
      let reader = BufReader::new(stdout);

      for line in reader.lines() {
        stdout_sender.send(line.unwrap()).unwrap();
      }
    });

    let stderr_sender = sender.clone();
    std::thread::spawn(move || {
      let reader = BufReader::new(stderr);

      for line in reader.lines() {
        stderr_sender.send(line.unwrap()).unwrap();
      }
    });

    let out = self.output.clone();
    std::thread::spawn(move || {
      loop {
        let mut buff = vec![];

        for line in receiver.try_iter() {
          buff.push(line);
        }

        if !buff.is_empty() {
          out.lock().unwrap().push_str(buff.join("\n").as_str());
        }

        std::thread::sleep(Duration::from_millis(PROCESS_DELAY));
      }
    });

    self.child = Some(child);

    Ok(())
  }

  // pub fn lines(&mut self, width: u16) -> Vec<String> {
  //   self.output.lock()
  //     .unwrap()
  //     .lines()
  //     .flat_map(|line| {
  //       let mut curr = line;
  //       let mut sub_lines = vec![];
  //
  //       while curr.len() > width as usize {
  //           let (a,b) = curr.split_at(width as usize);
  //           sub_lines.push(a);
  //           curr = b;
  //       }
  //
  //       if !curr.is_empty() {
  //           sub_lines.push(curr);
  //       }
  //
  //       sub_lines
  //     })
  //     .map(str::to_owned)
  //     .collect()
  // }

  pub fn lines(&mut self, width: u16) -> Vec<String> {
    self.output.lock()
      .unwrap()
      .lines()
      .flat_map(|line| {
        let chars: Vec<char> = line.chars().collect();
        chars.chunks(width as usize)
          .map(|ch| {
            ch.into_iter().collect::<String>()
          })
          .collect::<Vec<String>>()
      })
      // .map(str::to_owned)
      .collect()
  }
}
