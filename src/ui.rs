use tui::{
  Frame,
  backend::{Backend},
  widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
  style::{Style, Color},
  layout::{Constraint, Direction, Layout, Rect},
};
use crate::app::{App, AppTab};

pub fn render_ui<B: Backend>(frame: &mut Frame<B>, app: &mut App) {
  let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Length(25),
      Constraint::Min(0),
    ])
    .split(frame.size());

  render_sidebar(frame, chunks[0], app);
  render_console(frame, chunks[1], app);
}

fn render_sidebar<B: Backend>(frame: &mut Frame<B>, area: Rect, app:  &App) {
  let items = app.projects.iter()
    .map(|p| ListItem::new(p.name.as_str()))
    .collect::<Vec<ListItem>>();

  let mut block = Block::default()
    .borders(Borders::ALL)
    .title("Projects");

  if let AppTab::Sidebar = app.active_tab {
    block = block.border_style(Style::default().bg(Color::Green))
  }

  let sidebar = List::new(items)
    .block(block)
    .highlight_style(Style {
      fg: Some(Color::Blue),
      ..Style::default()
    });

  let mut state = ListState::default();
  if let Some(i) = app.active_project {
    state.select(Some(i as usize));
  }

  frame.render_stateful_widget(sidebar, area, &mut state);
}

fn render_console<B: Backend>(frame: &mut Frame<B>, area: Rect, app:  &mut App) {
  let mut block = Block::default()
    .style(
      Style::default()
        .bg(Color::White)
        .fg(Color::Black)
    )
    .title("Processes")
    .borders(Borders::ALL);

  if let AppTab::Console = app.active_tab {
    block = block.border_style(Style::default().bg(Color::Green))
  }

  let text = app.lines(block.inner(area).width).join("\n");
  let paragraph = Paragraph::new(text)
    .block(block)
    .style(Style {
      fg: Some(Color::Black),
      ..Style::default()
    })
    .wrap(Wrap { trim: false });

  frame.render_widget(paragraph, area);
}