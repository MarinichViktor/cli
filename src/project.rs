use std::str::FromStr;
use std::{sync::Mutex, time::Instant, vec, process::Child, io};
use std::result::{Result};
use std::error::{Error};
use std::io::{BufReader, Read, Stdin};
use std::sync::Arc;
use std::thread::JoinHandle;

pub struct Project {
  pub name: String,
  pub executable: String,
  pub workdir: String,
  pub output: Arc<Mutex<String>>,
  pub started_at: Option<Instant>,
  pub finished_at: Option<Instant>,
  pub child: Option<Arc<Mutex<Child>>>,
  pub join_handle: Option<JoinHandle<()>>
}

impl Project {
  pub fn new(name: String, executable: String, workdir: String) -> Self {
    let n = name.clone();
    Project {
      name,
      executable,
      workdir,
      output: Arc::new(Mutex::new(format!("{} -- {}", n, "Some interesting\n content".to_string()))),
      started_at: None,
      finished_at: None,
      child: None,
      join_handle: None
    }
  }

  pub fn run(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
    if self.started_at.is_some() {
      return Err(Box::new(std::fmt::Error));
    }
  
    use std::process::{Command, Stdio};

    self.started_at = Some(Instant::now());
    let child = Command::new("/bin/bash")
      .arg("-c")
      .arg(self.executable.as_str())
      .current_dir(self.workdir.as_str())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .stdin(Stdio::piped())
      .spawn()?;

    let child = Arc::new(Mutex::new(child));
    let mut thread_child = child.clone();
    self.child = Some(child);

    let mut process_output = self.output.clone();

    use std::io::BufRead;

    let join_handle = std::thread::spawn(move || {
      let stdout = thread_child.lock().unwrap().stdout.take().unwrap();
      let mut reader = BufReader::new(stdout);

      for line in reader.lines() {
        process_output.lock().unwrap().push_str(line.unwrap().as_str());
      }
    });
    self.join_handle = Some(join_handle);

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