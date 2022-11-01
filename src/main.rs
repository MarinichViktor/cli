use std::time::{Duration, Instant};
use term::project::{Project};
use crossterm::execute;
use crossterm::event::{Event};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use tui::backend::{Backend, CrosstermBackend};
use tui::{Terminal};
use term::{ui, app::{App}, result::Result};

fn main() -> Result<()> {
  let mut out = std::io::stdout();

  enable_raw_mode()?;
  execute!(out, EnterAlternateScreen)?;

  let backend = CrosstermBackend::new(out);
  let mut terminal = Terminal::new(backend)?;
  terminal.hide_cursor()?;
  let mut app = App::default();

  app.projects = vec![
    Project::new("Docker Core Api".to_string(), "docker-compose up".to_string(), "/home/vmaryn/projects/go/sandbox".to_string()),
    Project::new("Angular app".to_string(), "ng serve".to_string(), "/home/vmaryn/projects/dotnet/echat/src/WebSpa".to_string()),
    Project::new("Admin App".to_string(), "bundle exec rails s -p 3100".to_string(), "/Users/vmaryn/telapp/admin".to_string()),
  ];

  run(&mut terminal, & mut app)?;

  disable_raw_mode()?;
  execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
  terminal.show_cursor()?;

  Ok(())
}

fn run<T: Backend>(terminal: &mut Terminal<T>, app: &mut App) -> Result<()> {
  let tick_interval = Duration::from_millis(100);
  let mut last_tick = Instant::now();

  loop {
    terminal.draw(|f| ui::render_ui(f, app))?;

    let poll_timeout = tick_interval.checked_sub(last_tick.elapsed())
      .unwrap_or(Duration::from_secs(0));

    if crossterm::event::poll(poll_timeout)? {
      match crossterm::event::read()? {
        Event::Key(evt) => app.on_key(evt)?,
        _ => {}
      }
    }

    if last_tick.elapsed() >= tick_interval {
      last_tick = Instant::now();
    }

    if app.should_exit {
      return Ok(());
    }
  }
}