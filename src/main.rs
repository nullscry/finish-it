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

use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Clear, ListState, Paragraph, TableState, Tabs},
    Terminal,
};

mod add;
use add::{
    get_add_err_text, get_add_ok_text, get_text_areas, validate_text_areas, TextAreaContainer,
};

mod db;
use db::*;

mod render;
use render::*;

use rusqlite::Result;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ActiveBlock {
    EventBlock,
    InstanceBlock,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ActivePopUp {
    Update,
    Delete,
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct EventItem {
    name: String,
    created: DateTime<Utc>,
}

impl EventItem {
    fn as_delete_paragraph(&self) -> Paragraph {
        let text = vec![
            Spans::from(
                vec![Span::raw(format!("Are you sure you want to DELETE Event {} and ALL INSTANCES belonging to this Event", self.name.to_owned()))],
            ),
            Spans::from(vec![Span::raw("Hit Enter to Confirm or Esc to cancel")]),
        ];

        let block = Paragraph::new(text)
            .style(Style::default().fg(Color::LightCyan))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .title("Modifying")
                    .border_type(BorderType::Plain),
            );
        block
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstanceItem {
    instanceid: usize,
    name: String,
    eventtype: String,
    isrecurring: u8,
    percentage: u8,
    timesfinished: usize,
    daylimit: usize,
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

    fn increment_one(&mut self) {
        match self.isrecurring {
            0 => {
                if self.percentage + 1 < 100 {
                    self.percentage += 1;
                } else {
                    self.percentage = 100;
                    self.timesfinished = 1;
                }
            }
            1 => {
                if self.percentage + 1 <= 100 {
                    self.percentage += 1;
                } else {
                    self.percentage = 1;
                    self.timesfinished += 1;
                }
            }
            _ => {}
        };
    }

    fn decrement_one(&mut self) {
        match self.isrecurring {
            0 => {
                self.percentage = self.percentage.saturating_sub(1);
                if self.percentage < 100 {
                    self.timesfinished = 0;
                }
            }
            1 => match self.timesfinished {
                0 => self.percentage = self.percentage.saturating_sub(1),
                _ => match self.percentage.checked_sub(1) {
                    Some(n) => self.percentage = n,
                    None => {
                        self.percentage = 100;
                        self.timesfinished -= 1;
                    }
                },
            },
            _ => {}
        };
    }

    fn finish_once(&mut self) {
        match self.isrecurring {
            0 => {
                self.percentage = 100;
                self.timesfinished = 1;
            }
            1 => {
                self.percentage = 0;
                self.timesfinished += 1;
            }
            _ => {}
        };
    }

    fn as_update_paragraph(&self) -> Paragraph {
        let text = vec![
            Spans::from(vec![Span::raw(self.eventtype.to_owned())]),
            Spans::from(vec![Span::raw(self.name.to_owned())]),
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![Span::raw(format!(
                "{} : {:.1}",
                self.get_dot_vec(),
                self.percentage
            ))]),
            Spans::from(vec![Span::raw(format!(
                "Is Recurring? = {}  Times Finished = {}",
                self.isrecurring, self.timesfinished
            ))]),
            Spans::from(vec![Span::raw(
                "Change Progress with <- and -> Arrow Keys.",
            )]),
            Spans::from(vec![Span::raw(
                "Press Enter to Update The Progress. Press Esc to Cancel.",
            )]),
        ];

        let block = Paragraph::new(text)
            .style(Style::default().fg(Color::LightCyan))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .title("Modifying")
                    .border_type(BorderType::Plain),
            );
        block
    }

    fn as_delete_paragraph(&self) -> Paragraph {
        let text = vec![
            Spans::from(vec![Span::raw("Are you sure you want to DELETE:")]),
            Spans::from(vec![Span::raw(self.name.to_owned())]),
            Spans::from(vec![Span::raw("Hit Enter to Confirm or Esc to cancel")]),
        ];

        let block = Paragraph::new(text)
            .style(Style::default().fg(Color::LightCyan))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .title("Modifying")
                    .border_type(BorderType::Plain),
            );
        block
    }
}

impl Default for InstanceItem {
    fn default() -> Self {
        InstanceItem {
            instanceid: 0,
            name: "".to_string(),
            eventtype: "".to_string(),
            isrecurring: 0,
            percentage: 0,
            timesfinished: 0,
            daylimit: 0,
            created: chrono::offset::Utc::now(),
        }
    }
}

// #[derive(Error, Debug)]
// pub enum Error {
//     #[error("error reading the DB file: {0}")]
//     ReadDBError(#[from] io::Error),
//     #[error("error parsing the DB file: {0}")]
//     ParseDBError(#[from] serde_json::Error),
// }

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
    let mut active_block = ActiveBlock::EventBlock;

    let mut event_list_state = ListState::default();
    event_list_state.select(None);

    let mut instance_list_state = TableState::default();
    instance_list_state.select(Some(0));

    let mut instance_count = 0;

    let mut events = read_events_from_db(&conn).expect("can fetch EventItem list");

    let mut text_areas = get_text_areas();

    let mut add_given_ok = false;

    let mut which: usize = 0;

    let mut active_popup = ActivePopUp::None;
    // let mut progress_amount: f64 = 0.0;
    let mut selected_instance = InstanceItem::default();
    let mut selected_event = EventItem::default();

    loop {
        terminal.draw(|rect| {
            // let selected_instance: &InstanceItem;
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
                    let (left, selected_instance_, selected_event_, right) = render_events(&event_list_state, &instance_list_state, &conn, &active_block);

                    rect.render_stateful_widget(left, event_chunks[0], &mut event_list_state);
                    rect.render_stateful_widget(right, event_chunks[1], &mut instance_list_state);
                    match (active_block, active_popup) {
                        (ActiveBlock::InstanceBlock, ActivePopUp::Update) => {
                            let block = selected_instance.as_update_paragraph();

                            let area = centered_rect(60, 20, size);
                            rect.render_widget(Clear, area);
                            rect.render_widget(block, area);
                        }

                        (ActiveBlock::InstanceBlock, ActivePopUp::Delete) => {
                            let block = selected_instance.as_delete_paragraph();
                            let area = centered_rect(60, 20, size);
                            rect.render_widget(Clear, area);
                            rect.render_widget(block, area);
                        }

                        (ActiveBlock::EventBlock, ActivePopUp::Delete) => {
                            let block = selected_event.as_delete_paragraph();
                            let area = centered_rect(60, 20, size);
                            rect.render_widget(Clear, area);
                            rect.render_widget(block, area);
                        }

                        (ActiveBlock::EventBlock, ActivePopUp::Update) => {}


                        (_, ActivePopUp::None) => {
                            selected_instance = selected_instance_;
                            selected_event = selected_event_;
                        }
                    }
                    // let block = Block::default().title("Popup").borders(Borders::ALL);
                    // let area = centered_rect(60, 20, size);
                    // // f.render_widget(Clear, size); //this clears out the background
                    // rect.render_widget(block, size);
                }
                MenuItem::Add => {
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
                                Constraint::Percentage(20),
                                Constraint::Percentage(20),
                                Constraint::Percentage(15),
                                Constraint::Percentage(15),
                                Constraint::Percentage(15),
                                Constraint::Percentage(15),
                            ]
                            .as_ref(),
                        )
                        .split(cols[0]);
                    for (ta, chunk) in text_areas.iter().zip(layout_cols) {
                        let widget = ta.text_area.widget();
                        rect.render_widget(widget, chunk);
                    }
                    let helper_text = if !add_given_ok {
                        get_add_err_text()
                    } else {
                        get_add_ok_text()
                    };
                    rect.render_widget(helper_text, cols[1]);


                    // let block = Block::default().title("Popup").borders(Borders::ALL);
                    // let area = centered_rect(60, 20, size);
                    // // f.render_widget(Clear, size); //this clears out the background
                    // rect.render_widget(block, size);
                }
            }
            rect.render_widget(copyright, chunks[2]);
        })?;

        match rx.recv()? {
            Event::Input(event) => match (event, active_menu_item, active_block, active_popup) {
                // Global Keys
                (
                    KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::ALT,
                        ..
                    },
                    _,
                    _,
                    _,
                ) => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }

                (
                    KeyEvent {
                        code: KeyCode::Char('h'),
                        modifiers: KeyModifiers::ALT,
                        ..
                    },
                    _,
                    _,
                    ActivePopUp::None,
                ) => active_menu_item = MenuItem::Home,

                (
                    KeyEvent {
                        code: KeyCode::Char('e'),
                        modifiers: KeyModifiers::ALT,
                        ..
                    },
                    _,
                    _,
                    ActivePopUp::None,
                ) => active_menu_item = MenuItem::Instances,

                (
                    KeyEvent {
                        code: KeyCode::Char('a'),
                        modifiers: KeyModifiers::ALT,
                        ..
                    },
                    _,
                    _,
                    ActivePopUp::None,
                ) => active_menu_item = MenuItem::Add,
                // KeyCode::Char('a') => {
                //     add_random_pet_to_db().expect("can add new random EventItem");
                // }
                // KeyCode::Char('d') => {
                //     remove_pet_at_index(&mut event_list_state).expect("can remove EventItem");
                // }

                // Instances - Event Block Keys
                (
                    KeyEvent {
                        code: KeyCode::Down,
                        ..
                    },
                    MenuItem::Instances,
                    ActiveBlock::EventBlock,
                    ActivePopUp::None,
                ) => {
                    match read_events_from_db(&conn) {
                        Ok(e) => {
                            if !e.is_empty() {
                                let selected = match event_list_state.selected() {
                                    Some(s) => s,
                                    None => 0,
                                };
                                if selected >= e.len() - 1 {
                                    event_list_state.select(Some(0));
                                } else {
                                    event_list_state.select(Some(selected + 1));
                                }
                            }
                        }
                        Err(_) => {}
                    }

                    // let amount_events = read_events_from_db(&conn)
                    //     .expect("can fetch EventItem list")
                    //     .len();
                    // if selected >= amount_events - 1 {
                    //     event_list_state.select(Some(0));
                    // } else {
                    //     event_list_state.select(Some(selected + 1));
                    // }
                }

                (
                    KeyEvent {
                        code: KeyCode::Up, ..
                    },
                    MenuItem::Instances,
                    ActiveBlock::EventBlock,
                    ActivePopUp::None,
                ) => {
                    // if let Some(selected) = event_list_state.selected() {
                    //     match read_events_from_db(&conn) {
                    //         Ok(e) => {
                    //             if e.len() > 0 {
                    //                 let amount_events = e.len();
                    //                 if selected > 0 {
                    //                     event_list_state.select(Some(selected - 1));
                    //                 } else {
                    //                     event_list_state.select(Some(amount_events - 1));
                    //                 }
                    //             }
                    //         }
                    //         Err(_) => {}
                    //     }

                    //     // let amount_events = read_events_from_db(&conn)
                    //     //     .expect("can fetch EventItem list")
                    //     //     .len();
                    //     // if selected > 0 {
                    //     //     event_list_state.select(Some(selected - 1));
                    //     // } else {
                    //     //     event_list_state.select(Some(amount_events - 1));
                    //     // }
                    // }
                    match read_events_from_db(&conn) {
                        Ok(e) => {
                            if !e.is_empty() {
                                let selected = match event_list_state.selected() {
                                    Some(s) => s,
                                    None => 0,
                                };
                                if selected > 0 {
                                    event_list_state.select(Some(selected - 1));
                                } else {
                                    event_list_state.select(Some(e.len() - 1));
                                }
                            }
                        }
                        Err(_) => {}
                    }
                }

                (
                    KeyEvent {
                        code: KeyCode::Right,
                        ..
                    },
                    MenuItem::Instances,
                    ActiveBlock::EventBlock,
                    ActivePopUp::None,
                ) => {
                    match read_events_from_db(&conn) {
                        Ok(e) => {
                            if !e.is_empty() {
                                match e.get(event_list_state.selected().unwrap_or(0)) {
                                    Some(sel_event) => {
                                        instance_count =
                                            read_instances_count_from_db(&conn, &sel_event.name)?;
                                        if instance_count > 0 {
                                            active_block = ActiveBlock::InstanceBlock;
                                            instance_list_state.select(Some(0));
                                        }
                                    }
                                    None => {}
                                }
                            }
                        }
                        Err(_) => {}
                    }

                    // instance_count = match read_instances_count_from_db(
                    //     &conn,
                    //     &events
                    //         .get(event_list_state.selected().unwrap_or(0))
                    //         .expect("Event list state error")
                    //         .name,
                    // ) {
                    //     Ok(n) => n,
                    //     Err(_) => 0,
                    // };

                    // if instance_count > 0 {
                    //     active_block = ActiveBlock::InstanceBlock;
                    //     instance_list_state.select(Some(0));
                    // }
                }

                (
                    KeyEvent {
                        code: KeyCode::Char('d'),
                        modifiers: KeyModifiers::ALT,
                        ..
                    },
                    MenuItem::Instances,
                    ActiveBlock::EventBlock,
                    ActivePopUp::None,
                ) => {
                    active_popup = ActivePopUp::Delete;
                }

                (
                    KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    },
                    MenuItem::Instances,
                    ActiveBlock::EventBlock,
                    ActivePopUp::Delete,
                ) => {
                    delete_event(&conn, &selected_event)?;
                    active_popup = ActivePopUp::None;
                    match read_events_from_db(&conn) {
                        Ok(e) => {
                            if e.is_empty() {
                                event_list_state.select(None);
                            } else if let Some(selected) = event_list_state.selected() {
                                if selected >= e.len() {
                                    event_list_state.select(Some(e.len() - 1));
                                }
                            }
                        }
                        Err(_) => {}
                    }
                }

                (
                    KeyEvent {
                        code: KeyCode::Esc, ..
                    },
                    MenuItem::Instances,
                    ActiveBlock::EventBlock,
                    ActivePopUp::Delete,
                ) => {}

                // Instances - Instance Block Keys
                (
                    KeyEvent {
                        code: KeyCode::Down,
                        ..
                    },
                    MenuItem::Instances,
                    ActiveBlock::InstanceBlock,
                    ActivePopUp::None,
                ) => {
                    if let Some(selected) = instance_list_state.selected() {
                        if instance_count > 0 {
                            if selected >= instance_count - 1 {
                                instance_list_state.select(Some(0));
                            } else {
                                instance_list_state.select(Some(selected + 1));
                            }
                        }
                    }
                }

                (
                    KeyEvent {
                        code: KeyCode::Up, ..
                    },
                    MenuItem::Instances,
                    ActiveBlock::InstanceBlock,
                    ActivePopUp::None,
                ) => {
                    if let Some(selected) = instance_list_state.selected() {
                        if instance_count > 0 {
                            if selected > 0 {
                                instance_list_state.select(Some(selected - 1));
                            } else {
                                instance_list_state.select(Some(instance_count - 1));
                            }
                        }
                    }
                }

                (
                    KeyEvent {
                        code: KeyCode::Left,
                        ..
                    },
                    MenuItem::Instances,
                    ActiveBlock::InstanceBlock,
                    ActivePopUp::None,
                ) => {
                    active_block = ActiveBlock::EventBlock;
                    let selected = match event_list_state.selected() {
                        Some(s) => s,
                        None => 0,
                    };
                    event_list_state.select(Some(selected));
                }

                (
                    KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    },
                    MenuItem::Instances,
                    ActiveBlock::InstanceBlock,
                    ActivePopUp::None,
                ) => {
                    active_popup = ActivePopUp::Update;
                }

                (
                    KeyEvent {
                        code: KeyCode::Char('d'),
                        modifiers: KeyModifiers::ALT,
                        ..
                    },
                    MenuItem::Instances,
                    ActiveBlock::InstanceBlock,
                    ActivePopUp::None,
                ) => {
                    active_popup = ActivePopUp::Delete;
                }

                // Update Popup
                (
                    KeyEvent {
                        code: KeyCode::Right,
                        ..
                    },
                    MenuItem::Instances,
                    ActiveBlock::InstanceBlock,
                    ActivePopUp::Update,
                ) => {
                    selected_instance.increment_one();
                }

                (
                    KeyEvent {
                        code: KeyCode::Left,
                        ..
                    },
                    MenuItem::Instances,
                    ActiveBlock::InstanceBlock,
                    ActivePopUp::Update,
                ) => {
                    selected_instance.decrement_one();
                }

                (
                    KeyEvent {
                        code: KeyCode::Tab, ..
                    },
                    MenuItem::Instances,
                    ActiveBlock::InstanceBlock,
                    ActivePopUp::Update,
                ) => {
                    selected_instance.finish_once();
                }

                (
                    KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    },
                    MenuItem::Instances,
                    ActiveBlock::InstanceBlock,
                    ActivePopUp::Update,
                ) => {
                    update_instance(&conn, &selected_instance)?;
                    active_popup = ActivePopUp::None;
                }

                // Delete Popup
                (
                    KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    },
                    MenuItem::Instances,
                    ActiveBlock::InstanceBlock,
                    ActivePopUp::Delete,
                ) => {
                    delete_instance(&conn, &selected_instance)?;
                    active_popup = ActivePopUp::None;
                    match read_events_from_db(&conn) {
                        Ok(e) => {
                            if !e.is_empty() {
                                match e.get(event_list_state.selected().unwrap_or(0)) {
                                    Some(sel_event) => {
                                        instance_count =
                                            read_instances_count_from_db(&conn, &sel_event.name)?;
                                        if instance_count == 0 {
                                            active_block = ActiveBlock::EventBlock;
                                        } else if let Some(selected) =
                                            instance_list_state.selected()
                                        {
                                            if selected >= instance_count {
                                                instance_list_state
                                                    .select(Some(instance_count - 1));
                                            }
                                        }
                                    }
                                    None => {}
                                }
                            }
                        }
                        Err(_) => {}
                    }
                }

                // Instances - For Both Popups
                (
                    KeyEvent {
                        code: KeyCode::Esc, ..
                    },
                    MenuItem::Instances,
                    ActiveBlock::InstanceBlock,
                    ActivePopUp::Update | ActivePopUp::Delete,
                ) => {
                    active_popup = ActivePopUp::None;
                }

                // Add Tab Keys
                (
                    KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    },
                    MenuItem::Add,
                    _,
                    ActivePopUp::None,
                ) => {
                    // text_areas[which].inactivate();
                    // which += 1;
                    if which + 1 >= text_areas.len() {
                        // which -= 1;
                        if add_given_ok {
                            insert_into_db(&conn, &mut text_areas)?;
                            add_given_ok = validate_text_areas(&text_areas);
                            which = 0;
                        }
                    } else {
                        text_areas[which].inactivate();
                        which += 1;
                        text_areas[which].activate();
                    }
                }

                (
                    KeyEvent {
                        code: KeyCode::Esc, ..
                    },
                    MenuItem::Add,
                    _,
                    ActivePopUp::None,
                ) => {
                    text_areas[which].inactivate();
                    which = which.saturating_sub(1);
                    text_areas[which].activate();
                }

                (input, MenuItem::Add, _, ActivePopUp::None) => {
                    if text_areas[which].text_area.input(input) {
                        text_areas[which].validate();
                    }
                    add_given_ok = validate_text_areas(&text_areas);
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }
    Ok(())
}
