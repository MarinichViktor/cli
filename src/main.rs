use std::{io};
use std::time::{Duration, Instant};
use term::project::{Project};
use crossterm::execute;
use crossterm::event::{EnableMouseCapture, DisableMouseCapture, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use tui::backend::{Backend, CrosstermBackend};
use tui::{Terminal};
use term::{ui, app::{App, AppTab}};

fn main() {
  let mut out = std::io::stdout();

  enable_raw_mode().unwrap();

  execute!(out, EnterAlternateScreen, EnableMouseCapture ).unwrap();
  let backend = CrosstermBackend::new(out);
  let mut terminal = Terminal::new(backend).unwrap();

  let mut  app = App::default();

  app.projects = vec![
    Project::new("Docker Core Api".to_string(), "docker-compose up".to_string(), "/home/vmaryn/projects/go/sandbox".to_string(), app.sender.clone()),
    Project::new("Angular app".to_string(), "ng serve".to_string(), "/home/vmaryn/projects/dotnet/echat/src/WebSpa".to_string(),app.sender.clone()),
    Project::new("Admin App".to_string(), "bundle exec rails s -p 3100".to_string(), "/Users/vmaryn/telapp/admin".to_string(), app.sender.clone()),
  ];
  app.active_project = Some(0);

  run(&mut terminal, & mut app).unwrap();

  disable_raw_mode().unwrap();
  execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    ).unwrap();
}

fn run<T: Backend>(terminal: &mut Terminal<T>, app: &mut App) -> io::Result<()> {
  let tick_interval = Duration::from_millis(100);
  let mut last_tick = Instant::now();

  loop {
    if crossterm::event::poll(tick_interval.checked_sub(last_tick.elapsed()).unwrap_or(Duration::from_secs(0)))? {
      match crossterm::event::read()? {
        Event::Key(evt) => {
          match evt.code {
            KeyCode::Char(ch) => {
              match ch {
                'q' => {
                  for project in app.projects.iter_mut(){
                    match &mut project.child {
                      Some(ch) => {
                        ch.kill().unwrap();
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
        _ => {}
      }
    }

    if last_tick.elapsed().gt(&tick_interval)  {
      app.on_tick();
      terminal.draw(|f| ui::render_ui(f, app)).unwrap();

      last_tick = Instant::now();
    }
  }
}