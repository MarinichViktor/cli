use std::io::BufReader;
use std::{io, thread};
use std::time::Duration;
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

fn split_by_size<'a>(input: &'a str, len: u16) -> Vec<String> {
  input.lines()
    .flat_map(|line| {
      let mut curr = line;
      let mut sublines = vec![];

      while curr.len() > len as usize {
        let (a,b) = curr.split_at(len as usize);
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
fn split_by_sizeref<'a>(input: &'a str, len: u16) -> Vec<&'a str> {
  input.lines()
    .flat_map(|line| {
      let mut curr = line;
      let mut sublines = vec![];

      while curr.len() > len as usize {
        let (a,b) = curr.split_at(len as usize);
        sublines.push(a);
        curr = b;
      }

      if !curr.is_empty() {
        sublines.push(curr);
      }

      sublines
    })
    .collect()
}

fn draw_ui<B: Backend>(frame: &mut Frame<B>, app: &mut App) {
  let size = frame.size();
  let block = Block::default()
    .style(Style {
      bg: Some(Color::White),
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
    .wrap(Wrap { trim: false })
    .scroll((1000, 0));

  frame.render_widget(paragraph, chunks[1]);

  let sidebar_block = Block::default()
    .borders(Borders::ALL)
    .title("Projects");
  let sidebar = List::new(vec![ListItem::new("Frontend"), ListItem::new("Backend"), ListItem::new("Management")])
    .block(sidebar_block);
  let mut state = ListState::default();
  state.select(Some(1));
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
              if ch == 'q' {
                return Ok(());
              }
            }
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
use std::io::BufRead;
fn main() {
  use std::process::{Command, Stdio};
  let mut cmd = Command::new("/bin/bash")
  .arg("-c")
  .arg("bundle exec rails s -p 3030")
  .current_dir("/Users/vmaryn/telapp/tas")
    .stdout(Stdio::piped())
    .spawn()
    .expect("Failed to spawn child process");
  let out = cmd.stdout.unwrap();
  let mut reader = BufReader::new(out);
  for line in reader.lines() {
    println!("Line {}", line.unwrap());
  }


  return;
  let mut out = std::io::stdout();

  enable_raw_mode().unwrap();

  execute!(out, EnterAlternateScreen, EnableMouseCapture).unwrap();
  let backend = CrosstermBackend::new(out);
  let mut terminal = Terminal::new(backend).unwrap();
  let mut  app = App::default();

  run(&mut terminal, & mut app);


  disable_raw_mode().unwrap();
  execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    ).unwrap();
  // terminal.show_cursor().unwrap();
}

