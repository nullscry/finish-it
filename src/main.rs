use chrono::prelude::*;
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use serde::{Deserialize, Serialize};
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
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table,
        TableState, Tabs,
    },
    Terminal,
};
use tui_textarea::{Input, Key, TextArea};
mod add;
use add::{activate, inactivate, initialize_title};

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

#[derive(Copy, Clone, Debug, PartialEq)]
enum MenuItem {
    Home,
    Instances,
    Add,
}

enum ActiveBlock {
    EventBlock,
    InstanceBlock,
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

    let mut textarea = [
        TextArea::default(),
        TextArea::default(),
        TextArea::default(),
        TextArea::default(),
        TextArea::default(),
        TextArea::default(),
        TextArea::default(),
        TextArea::default(),
    ];

    let titles = [
        "Event Name",
        "Event Type",
        "Instance Name",
        "Is Recurring?",
        "Is Finished?",
        "% Completed",
        "# Completed",
        "Remaining Days",
    ];

    let layout_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref());

    let mut which: usize = 0;

    for (ta, title) in textarea.iter_mut().zip(titles) {
        initialize_title(ta, title);
    }

    activate(&mut textarea[0]);
    for ta in textarea.iter_mut().skip(1) {
        inactivate(ta);
    }
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
                    let (left, right) = render_events(&event_list_state, &conn, &active_block);
                    rect.render_stateful_widget(left, event_chunks[0], &mut event_list_state);
                    rect.render_stateful_widget(right, event_chunks[1], &mut instance_list_state);
                }
                MenuItem::Add => {
                    let add_chunks = layout_rows.split(chunks[1]);
                    let mut layout_cols = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [
                                Constraint::Percentage(25),
                                Constraint::Percentage(25),
                                Constraint::Percentage(25),
                                Constraint::Percentage(25),
                            ]
                            .as_ref(),
                        )
                        .split(add_chunks[0]);
                    let mut layout_cols_lower = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [
                                Constraint::Percentage(25),
                                Constraint::Percentage(25),
                                Constraint::Percentage(25),
                                Constraint::Percentage(25),
                            ]
                            .as_ref(),
                        )
                        .split(add_chunks[1]);

                    layout_cols.append(&mut layout_cols_lower);

                    for (textarea, chunk) in textarea.iter().zip(layout_cols) {
                        let widget = textarea.widget();
                        rect.render_widget(widget, chunk);
                    }
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
                    inactivate(&mut textarea[which]);
                    which += 1;
                    if which > (textarea.len() - 1) {
                        which -= 1;
                        // TODO popup here
                    } else {
                        activate(&mut textarea[which]);
                    }
                }
                Input { key: Key::Esc, .. } => {
                    inactivate(&mut textarea[which]);
                    which = which.saturating_sub(1);
                    activate(&mut textarea[which]);
                }
                input => {
                    textarea[which].input(input);
                    // TODO Check inputs
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
    home
}

fn render_events<'a>(
    event_list_state: &ListState,
    conn: &Connection,
    active_block: &ActiveBlock,
) -> (List<'a>, Table<'a>) {
    let (list_highlight, table_highlight) = match active_block {
        ActiveBlock::EventBlock => (Color::Red, Color::Yellow),
        ActiveBlock::InstanceBlock => (Color::Yellow, Color::Red),
    };
    let events = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("events")
        .border_type(BorderType::Plain);

    let event_list = read_events_from_db(conn).expect("can fetch EventItem list");

    let items: Vec<_> = event_list
        .iter()
        .map(|instance| {
            ListItem::new(Spans::from(vec![Span::styled(
                instance.name.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let selected_event = event_list
        .get(
            event_list_state
                .selected()
                .expect("there is always a selected EventItem"),
        )
        .expect("exists")
        .clone();

    let list = List::new(items).block(events).highlight_style(
        Style::default()
            .bg(list_highlight)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let instance_list =
        read_instances_from_db(conn, &selected_event.name).expect("can fetch EventItem list");

    let mut rows: Vec<Row<'a>> = Vec::new();
    for instance in instance_list {
        rows.push(Row::new(vec![
            Cell::from(Span::raw(instance.instanceid.to_string())),
            Cell::from(Span::raw(instance.name.to_string())),
            Cell::from(Span::raw(instance.eventtype.to_string())),
            Cell::from(Span::raw(instance.isrecurring.to_string())),
            Cell::from(Span::raw(instance.isfinished.to_string())),
            Cell::from(Span::raw(instance.percentage.to_string())),
            Cell::from(Span::raw(instance.timesfinished.to_string())),
            Cell::from(Span::raw(instance.daylimit.to_string())),
            Cell::from(Span::raw(instance.created.to_string())),
        ]));
    }

    let instance_detail = Table::new(rows)
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
        ])
        .highlight_style(
            Style::default()
                .bg(table_highlight)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    // let selected_instance = instance_list
    //     .get(
    //         instance_list_state
    //             .selected()
    //             .expect("there is always a selected EventItem"),
    //     )
    //     .expect("exists")
    //     .clone();

    (list, instance_detail)
}

fn read_events_from_db(conn: &Connection) -> Result<Vec<EventItem>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM events")?;
    let event_iter = stmt.query_map([], |row| {
        Ok(EventItem {
            name: row.get(0)?,
            eventgroup: row.get(1)?,
            created: row.get(2)?,
        })
    })?;

    let mut events = Vec::new();
    for event in event_iter {
        events.push(event?);
    }

    Ok(events)
}

fn read_instances_count_from_db(
    conn: &Connection,
    selected_event: &str,
) -> Result<usize, rusqlite::Error> {
    let mut stmt = conn.prepare(
        format!(
            "SELECT COUNT(*) FROM instances WHERE eventtype = \"{}\"",
            selected_event
        )
        .as_str(),
    )?;
    let mut rows = (stmt.query([]))?;

    let row = rows.next().unwrap().expect("Invalid Event");
    let instance_count: usize = row.get(0).expect("Invalid Event");

    Ok(instance_count)
}

fn read_instances_from_db(
    conn: &Connection,
    event_name: &str,
) -> Result<Vec<InstanceItem>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        format!(
            "SELECT * FROM instances WHERE eventtype = \"{}\"",
            event_name
        )
        .as_str(),
    )?;
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
