use chrono::prelude::*;
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use serde::{Deserialize, Serialize};
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;

use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, ListState, Paragraph, TableState, Tabs},
    Terminal,
};
use tui_textarea::{Input, Key};
mod add;
use add::get_text_areas;

mod db;
use db::*;

mod render;
use render::*;

use rusqlite::Result;

pub enum ActiveBlock {
    EventBlock,
    InstanceBlock,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventItem {
    name: String,
    created: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstanceItem {
    instanceid: usize,
    name: String,
    eventtype: String,
    isrecurring: u8,
    isfinished: u8,
    percentage: f32,
    timesfinished: usize,
    daylimit: usize,
    // lastfinished: DateTime<Utc>,
    created: DateTime<Utc>,
}

impl InstanceItem {
    pub fn get_dot_vec(&self) -> String {
        let p = self.percentage as usize;
        let p = p / 5;
        let r = 20 - p;

        let filled = "█".repeat(p);
        let remaining = "░".repeat(r);
        [filled, remaining].concat()
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum MenuItem {
    Home,
    Instances,
    Add,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::Instances => 1,
            MenuItem::Add => 2,
        }
    }
}

// fn validate_num<T, E>(text_area: &mut TextArea) -> bool
// where
//     T: FromStr<Err = E>,
//     E: std::fmt::Display,
// {
//     let title = text_area.block().unwrap().title(title)
//     if let Err(err) = text_area.lines()[0].parse::<T>() {
//         text_area.set_style(Style::default().fg(Color::LightRed));
//         text_area.set_block(
//             Block::default()
//                 .borders(Borders::ALL)
//                 .title(format!("ERROR: {}", err)),
//         );
//         false
//     } else {
//         text_area.set_style(Style::default().fg(Color::LightGreen));
//         text_area.set_block(Block::default().borders(Borders::ALL).title("OK"));
//         true
//     }
// }

// fn reset_text_area(text_area: &mut TextArea) {

//     text_area.set_style(Style::default().fg(Color::LightRed));
//     text_area.set_block(
//         Block::default()
//             .borders(Borders::ALL)
//             .title(format!("ERROR: {}", err)),

//     text_area.set_style(Style::default().fg(Color::LightGreen));
//     text_area.set_block(Block::default().borders(Borders::ALL).title("OK"));

// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = get_db_connection();
    enable_raw_mode().expect("can run in raw mode");

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(100);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate && tx.send(Event::Tick).is_ok() {
                last_tick = Instant::now();
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let menu_titles = vec!["Home", "Events", "Add", "Delete", "Quit"];
    let mut active_menu_item = MenuItem::Home;
    let mut event_list_state = ListState::default();
    event_list_state.select(Some(0));

    let mut instance_list_state = TableState::default();
    instance_list_state.select(Some(0));

    let mut active_block = ActiveBlock::EventBlock;
    let mut instance_count = 0;

    let events = read_events_from_db(&conn).expect("can fetch EventItem list");

    let mut text_areas = get_text_areas();

    // let layout_rows = Layout::default()
    //     .direction(Direction::Vertical)
    //     .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref());

    let mut which: usize = 0;

    loop {
        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            let copyright = Paragraph::new("Hit Enter to enter Edit mode. Edit Progress with <-, -> and Enter again to confirm. Alt+W to delete selected instance.")
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("Modifying")
                        .border_type(BorderType::Plain),
                );

            let menu = menu_titles
                .iter()
                .map(|t| {
                    let (first, rest) = t.split_at(1);
                    Spans::from(vec![
                        Span::styled(
                            first,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::UNDERLINED),
                        ),
                        Span::styled(rest, Style::default().fg(Color::White)),
                    ])
                })
                .collect();

            let tabs = Tabs::new(menu)
                .select(active_menu_item.into())
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(Span::raw("|"));

            rect.render_widget(tabs, chunks[0]);

            match active_menu_item {
                MenuItem::Home => rect.render_widget(render_home(), chunks[1]),
                MenuItem::Instances => {
                    let event_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(chunks[1]);
                    let (left, right) = render_events(&event_list_state, &conn, &active_block);
                    rect.render_stateful_widget(left, event_chunks[0], &mut event_list_state);
                    rect.render_stateful_widget(right, event_chunks[1], &mut instance_list_state);
                }
                MenuItem::Add => {
                    // let add_chunks = layout_rows.split(chunks[1]);
                    let cols = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [
                                Constraint::Percentage(50),
                                Constraint::Percentage(50),
                            ]
                            .as_ref(),
                        )
                        .split(chunks[1]);
                    let layout_cols = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(
                            [
                                Constraint::Percentage(15),
                                Constraint::Percentage(15),
                                Constraint::Percentage(14),
                                Constraint::Percentage(14),
                                Constraint::Percentage(14),
                                Constraint::Percentage(14),
                                Constraint::Percentage(14),
                            ]
                            .as_ref(),
                        )
                        .split(cols[0]);
                    // let mut layout_cols_lower = Layout::default()
                    //     .direction(Direction::Horizontal)
                    //     .constraints(
                    //         [
                    //             Constraint::Percentage(25),
                    //             Constraint::Percentage(25),
                    //             Constraint::Percentage(25),
                    //             Constraint::Percentage(25),
                    //         ]
                    //         .as_ref(),
                    //     )
                    //     .split(add_chunks[1]);

                    // layout_cols.append(&mut layout_cols_lower);

                    for (ta, chunk) in text_areas.iter().zip(layout_cols) {
                        let widget = ta.text_area.widget();
                        rect.render_widget(widget, chunk);
                    }

                    let home = Paragraph::new(vec![
                        Spans::from(vec![Span::raw("")]),
                        Spans::from(vec![Span::raw("Welcome")]),
                        Spans::from(vec![Span::raw("")]),
                        Spans::from(vec![Span::raw("to")]),
                        Spans::from(vec![Span::raw("")]),
                        Spans::from(vec![Span::styled(
                            "EventItem-CLI",
                            Style::default().fg(Color::LightBlue),
                        )]),
                        Spans::from(vec![Span::raw("")]),
                        Spans::from(vec![Span::raw("Press 'Alt+e' to access events, 'Alt+a' to add instances and 'Alt+u' to update and 'Alt+d' to delete the currently selected EventItem.")]),
                    ])
                    .alignment(Alignment::Center)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .style(Style::default().fg(Color::White))
                            .title("Home")
                            .border_type(BorderType::Plain),
                    );
                    rect.render_widget(home, cols[1]);
                }
            }
            rect.render_widget(copyright, chunks[2]);
        })?;

        if active_menu_item == MenuItem::Add {
            match crossterm::event::read()?.into() {
                Input {
                    key: Key::Char('h'),
                    alt: true,
                    ..
                } => active_menu_item = MenuItem::Home,
                Input {
                    key: Key::Char('e'),
                    alt: true,
                    ..
                } => active_menu_item = MenuItem::Instances,
                Input {
                    key: Key::Char('q'),
                    alt: true,
                    ..
                } => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }
                Input {
                    key: Key::Enter, ..
                } => {
                    text_areas[which].inactivate();
                    which += 1;
                    if which > (text_areas.len() - 1) {
                        which -= 1;
                        insert_into_db(&conn, &text_areas)?;
                        // TODO popup here
                    } else {
                        text_areas[which].activate();
                    }
                }
                Input { key: Key::Esc, .. } => {
                    text_areas[which].inactivate();
                    which = which.saturating_sub(1);
                    text_areas[which].activate();
                }
                input => {
                    if text_areas[which].text_area.input(input) {
                        // is_valid = validate(&mut text_areas[which], which);
                    }
                }
            }
        } else {
            match rx.recv()? {
                Event::Input(event) => match event {
                    KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::ALT,
                        ..
                    } => {
                        disable_raw_mode()?;
                        terminal.show_cursor()?;
                        break;
                    }
                    KeyEvent {
                        code: KeyCode::Char('h'),
                        modifiers: KeyModifiers::ALT,
                        ..
                    } => active_menu_item = MenuItem::Home,
                    KeyEvent {
                        code: KeyCode::Char('e'),
                        modifiers: KeyModifiers::ALT,
                        ..
                    } => active_menu_item = MenuItem::Instances,
                    KeyEvent {
                        code: KeyCode::Char('a'),
                        modifiers: KeyModifiers::ALT,
                        ..
                    } => active_menu_item = MenuItem::Add,
                    // KeyCode::Char('a') => {
                    //     add_random_pet_to_db().expect("can add new random EventItem");
                    // }
                    // KeyCode::Char('d') => {
                    //     remove_pet_at_index(&mut event_list_state).expect("can remove EventItem");
                    // }
                    KeyEvent {
                        code: KeyCode::Down,
                        ..
                    } => match active_block {
                        ActiveBlock::EventBlock => {
                            if let Some(selected) = event_list_state.selected() {
                                let amount_events = read_events_from_db(&conn)
                                    .expect("can fetch EventItem list")
                                    .len();
                                if selected >= amount_events - 1 {
                                    event_list_state.select(Some(0));
                                } else {
                                    event_list_state.select(Some(selected + 1));
                                }
                            }
                        }
                        ActiveBlock::InstanceBlock => {
                            if let Some(selected) = instance_list_state.selected() {
                                if selected >= instance_count - 1 {
                                    instance_list_state.select(Some(0));
                                } else {
                                    instance_list_state.select(Some(selected + 1));
                                }
                            }
                        }
                    },
                    KeyEvent {
                        code: KeyCode::Up, ..
                    } => match active_block {
                        ActiveBlock::EventBlock => {
                            if let Some(selected) = event_list_state.selected() {
                                let amount_events = read_events_from_db(&conn)
                                    .expect("can fetch EventItem list")
                                    .len();
                                if selected > 0 {
                                    event_list_state.select(Some(selected - 1));
                                } else {
                                    event_list_state.select(Some(amount_events - 1));
                                }
                            }
                        }
                        ActiveBlock::InstanceBlock => {
                            if let Some(selected) = instance_list_state.selected() {
                                if selected > 0 {
                                    instance_list_state.select(Some(selected - 1));
                                } else {
                                    instance_list_state.select(Some(instance_count - 1));
                                }
                            }
                        }
                    },
                    KeyEvent {
                        code: KeyCode::Right,
                        ..
                    } => {
                        instance_count = read_instances_count_from_db(
                            &conn,
                            &events
                                .get(event_list_state.selected().unwrap())
                                .expect("Event list state error")
                                .name,
                        )
                        .expect("Error in counting instances from DB of selected event");

                        if instance_count > 0 {
                            active_block = ActiveBlock::InstanceBlock;
                            instance_list_state.select(Some(0));
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Left,
                        ..
                    } => {
                        active_block = ActiveBlock::EventBlock;
                    }
                    _ => {}
                },
                Event::Tick => {}
            }
        }
    }

    Ok(())
}

// fn add_random_pet_to_db() -> Result<Vec<EventItem>, Error> {
//     let mut rng = rand::thread_rng();
//     let db_content = fs::read_to_string(DB_PATH)?;
//     let mut parsed: Vec<EventItem> = serde_json::from_str(&db_content)?;
//     let catsdogs = match rng.gen_range(0, 1) {
//         0 => "cats",
//         _ => "dogs",
//     };

//     let random_pet = EventItem {
//         id: rng.gen_range(0, 9999999),
//         name: rng.sample_iter(Alphanumeric).take(10).collect(),
//         category: catsdogs.to_owned(),
//         age: rng.gen_range(1, 15),
//         created_at: Utc::now(),
//     };

//     parsed.push(random_pet);
//     fs::write(DB_PATH, &serde_json::to_vec(&parsed)?)?;
//     Ok(parsed)
// }

// fn remove_pet_at_index(event_list_state: &mut ListState) -> Result<(), Error> {
//     if let Some(selected) = event_list_state.selected() {
//         let db_content = fs::read_to_string(DB_PATH)?;
//         let mut parsed: Vec<EventItem> = serde_json::from_str(&db_content)?;
//         parsed.remove(selected);
//         fs::write(DB_PATH, &serde_json::to_vec(&parsed)?)?;
//         let amount_events = read_db().expect("can fetch EventItem list").len();
//         if selected > 0 {
//             event_list_state.select(Some(selected - 1));
//         } else {
//             event_list_state.select(Some(0));
//         }
//     }
//     Ok(())
// }
