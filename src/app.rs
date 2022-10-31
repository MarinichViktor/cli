use std::{vec};
use crossterm::event::{KeyCode, KeyEvent};
use crate::project::{Project};
use crate::result::{Result};

pub struct App {
  pub content: String,
  pub projects: Vec<Project>,
  pub active_project: Option<u8>,
  pub active_tab: AppTab,
  pub should_exit: bool
}

impl App {
  pub fn lines(&mut self, width: u16) -> Vec<String> {
    match self.selected_project() {
      Some(p) => {
        p.lines(width)
      }
      None => vec![String::from("Fallback text")]
    }
  }

  pub fn selected_project<'a>(&'a mut self) -> Option<&'a mut Project> {
    match self.active_project {
      Some(idx) => {
        Some(&mut self.projects[idx as usize])
      }
      None => None
    }
  }

  pub fn select_next(&mut self) {
    if self.projects.is_empty() {
      return;
    }

    if let Some(index) = self.active_project {
        self.active_project = Some((index + 1).min((self.projects.len() - 1) as u8));
    } else {
      self.active_project = Some(0);
    }
  }

  pub fn select_prev(&mut self) {
    if self.projects.is_empty() {
      return;
    }

    if let Some(index) = self.active_project {
        self.active_project = Some(((index as i8) - 1).max(0) as u8);
    } else {
      self.active_project = Some(0);
    }
  }

  pub fn next_tab(&mut self) {
    self.active_tab = match self.active_tab {
      AppTab::Console => AppTab::Sidebar,
      AppTab::Sidebar => AppTab::Console
    }
  }

  pub fn on_tick(&mut self) {
  }

  pub fn on_key(&mut self, key_event: KeyEvent) -> Result<()> {
    match key_event.code {
      KeyCode::Char(ch) => {
        match ch {
          'q' => {
            for project in self.projects.iter_mut() {
              match &mut project.child {
                Some(ch) => {
                  match ch.try_wait() {
                    Ok(Some(_)) => {},
                    _ => { ch.kill()? },
                  }
                },
                _ => {}
              }
            }

            self.should_exit = true;
          },
          'r' => self.selected_project().unwrap().run()?,
          's' => { self.selected_project().unwrap().stop()?; }
          _ => {}
        }
      },
      KeyCode::Tab => self.next_tab(),
      KeyCode::Up => match self.active_tab {
        AppTab::Sidebar => self.select_prev(),
        AppTab::Console => {

        }
      },
      KeyCode::Down => match self.active_tab {
        AppTab::Sidebar => self.select_next(),
        AppTab::Console => {

        }
      },
      _ => {}
    };
    Ok(())
  }
}


impl Default for App {
  fn default() -> Self {
    App {
      projects: vec![],
      content: String::new(),
      active_project: None,
      active_tab: AppTab::Sidebar,
      should_exit: false
    }
  }
}

pub enum AppTab {
  Sidebar,
  Console
}