use std::io::BufReader;
use std::{io, thread};
use std::time::Duration;
use app::Project;
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

mod app;

static CONTENT: &str = r#"1.Check the "Autoloading and Reloading Constants" guide to learn more about how
2.Rails autoloads and reloads.
3.(called from <main> at /app/config/environment.rb:5)
4.[1m[36mLanguage Load (0.2ms)[0m  [1m[34mSELECT "languages".* FROM "languages" WHERE "languages"."hidden" = $1[0m  [["hidden", false]]
5.↳ config/initializers/locale.rb:2:in `map'
6.[1m[36mLanguage Load (0.3ms)[0m  [1m[34mSELECT "languages".* FROM "languages" WHERE "languages"."hidden" = $1[0m  [["hidden", false]]
7.↳ app/lib/i18n_extensions/hybrid_backend.rb:43:in `populate_translations'
8.DEPRECATION WARNING: Initialization autoloaded the constants ApplicationRecord, Hideable, Language, I18nExtensions, I18nExtensions::HybridBackend, and AnyLogin::ApplicationHelper."#;

fn draw_ui<B: Backend>(frame: &mut Frame<B>, app: &mut App) {
  let size = frame.size();
  let block = Block::default()
    .style(Style {
      bg: Some(Color::White),
      fg: Some(Color::Black),
      ..Style::default()
    })
    .title("Processes")
    .borders(Borders::ALL);

  let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Length(25),
      Constraint::Min(0),
    ])
    .split(frame.size());
  // Constraint::Percentage()
  // let data = std::fs::read_to_string("/home/vmaryn/projects/ruby/sport-news/log/development.log").unwrap();
  app.content = CONTENT.to_string();

  let paragraph = Paragraph::new(app.lines(block.inner(size).width).join("\n"))
    .block(block)
    .style(Style {
      fg: Some(Color::Black),
      ..Style::default()
    })
    .wrap(Wrap { trim: false });

  frame.render_widget(paragraph, chunks[1]);

  let sidebar_block = Block::default()
    .borders(Borders::ALL)
    .title("Projects");
  let items = app.projects.iter()
    .map(|p| ListItem::new(p.name.as_str()))
    .collect::<Vec<ListItem>>();

  let sidebar = List::new(items)
    .block(sidebar_block)
    .highlight_style(Style {
      fg: Some(Color::Blue),
      ..Style::default()
    });
  let mut state = ListState::default();
  if let Some(idx) = app.active_project {
    state.select(Some(idx as usize));
  }
  frame.render_stateful_widget(sidebar, chunks[0], &mut state);

}

fn run<T: Backend>(terminal: &mut Terminal<T>, app: &mut App) -> io::Result<()> {
  loop {
    terminal.draw(|f| {
      draw_ui(f, app);
    }).unwrap();

    let poll_duration = Duration::from_millis(200);

    if crossterm::event::poll(poll_duration)? {
      match crossterm::event::read()? {
        Event::Key(evt) => {
          match evt.code {
            KeyCode::Char(ch) => {
              match ch {
                  'q' => {
                    return Ok(());
                  },
                  _ => {}
              }
            },
            KeyCode::Up => {
              app.select_prev();
            },
            KeyCode::Down => {
              app.select_next();
            },
            _ => {}
          }
        },
        Event::Resize(x, y) => {},
        _ => {}
      }
    };

  }
  Ok(())
}
// dir
// executable
use std::io::BufRead;
fn main() {
  // use std::process::{Command, Stdio};
  // let mut cmd = Command::new("/bin/bash")
  // .arg("-c")
  // .arg("bundle exec rails s -p 3030")
  // .current_dir("/Users/vmaryn/telapp/tas")
  //   .stdout(Stdio::piped())
  //   .spawn()
  //   .expect("Failed to spawn child process");
  // let out = cmd.stdout.unwrap();
  // let mut reader = BufReader::new(out);
  // for line in reader.lines() {
  //   println!("Line {}", line.unwrap());
  // }

  // return;
  let mut out = std::io::stdout();

  enable_raw_mode().unwrap();

  execute!(out, EnterAlternateScreen, EnableMouseCapture).unwrap();
  let backend = CrosstermBackend::new(out);
  let mut terminal = Terminal::new(backend).unwrap();
  let mut  app = App::default();
  app.projects = vec![
    Project::new("Core Api".to_string(), "bundle exec rails s -p 3030".to_string(), "/Users/vmaryn/telapp/tas".to_string()),
    Project::new("Admin App".to_string(), "bundle exec rails s -p 3100".to_string(), "/Users/vmaryn/telapp/admin".to_string()),
  ];
  app.active_project = Some(1);

  run(&mut terminal, & mut app);

  disable_raw_mode().unwrap();
  execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    ).unwrap();
  // terminal.show_cursor().unwrap();
}

