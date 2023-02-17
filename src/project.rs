use std::{sync::Mutex, time::Instant, vec, process::Child};
use crate::result::{Result};
use std::io::{BufReader};
use std::sync::Arc;
use std::io::BufRead;
use std::sync::mpsc::{channel};
use std::time::Duration;
use std::process::{Command, Stdio};
use serde::{Deserialize, Serialize};
use crate::buffer::Buff;

static PROCESS_DELAY: u64 = 200;

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectDescriptor {
  pub name: String,
  pub executable: String,
  pub workdir: String,
}

// todo: to be refactored
pub struct Project {
  pub name: String,
  pub executable: String,
  pub workdir: String,
  pub output: Arc<Mutex<Vec<String>>>,
  pub child: Option<Child>,
  pub offset: Arc<Mutex<i32>>,
  pub offset2: Arc<Mutex<(usize, usize)>>,
  pub status: Arc<Mutex<ProcessStatus>>,
  buff: Arc<Mutex<Buff>>
}

impl From<ProjectDescriptor> for Project {
  fn from(descriptor: ProjectDescriptor) -> Self {
    Project::new(descriptor.name, descriptor.executable, descriptor.workdir)
  }
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
      output: Arc::new(Mutex::new(vec![])),
      child: None,
      offset: Arc::new(Mutex::new(0)),
      offset2: Arc::new(Mutex::new((0, 0))),
      buff: Arc::new(Mutex::new(Buff::default())),
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
      return Ok(());
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

    // todo: to be refactored
    let project_status = self.status.clone();
    let project_output = self.output.clone();
    // TODO: push data into temp vector and update it each n seconds
    std::thread::spawn(move || {
      let reader = BufReader::new(stdout);

      for line in reader.lines() {
        stdout_sender.send(line.unwrap()).unwrap();
      }

      let mut status = project_status.lock().unwrap();
      status.started_at = None;
      status.is_running = false;
      project_output.lock().unwrap().clear();
    });

    let stderr_sender = sender.clone();
    std::thread::spawn(move || {
      let reader = BufReader::new(stderr);

      for line in reader.lines() {
        stderr_sender.send(line.unwrap()).unwrap();
      }
    });

    let out = self.output.clone();
    // let offset = self.offset.clone();
    std::thread::spawn(move || {
      loop {
        let mut buff = vec![];

        for line in receiver.try_iter() {
          buff.push(line);
        }

        // todo: store output in lines instead of string
        if !buff.is_empty() {
          let mut data = out.lock().unwrap();
          data.append(&mut buff);

          if data.len() > 10_000 {
            let (_, remain) = data.split_at(data.len() - 5_000);
            *data = remain.to_vec();
          }
        }

        std::thread::sleep(Duration::from_millis(PROCESS_DELAY));
      }
    });

    self.child = Some(child);

    Ok(())
  }

  pub fn lines(&mut self, width: u16) -> Vec<String> {
    self.output.lock()
      .unwrap()
      .iter()
      .flat_map(|line| {
        let mut chars: Vec<char> = line.chars().collect();
        if chars.is_empty() {
          chars.push('\n');
        }
        chars.chunks(width as usize)
          .map(|ch| {
            ch.into_iter().collect::<String>()
          })
          .collect::<Vec<String>>()
      })
      // .map(str::to_owned)
      .collect()
  }

  pub fn render(&self, w: u32, h: u32) -> Vec<String> {
    // alternative impl
    let mut v = vec![];
    let buff = self.buff.lock().unwrap();

    if buff.blocks.is_empty() {
      return v;
    }
    let (init_block_idx, init_line_idx) = *self.offset2.lock().unwrap();
    let mut block_idx = init_block_idx;
    let mut line_idx = init_line_idx;
    let mut total_lines = 0usize;

    loop {
      let block = buff.blocks.get(block_idx).unwrap();

      total_lines += &block.data()[line_idx..]
          .iter()
          .map(|s| {
            ((s.len() as f32)/(w as f32)).ceil() as usize
          })
          .sum::<usize>();

      if total_lines >= h as usize {
        break;
      }

      if block_idx == buff.blocks.len() -1 {
        break;
      }

      block_idx += 1;
      line_idx = 0;
    }

    let mut is_prepended = false;

    if total_lines < h as usize {
      if init_line_idx > 0 {
        block_idx = init_block_idx;
        line_idx = init_line_idx;
      } else if init_block_idx > 0 {
        block_idx = init_line_idx - 1;
        line_idx = buff.block_size as usize;
      }

      loop {
        let block = buff.blocks.get(block_idx).unwrap();

        total_lines += &block.data()[..line_idx]
            .iter()
            .map(|s| {
              ((s.len() as f32)/(w as f32)).ceil() as usize
            })
            .sum::<usize>();

        if total_lines >= h as usize {
          break;
        }

        if block_idx == 0 {
          break;
        }

        block_idx -= 1;
        line_idx = buff.block_size as usize;
      }

      line_idx = 0;
      is_prepended = true;
    }

    if init_block_idx < block_idx {
      block_idx = init_block_idx;
      line_idx = init_line_idx;
    }

    while (v.len() < h as usize) && (block_idx < buff.blocks.len())  {
      let block = buff.blocks.get(block_idx).unwrap();
      v.append(&mut self.chop_to_w(&block.data()[line_idx..], w as usize).to_vec());
      block_idx += 1;
      line_idx = 0;
    }

    if is_prepended && v.len() > h as usize {
      let (_, rem) = v.split_at(v.len() - h as usize);
      v = rem.to_vec();
    } else {
      v.truncate(h as usize);
    }

    return v;

    // end alter
  }

  fn chop_to_w(&self, data: &[String], w: usize) -> Vec<String> {
    data
      .iter()
      .flat_map(|line| {
        let mut chars: Vec<char> = line.chars().collect();
        if chars.is_empty() {
          chars.push('\n');
        }
        chars.chunks(w)
          .map(|ch| ch.into_iter().collect::<String>())
          .collect::<Vec<String>>()
      })
      .collect::<Vec<String>>()
  }
}

#[cfg(test)]
mod project_tests {
  use std::sync::{Arc, Mutex};
  use crate::buffer::Buff;
  use crate::project::Project;

  fn create_buff() -> Buff {
    Buff::new(
      [
        "1*1*1*1".to_string(),
        "2*2*2*2".to_string(),
        "3*3*3*3".to_string(),
        "4*4*4*4".to_string(),
        "5*5*5*5".to_string(),
      ].to_vec(),
      2
    )
  }

  #[test]
  fn render_when_block_contains_all_data() {
    let mut project = Project::new("Foo".to_string(), "Bar".to_string(), "Ban".to_string());
    let buff = create_buff();
    project.buff = Arc::new(Mutex::new(buff));
    project.offset2 = Arc::new(Mutex::new((1, 1)));

    assert_eq!(
      project.render(3, 2),
      [
        "4*4",
        "*4*",
      ]
    );
  }

  #[test]
  fn render_when_data_spans_current_and_next_blocks() {
    let mut project = Project::new("Foo".to_string(), "Bar".to_string(), "Ban".to_string());
    let buff = create_buff();
    project.buff = Arc::new(Mutex::new(buff));
    project.offset2 = Arc::new(Mutex::new((1, 1)));

    assert_eq!(
      project.render(3, 4),
      [
        "4*4",
        "*4*",
        "4",
        "5*5",
      ]
    );
  }


  #[test]
  fn render_when_data_spans_current_and_previous_blocks() {
    let mut project = Project::new("Foo".to_string(), "Bar".to_string(), "Ban".to_string());
    let buff = create_buff();
    project.buff = Arc::new(Mutex::new(buff));
    project.offset2 = Arc::new(Mutex::new((1, 1)));

    assert_eq!(
      project.render(5, 5),
      [
        "*3",
        "4*4*4",
        "*4",
        "5*5*5",
        "*5",
      ]
    );
  }
}