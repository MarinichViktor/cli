use tui::{
  Frame,
  backend::{Backend},
  widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
  style::{Style, Color},
  layout::{Constraint, Direction, Layout, Rect},
};
use crate::app::{App, AppTab};

static SIDEBAR_ITEM_TEXT_ACTIVE_COLOR: Color = Color::Rgb(198, 198, 102);
static SIDEBAR_ITEM_TEXT_COLOR: Color = Color::DarkGray;
// static SIDEBAR_ITEM_BACKGROUND: Color = Color::DarkGray;
static SIDEBAR_ITEM_ACTIVE_BACKGROUND: Color = Color::Rgb(45, 44, 46);
static SIDEBAR_BACKGROUND_COLOR: Color = Color::Rgb(148, 111, 166);
static SIDEBAR_ACTIVE_BACKGROUND_COLOR: Color = Color::Rgb(148, 111, 166);
static SIDEBAR_BORDER_COLOR: Color = Color::DarkGray;
static SIDEBAR_ACTIVE_BORDER_COLOR: Color = Color::White;

static CONSOLE_BACKGROUND_COLOR: Color = Color::Rgb(92, 138, 138);
static CONSOLE_ACTIVE_BACKGROUND_COLOR: Color = Color::Rgb(92, 138, 138);
static CONSOLE_BORDER_COLOR: Color = Color::DarkGray;
static CONSOLE_ACTIVE_BORDER_COLOR: Color = Color::White;

static CONSOLE_TEXT_COLOR: Color = Color::White;
static CONSOLE_ACTIVE_TEXT_COLOR: Color = Color::White;


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
    .map(|project| {
      let mut name = project.name.clone();
      if project.status.lock().unwrap().is_running {
        name = format!("{} ◕", name);
      }
      ListItem::new(name)
          .style(
            Style::default().fg(SIDEBAR_ITEM_TEXT_COLOR)
          )
    })
    .collect::<Vec<ListItem>>();

  let mut block = Block::default()
    .borders(Borders::ALL)
    .border_style(Style::default().fg(SIDEBAR_BORDER_COLOR))
    .title("Projects");

  if let AppTab::Sidebar = app.active_tab {
    block = block.border_style(
      Style::default().fg(SIDEBAR_ACTIVE_BORDER_COLOR)
    );
  }

  let mut sidebar = List::new(items)
    .block(block)
    .highlight_style(
      Style::default()
          .fg(SIDEBAR_ITEM_TEXT_ACTIVE_COLOR)
          .bg(SIDEBAR_ITEM_ACTIVE_BACKGROUND)
    ).style(
      Style::default().bg(SIDEBAR_BACKGROUND_COLOR)
    );

  if let AppTab::Sidebar = app.active_tab {
    sidebar = sidebar.style(
      Style::default()
          .bg(SIDEBAR_ACTIVE_BACKGROUND_COLOR)
          .fg(SIDEBAR_ACTIVE_BORDER_COLOR)
    );
  }

  let mut state = ListState::default();
  state.select(Some(app.selected_project_index as usize));

  frame.render_stateful_widget(sidebar, area, &mut state);
}

// todo: review colors
fn render_console<B: Backend>(frame: &mut Frame<B>, area: Rect, app:  &mut App) {
  let mut block = Block::default()
    .style(
      Style::default()
        .fg(CONSOLE_BORDER_COLOR)
    )
    .title("Console")
    .borders(Borders::ALL);

  let mut paragraph_style = Style::default()
      .fg(CONSOLE_TEXT_COLOR)
      .bg(CONSOLE_BACKGROUND_COLOR);

  if let AppTab::Console = app.active_tab {
    block = block.border_style(Style::default().fg(CONSOLE_ACTIVE_BORDER_COLOR));
    paragraph_style = paragraph_style.bg(CONSOLE_ACTIVE_BACKGROUND_COLOR)
        .fg(CONSOLE_ACTIVE_TEXT_COLOR);
  }

  let text_area = block.inner(area);
  app.console_widget_size = text_area;

  // todo: to be refactored

  // let items: Vec<String> = vec![];//app.lines(text_area.width as usize, text_area.height as usize);
  let items: Vec<String> = app.lines(text_area.width as usize, text_area.height as usize);
  // println!("Ittems {:?}", items);

  let text = items.join("\n");

  let paragraph = Paragraph::new(text)
    .block(block)
    .style(
      paragraph_style
    )
    .wrap(Wrap { trim: false });

  frame.render_widget(paragraph, area);
}