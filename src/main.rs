use std::io::BufReader;
use std::{io, thread};
use std::time::Duration;
use app::{Project, AppTab};
use crossterm::execute;
use crossterm::event::{EnableMouseCapture, DisableMouseCapture, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::{Frame, Terminal};
use tui::text::Text;
use tui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use crate::app::App;
use term::{ui, app};
use std::io::BufRead;

fn main() {
  let mut out = std::io::stdout();

  enable_raw_mode().unwrap();

  execute!(out, EnterAlternateScreen, EnableMouseCapture).unwrap();
  let backend = CrosstermBackend::new(out);
  let mut terminal = Terminal::new(backend).unwrap();

  let mut  app = App::default();

  app.projects = vec![
    Project::new("Docker Core Api".to_string(), "docker-compose up".to_string(), "/home/vmaryn/projects/go/sandbox".to_string()),
    Project::new("Admin App".to_string(), "bundle exec rails s -p 3100".to_string(), "/Users/vmaryn/telapp/admin".to_string()),
  ];
  app.active_project = Some(0);

  run(&mut terminal, & mut app);

  disable_raw_mode().unwrap();
  execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    ).unwrap();
}

fn run<T: Backend>(terminal: &mut Terminal<T>, app: &mut App) -> io::Result<()> {
  loop {
    terminal.draw(|f| ui::render_ui(f, app)).unwrap();

    let poll_duration = Duration::from_millis(200);

    if crossterm::event::poll(poll_duration)? {
      match crossterm::event::read()? {
        Event::Key(evt) => {
          match evt.code {
            KeyCode::Char(ch) => {
              match ch {
                'q' => {
                  for project in app.projects.iter() {
                    match &project.child {
                      Some(ch) => {
                        ch.lock().unwrap().kill().unwrap();
                      }
                      _ => {}
                    }
                  }

                  return Ok(());
                },
                'r' => {
                  let project = app.selected_project().unwrap();
                  project.run().unwrap();
                },
                _ => {}
              }
            },
            KeyCode::Tab => {
              app.next_tab();
            }
            KeyCode::Up => {
              match app.active_tab {
                AppTab::Sidebar => {
                  app.select_prev();
                }
                AppTab::Console => {

                }
              }
            },
            KeyCode::Down => {
              match app.active_tab {
                AppTab::Sidebar => {
                  app.select_next();
                }
                AppTab::Console => {

                }
              }
            },
            _ => {}
          }
        },
        Event::Resize(x, y) => {},
        _ => {}
      }
    }
  }
  Ok(())
}