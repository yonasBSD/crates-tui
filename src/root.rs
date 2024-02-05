use std::sync::{
  atomic::{AtomicBool, Ordering},
  Arc, Mutex,
};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Flex, prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tui_input::backend::crossterm::EventHandler;

use crate::{
  action::Action,
  config,
  widgets::{crate_info::CrateInfo, crates_table::CratesTable},
};

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
  Picker,
  #[default]
  PickerSearchQueryEditing,
  PickerFilterEditing,
  Info,
  Error,
}

#[derive(Debug)]
pub struct Root {
  tx: UnboundedSender<Action>,
  page: u64,
  page_size: u64,
  mode: Mode,
  last_events: Vec<KeyEvent>,
  loading_status: Arc<AtomicBool>,
  search: String,
  filter: String,
  filtered_crates: Vec<crates_io_api::Crate>,
  crates: Arc<Mutex<Vec<crates_io_api::Crate>>>,
  crate_info: Arc<Mutex<Option<crates_io_api::Crate>>>,
  total_num_crates: Option<u64>,
  input: tui_input::Input,
  show_crate_info: bool,
  error: Option<String>,
  table_state: TableState,
  scrollbar_state: ScrollbarState,
}

impl Root {
  pub fn new(tx: UnboundedSender<Action>) -> Self {
    Self {
      tx,
      page: 1,
      page_size: 25,
      mode: Mode::default(),
      last_events: Default::default(),
      loading_status: Default::default(),
      search: Default::default(),
      filter: Default::default(),
      filtered_crates: Default::default(),
      crates: Default::default(),
      crate_info: Default::default(),
      total_num_crates: Default::default(),
      table_state: Default::default(),
      scrollbar_state: Default::default(),
      input: Default::default(),
      show_crate_info: Default::default(),
      error: Default::default(),
    }
  }

  pub fn next(&mut self) {
    if self.filtered_crates.len() == 0 {
      self.table_state.select(None)
    } else {
      // wrapping behavior
      let i = match self.table_state.selected() {
        Some(i) => {
          if i >= self.filtered_crates.len().saturating_sub(1) {
            0
          } else {
            i + 1
          }
        },
        None => 0,
      };
      self.table_state.select(Some(i));
      self.scrollbar_state = self.scrollbar_state.position(i);
    }
  }

  pub fn previous(&mut self) {
    if self.filtered_crates.len() == 0 {
      self.table_state.select(None)
    } else {
      // wrapping behavior
      let i = match self.table_state.selected() {
        Some(i) => {
          if i == 0 {
            self.filtered_crates.len().saturating_sub(1)
          } else {
            i.saturating_sub(1)
          }
        },
        None => 0,
      };
      self.table_state.select(Some(i));
      self.scrollbar_state = self.scrollbar_state.position(i);
    }
  }

  pub fn top(&mut self) {
    if self.filtered_crates.len() == 0 {
      self.table_state.select(None)
    } else {
      self.table_state.select(Some(0));
      self.scrollbar_state = self.scrollbar_state.position(0);
    }
  }

  pub fn bottom(&mut self) {
    if self.filtered_crates.len() == 0 {
      self.table_state.select(None)
    } else {
      self.table_state.select(Some(self.filtered_crates.len() - 1));
      self.scrollbar_state = self.scrollbar_state.position(self.filtered_crates.len() - 1);
    }
  }

  fn increment_page(&mut self) {
    if let Some(n) = self.total_num_crates {
      let max_page_size = (n / self.page_size) + 1;
      if self.page < max_page_size {
        self.page = self.page.saturating_add(1).min(max_page_size);
        self.reload_data();
      }
    }
  }

  fn decrement_page(&mut self) {
    let min_page_size = 1;
    if self.page > min_page_size {
      self.page = self.page.saturating_sub(1).max(min_page_size);
      self.reload_data();
    }
  }

  fn reload_data(&mut self) {
    self.table_state.select(None);
    *self.crate_info.lock().unwrap() = None;
    let crates = self.crates.clone();
    let search = self.search.clone();
    let loading_status = self.loading_status.clone();
    let tx = self.tx.clone();
    let page = self.page.clamp(1, u64::MAX);
    let page_size = self.page_size;
    tokio::spawn(async move {
      loading_status.store(true, Ordering::SeqCst);
      let client =
        crates_io_api::AsyncClient::new("crates-tui (crates-tui@kdheepak.com)", std::time::Duration::from_millis(1000))
          .unwrap();
      let query = crates_io_api::CratesQueryBuilder::default()
        .search(&search)
        .page(page)
        .page_size(page_size)
        .sort(crates_io_api::Sort::Relevance)
        .build();
      match client.crates(query).await {
        Ok(page) => {
          let mut all_crates = vec![];
          for _crate in page.crates.iter() {
            all_crates.push(_crate.clone())
          }
          all_crates.sort_by(|a, b| b.downloads.cmp(&a.downloads));
          crates.lock().unwrap().drain(0..);
          *crates.lock().unwrap() = all_crates;
          if crates.lock().unwrap().len() > 0 {
            tx.send(Action::StoreTotalNumberOfCrates(page.meta.total)).unwrap_or_default();
            tx.send(Action::Tick).unwrap_or_default();
            tx.send(Action::MoveSelectionNext).unwrap_or_default();
          } else {
            tx.send(Action::Error(format!("Could not find any crates with query `{}`.", search))).unwrap_or_default();
          }
          loading_status.store(false, Ordering::SeqCst);
        },
        Err(err) => {
          tx.send(Action::Error(format!("API Client Error: {:?}", err))).unwrap_or_default();
          loading_status.store(false, Ordering::SeqCst);
        },
      }
    });
  }

  fn get_info(&mut self) {
    let name = if let Some(index) = self.table_state.selected() {
      if self.filtered_crates.len() > 0 {
        self.filtered_crates[index].name.clone()
      } else {
        return;
      }
    } else if self.filtered_crates.len() > 0 {
      self.table_state.select(Some(0));
      self.filtered_crates[0].name.clone()
    } else {
      return;
    };
    let tx = self.tx.clone();
    if !name.is_empty() {
      let crate_info = self.crate_info.clone();
      tokio::spawn(async move {
        let client = crates_io_api::AsyncClient::new(
          "crates-tui (crates-tui@kdheepak.com)",
          std::time::Duration::from_millis(1000),
        )
        .unwrap();
        match client.get_crate(&name).await {
          Ok(_crate_info) => *crate_info.lock().unwrap() = Some(_crate_info.crate_data),
          Err(_err) => tx.send(Action::Error("Unable to get crate information".into())).unwrap_or_default(),
        }
      });
    }
  }

  fn tick(&mut self) {
    self.last_events.drain(..);
    self.update_filtered_crates();
    self.update_scrollbar_state();
  }

  fn update_scrollbar_state(&mut self) {
    self.scrollbar_state = self.scrollbar_state.content_length(self.filtered_crates.len());
  }

  fn update_filtered_crates(&mut self) {
    let filter = self.filter.clone();
    let filter_words = filter.split_whitespace().collect::<Vec<_>>();
    self.filtered_crates = self
      .crates
      .lock()
      .unwrap()
      .iter()
      .filter(|c| {
        filter_words.iter().all(|word| {
          c.name.to_lowercase().contains(word)
            || c.description.clone().unwrap_or_default().to_lowercase().contains(word)
        })
      })
      .map(|c| c.clone())
      .collect();
  }

  fn cargo_add(&mut self) {
    let crate_info = self.crate_info.lock().unwrap().clone();
    let tx = self.tx.clone();
    if let Some(ci) = crate_info {
      tokio::spawn(async move {
        let output = tokio::process::Command::new("cargo").arg("add").arg(ci.name).output().await;
        match output {
          Ok(output) => {
            let data = String::from_utf8_lossy(&output.stdout).to_string();
            tx.send(Action::Error(data)).unwrap_or_default();
          },
          Err(err) => {
            let data = format!("ERROR: {:?}", err);
            tx.send(Action::Error(data)).unwrap_or_default();
          },
        }
      });
    }
  }
}

impl Root {
  pub fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::Tick => {
        self.tick();
      },
      Action::StoreTotalNumberOfCrates(n) => {
        self.total_num_crates = Some(n);
      },
      Action::MoveSelectionNext => {
        self.next();
        return Ok(Some(Action::GetInfo));
      },
      Action::MoveSelectionPrevious => {
        self.previous();
        return Ok(Some(Action::GetInfo));
      },
      Action::MoveSelectionTop => {
        self.top();
        return Ok(Some(Action::GetInfo));
      },
      Action::MoveSelectionBottom => {
        self.bottom();
        return Ok(Some(Action::GetInfo));
      },
      Action::EnterSearchQueryInsert => {
        self.mode = Mode::PickerSearchQueryEditing;
        self.input = self.input.clone().with_value(self.search.clone());
      },
      Action::EnterFilterInsert => {
        self.mode = Mode::PickerFilterEditing;
        self.input = self.input.clone().with_value(self.filter.clone());
      },
      Action::EnterNormal => {
        self.mode = Mode::Picker;
        if self.filtered_crates.len() > 0 && self.table_state.selected().is_none() {
          self.table_state.select(Some(0))
        }
      },
      Action::SubmitSearchQuery => {
        self.mode = Mode::Picker;
        self.filter.clear();
        return Ok(Some(Action::ReloadData));
      },
      Action::ReloadData => {
        self.reload_data();
      },
      Action::IncrementPage => {
        self.increment_page();
      },
      Action::DecrementPage => {
        self.decrement_page();
      },
      Action::ToggleShowCrateInfo => {
        self.show_crate_info = !self.show_crate_info;
        if self.crate_info.lock().unwrap().is_none() {
          self.show_crate_info = false;
        }
      },
      Action::GetInfo => {
        self.get_info();
      },
      Action::Error(err) => {
        self.error = Some(err);
        self.mode = Mode::Error;
      },
      Action::CloseError => {
        self.error = None;
        self.mode = Mode::PickerSearchQueryEditing;
      },
      Action::CargoAddCrate => self.cargo_add(),
      _ => {},
    }
    Ok(None)
  }

  pub fn handle_key_events(&mut self, key: KeyEvent, last_key_events: &[KeyEvent]) -> Result<Option<Action>> {
    let cmd = match self.mode {
      Mode::Error => {
        match key.code {
          KeyCode::Enter => Action::CloseError,
          KeyCode::Esc => Action::CloseError,
          _ => return Ok(None),
        }
      },
      Mode::Picker => {
        match key.code {
          KeyCode::Char('q') => Action::Quit,
          KeyCode::Char('?') => Action::EnterSearchQueryInsert,
          KeyCode::Char('/') => Action::EnterFilterInsert,
          KeyCode::Char('j') | KeyCode::Down => Action::MoveSelectionNext,
          KeyCode::Char('k') | KeyCode::Up => Action::MoveSelectionPrevious,
          KeyCode::Char('l') | KeyCode::Right => Action::IncrementPage,
          KeyCode::Char('h') | KeyCode::Left => Action::DecrementPage,
          KeyCode::Char('a') => Action::CargoAddCrate,
          KeyCode::Char('g') => {
            if let Some(KeyEvent { code: KeyCode::Char('g'), .. }) = last_key_events.last() {
              Action::MoveSelectionTop
            } else {
              return Ok(None);
            }
          },
          KeyCode::PageUp => Action::MoveSelectionTop,
          KeyCode::Char('G') | KeyCode::PageDown => Action::MoveSelectionBottom,
          KeyCode::Char('r') => Action::ReloadData,
          KeyCode::Home => Action::MoveSelectionTop,
          KeyCode::End => Action::MoveSelectionBottom,
          KeyCode::Esc => Action::Quit,
          KeyCode::Enter => Action::ToggleShowCrateInfo,
          _ => return Ok(None),
        }
      },
      Mode::PickerSearchQueryEditing => {
        match key.code {
          KeyCode::Esc => Action::EnterNormal,
          KeyCode::Enter => Action::SubmitSearchQuery,
          _ => {
            self.input.handle_event(&crossterm::event::Event::Key(key));
            self.search = self.input.value().into();
            return Ok(None);
          },
        }
      },
      Mode::PickerFilterEditing => {
        match key.code {
          KeyCode::Esc => Action::EnterNormal,
          KeyCode::Enter => Action::EnterNormal,
          _ => {
            self.input.handle_event(&crossterm::event::Event::Key(key));
            self.filter = self.input.value().into();
            self.table_state.select(None);
            Action::GetInfo
          },
        }
      },
      _ => return Ok(None),
    };
    Ok(Some(cmd))
  }

  pub fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
    f.render_widget(self.background(), area);

    let [table, input] =
      Layout::vertical([Constraint::Fill(0), Constraint::Length(3 + config::get().prompt_padding * 2)]).areas(area);

    let table = if self.show_crate_info {
      let [table, info] = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(table);

      if let Some(ci) = self.crate_info.lock().unwrap().clone() {
        f.render_widget(CrateInfo::new(ci), info);
      }
      table
    } else {
      table
    };

    f.render_stateful_widget(
      CratesTable::new(&self.filtered_crates, self.mode == Mode::Picker),
      table,
      &mut (&mut self.table_state, &mut self.scrollbar_state),
    );

    self.render_prompt(f, input);

    if let Some(err) = &self.error {
      self.render_error(f, area, err);
    }

    Ok(())
  }
}

impl Root {
  fn background(&self) -> impl Widget {
    Block::default().bg(config::get().background_color)
  }

  fn input_block(&self) -> impl Widget {
    let ncrates = self.total_num_crates.unwrap_or_default();
    let loading_status = if self.loading_status.load(Ordering::SeqCst) {
      format!("Loaded {} ...", ncrates)
    } else {
      format!(
        "{}/{}",
        self.table_state.selected().map_or(0, |n| (self.page.saturating_sub(1) * self.page_size) + n as u64 + 1),
        ncrates
      )
    };
    Block::default()
      .borders(Borders::ALL)
      .title(
        block::Title::from(Line::from(vec![
          "Query ".into(),
          "(Press ".into(),
          "?".bold(),
          " to search, ".into(),
          "/".bold(),
          " to filter, ".into(),
          "Enter".bold(),
          " to submit)".into(),
        ]))
        .alignment(Alignment::Left),
      )
      .title(loading_status)
      .title_alignment(Alignment::Right)
      .border_style(match self.mode {
        Mode::PickerSearchQueryEditing => Style::default().fg(config::get().search_query_outline_color),
        Mode::PickerFilterEditing => Style::default().fg(config::get().filter_query_outline_color),
        _ => Style::default().add_modifier(Modifier::DIM),
      })
  }

  fn input_text(&self, width: usize) -> impl Widget + '_ {
    let scroll = self.input.cursor().saturating_sub(width.saturating_sub(4));
    Paragraph::new(self.input.value()).scroll((0, scroll as u16))
  }

  fn render_prompt(&self, f: &mut Frame, area: Rect) {
    let vertical_margin = 1 + config::get().prompt_padding;
    let horizontal_margin = 1 + config::get().prompt_padding;
    f.render_widget(self.input_block(), area);
    f.render_widget(
      self.input_text(area.width as usize),
      area.inner(&Margin { horizontal: horizontal_margin, vertical: vertical_margin }),
    );
    if self.mode == Mode::PickerSearchQueryEditing || self.mode == Mode::PickerFilterEditing {
      f.set_cursor(
        (area.x + horizontal_margin + self.input.cursor() as u16).min(area.x + area.width.saturating_sub(2)),
        area.y + vertical_margin,
      )
    }
  }

  fn render_error(&self, f: &mut Frame<'_>, area: Rect, err: &str) {
    let [center] = Layout::vertical([Constraint::Percentage(50)]).flex(Flex::Center).areas(area);
    let [center] = Layout::horizontal([Constraint::Percentage(50)]).flex(Flex::Center).areas(center);
    f.render_widget(Clear, center);
    f.render_widget(
      Paragraph::new(err)
        .block(Block::bordered().title(block::Title::from("Error")).title(
          block::Title::from("Press `ESC` to exit").position(block::Position::Bottom).alignment(Alignment::Right),
        ))
        .wrap(Wrap { trim: true }),
      center,
    );
  }
}
