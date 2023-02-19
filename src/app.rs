use std::{vec};
use crossterm::event::{KeyCode, KeyEvent};
use tui::layout::Rect;
use crate::project::{Project};
use crate::result::{Result};

pub struct App {
  pub content: String,
  pub projects: Vec<Project>,
  pub selected_project_index: u8,
  pub active_tab: AppTab,
  pub should_exit: bool,
  pub console_widget_size: Rect
}

impl App {
  pub fn lines(&mut self, w: usize, h: usize) -> Vec<String> {
    self.selected_project().render(w, h)
  }

  pub fn selected_project<'a>(&'a mut self) -> &'a mut Project {
    &mut self.projects[self.selected_project_index as usize]
  }

  pub fn select_next(&mut self) {
    if self.projects.is_empty() {
      return;
    }

    self.selected_project_index = ( self.selected_project_index + 1).min((self.projects.len() - 1) as u8);
  }

  pub fn select_prev(&mut self) {
    if self.projects.is_empty() {
      return;
    }

    self.selected_project_index = ((self.selected_project_index  as i8) - 1).max(0) as u8;
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
              if let Some(ch) = &mut project.child {
                match ch.try_wait() {
                  Ok(Some(_)) => {},
                  _ => ch.kill()?,
                }
              }
            }

            self.should_exit = true;
          },
          'r' => self.selected_project().run()?,
          's' => { self.selected_project().stop()?; }
          _ => {}
        }
      },
      KeyCode::Tab => self.next_tab(),
      KeyCode::Up => match self.active_tab {
        AppTab::Sidebar => self.select_prev(),
        AppTab::Console => {
          *self.selected_project().offset.lock().unwrap() += 1;
        }
      },
      KeyCode::Down => match self.active_tab {
        AppTab::Sidebar => self.select_next(),
        AppTab::Console => {
          // todo: to be refactored
          let mut offset = self.selected_project().offset.lock().unwrap();
          *offset = (*offset - 1).max(0);
          // let curr_offset = *self.selected_project().offset.lock().unwrap();
          // *self.selected_project().offset.lock().unwrap() = if curr_offset > 0 {
          //   curr_offset - 1
          // } else {
          //   0
          // };
        }
      },
      KeyCode::PageDown => {
        if let AppTab::Console = self.active_tab {
            let x = self.console_widget_size;
            let mut curr_offset = self.selected_project().offset.lock().unwrap();
            *curr_offset = (*curr_offset - x.height as i32).max(0);
        }
      },
      KeyCode::PageUp => {
        if let AppTab::Console = self.active_tab {
          let x = self.console_widget_size;
        let mut curr_offset = self.selected_project().offset.lock().unwrap();
        *curr_offset += x.height as i32;
        }
      }
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
      selected_project_index: 0,
      active_tab: AppTab::Sidebar,
      should_exit: false,
      console_widget_size: Rect::default()
    }
  }
}

pub enum AppTab {
  Sidebar,
  Console
}
