use std::{vec};
use crate::project::{Project};

pub struct App {
  pub content: String,
  pub projects: Vec<Project>,
  pub active_project: Option<u8>,
  pub active_tab: AppTab,
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
}


impl Default for App {
  fn default() -> Self {
    App {
      content: String::new(),
      projects: vec![],
      active_project: None,
      active_tab: AppTab::Sidebar
    }
  }
}

pub enum AppTab {
  Sidebar,
  Console
}