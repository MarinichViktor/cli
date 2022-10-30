use std::{sync::Mutex, time::Instant, vec, process::Child};
use std::result::{Result};
use std::error::{Error};
use std::io::{BufReader};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::io::BufRead;
use std::os::unix::io::RawFd;
use std::sync::mpsc::Sender;
use std::time::Duration;

pub struct Project {
  pub name: String,
  pub executable: String,
  pub workdir: String,
  pub output: Arc<Mutex<String>>,
  pub started_at: Option<Instant>,
  pub finished_at: Option<Instant>,
  pub child: Option<Child>,
  pub join_handle: Option<JoinHandle<()>>,
  output_notifier: Sender::<ProjectEvent>
}

impl Project {
  pub fn new(name: String, executable: String, workdir: String, output_notifier: Sender::<ProjectEvent>) -> Self {
    Project {
      name,
      executable,
      workdir,
      output_notifier,
      output: Arc::new(Mutex::new("".to_string())),
      started_at: None,
      finished_at: None,
      child: None,
      join_handle: None,
    }
  }

  pub fn run(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
    if self.started_at.is_some() {
      return Err(Box::new(std::fmt::Error));
    }
  
    use std::process::{Command, Stdio};

    self.started_at = Some(Instant::now());
    let mut child = Command::new("/bin/bash")
      .arg("-c")
      .arg(self.executable.as_str())
      .current_dir(self.workdir.as_str())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()?;


    let stdout = child.stdout.take().unwrap();
    let on_data = self.output_notifier.clone();
    let event_name = self.name.clone();

    std::thread::spawn(move || {
      let reader = BufReader::new(stdout);

      for line in reader.lines() {
        let formatted_line = format!("{}\n", line.unwrap());
        on_data.send(ProjectEvent::new(event_name.clone(), formatted_line)).unwrap();
      }
    });

    let stderr = child.stderr.take().unwrap();
    let on_err = self.output_notifier.clone();
    let event_name = self.name.clone();

    std::thread::spawn(move || {
      let reader = BufReader::new(stderr);
      for line in reader.lines() {
        let formatted_line = format!("{}\n", line.unwrap());
        on_err.send(ProjectEvent::new(event_name.clone(), formatted_line)).unwrap();
      }
    });

    self.child = Some(child);

    Ok(())
  }

  pub fn lines(&mut self, width: u16) -> Vec<String> {
    self.output.lock()
      .unwrap()
      .lines()
      .flat_map(|line| {
        let mut curr = line;
        let mut sublines = vec![];

        while curr.len() > width as usize {
            let (a,b) = curr.split_at(width as usize);
            sublines.push(a);
            curr = b;
        }

        if !curr.is_empty() {
            sublines.push(curr);
        }

        sublines
      })
      .map(str::to_owned)
      .collect()
  }
}


pub struct ProjectEvent {
  pub name: String,
  pub data: String
}

impl ProjectEvent {
  pub fn new(name: String, data: String) -> Self {
    ProjectEvent {
      name,
      data
    }
  }
}
