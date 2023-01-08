use chrono::prelude::*;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use rand::{distributions::Alphanumeric, prelude::*};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
    },
    Terminal,
};

use rusqlite::NO_PARAMS;
use rusqlite::{Connection, Result};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct EventItem {
    name: String,
    eventgroup: String,
    created: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct InstanceItem {
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

#[derive(Copy, Clone, Debug)]
enum MenuItem {
    Home,
    Instances,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::Instances => 1,
        }
    }
}

fn get_db_connection() -> Connection {
    let db_path = path::Path::new("var/fit.db");

    if !path::Path::exists(db_path) {
        panic!(
            "Problem opening the file: {0:?}\nExecute \"fitdb create\" to initialize database at {0:?}",
            db_path
        );
    }

    Connection::open(db_path).unwrap()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = get_db_connection();
    enable_raw_mode().expect("can run in raw mode");

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
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

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let menu_titles = vec!["Home", "Events", "Add", "Delete", "Quit"];
    let mut active_menu_item = MenuItem::Home;
    let mut instance_list_state = ListState::default();
    instance_list_state.select(Some(0));

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

            let copyright = Paragraph::new("Finish-it 2023 - all rights reserved")
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("Copyright")
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
                    let (left, right) = render_events(&instance_list_state, &conn);
                    rect.render_stateful_widget(left, event_chunks[0], &mut instance_list_state);
                    rect.render_widget(right, event_chunks[1]);
                }
            }
            rect.render_widget(copyright, chunks[2]);
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char('h') => active_menu_item = MenuItem::Home,
                KeyCode::Char('e') => active_menu_item = MenuItem::Instances,
                // KeyCode::Char('a') => {
                //     add_random_pet_to_db().expect("can add new random EventItem");
                // }
                // KeyCode::Char('d') => {
                //     remove_pet_at_index(&mut instance_list_state).expect("can remove EventItem");
                // }
                KeyCode::Down => {
                    if let Some(selected) = instance_list_state.selected() {
                        let amount_events = read_db(&conn).expect("can fetch EventItem list").len();
                        if selected >= amount_events - 1 {
                            instance_list_state.select(Some(0));
                        } else {
                            instance_list_state.select(Some(selected + 1));
                        }
                    }
                }
                KeyCode::Up => {
                    if let Some(selected) = instance_list_state.selected() {
                        let amount_events = read_db(&conn).expect("can fetch EventItem list").len();
                        if selected > 0 {
                            instance_list_state.select(Some(selected - 1));
                        } else {
                            instance_list_state.select(Some(amount_events - 1));
                        }
                    }
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}

fn render_home<'a>() -> Paragraph<'a> {
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
        Spans::from(vec![Span::raw("Press 'e' to access events, 'a' to add random new events and 'd' to delete the currently selected EventItem.")]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Plain),
    );
    home
}

fn render_events<'a>(instance_list_state: &ListState, conn: &Connection) -> (List<'a>, Table<'a>) {
    let events = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("events")
        .border_type(BorderType::Plain);

    let event_list = read_db(conn).expect("can fetch EventItem list");
    let items: Vec<_> = event_list
        .iter()
        .map(|instance| {
            ListItem::new(Spans::from(vec![Span::styled(
                instance.name.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let selected_instance = event_list
        .get(
            instance_list_state
                .selected()
                .expect("there is always a selected EventItem"),
        )
        .expect("exists")
        .clone();

    let list = List::new(items).block(events).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let instance_detail = Table::new(vec![Row::new(vec![
        Cell::from(Span::raw(selected_instance.instanceid.to_string())),
        Cell::from(Span::raw(selected_instance.name.to_string())),
        Cell::from(Span::raw(selected_instance.eventtype.to_string())),
        Cell::from(Span::raw(selected_instance.isrecurring.to_string())),
        Cell::from(Span::raw(selected_instance.isfinished.to_string())),
        Cell::from(Span::raw(selected_instance.percentage.to_string())),
        Cell::from(Span::raw(selected_instance.timesfinished.to_string())),
        Cell::from(Span::raw(selected_instance.daylimit.to_string())),
        // Cell::from(Span::raw(selected_instance.lastfinished.to_string())),
        Cell::from(Span::raw(selected_instance.created.to_string())),
    ])])
    .header(Row::new(vec![
        Cell::from(Span::styled(
            "ID",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Name",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Category",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Recur",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Completed",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Completed %",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Completed #",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Day Limit",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Created At",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Detail")
            .border_type(BorderType::Plain),
    )
    .widths(&[
        Constraint::Percentage(5),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(5),
        Constraint::Percentage(5),
        Constraint::Percentage(5),
        Constraint::Percentage(5),
        Constraint::Percentage(5),
        Constraint::Percentage(20),
    ]);

    (list, instance_detail)
}

fn read_db(conn: &Connection) -> Result<Vec<InstanceItem>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM instances")?;
    let instance_iter = stmt.query_map([], |row| {
        Ok(InstanceItem {
            instanceid: row.get(0)?,
            name: row.get(1)?,
            eventtype: row.get(2)?,
            isrecurring: row.get(3)?,
            isfinished: row.get(4)?,
            percentage: row.get(5)?,
            timesfinished: row.get(6)?,
            daylimit: row.get(7)?,
            // lastfinished: row.get(8)?,
            created: row.get(8)?,
        })
    })?;

    let mut instances = Vec::new();
    for instance in instance_iter {
        instances.push(instance?);
    }

    Ok(instances)
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

// fn remove_pet_at_index(instance_list_state: &mut ListState) -> Result<(), Error> {
//     if let Some(selected) = instance_list_state.selected() {
//         let db_content = fs::read_to_string(DB_PATH)?;
//         let mut parsed: Vec<EventItem> = serde_json::from_str(&db_content)?;
//         parsed.remove(selected);
//         fs::write(DB_PATH, &serde_json::to_vec(&parsed)?)?;
//         let amount_events = read_db().expect("can fetch EventItem list").len();
//         if selected > 0 {
//             instance_list_state.select(Some(selected - 1));
//         } else {
//             instance_list_state.select(Some(0));
//         }
//     }
//     Ok(())
// }
