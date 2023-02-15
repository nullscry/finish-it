use rusqlite::Connection;

use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table,
        TableState,
    },
};

use super::db::{read_items_from_db, read_topics_from_db};
use super::{Item, Topic};
use crate::{ActiveBlock, Confirm};

pub fn render_home<'a>() -> Paragraph<'a> {
    let home = Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "Finish It!",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw(
            "A TUI application to track your progress in any one time or recurring task.",
        )]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![
            Span::raw("Use the "),
            Span::styled(
                "Topics tab (Alt+t or Home)",
                Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" to track, modify or delete your Topics and Items"),
        ]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![
            Span::raw("Use the "),
            Span::styled(
                "Add tab (Alt+a or Insert)",
                Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" to add new Topics and/or Items"),
        ]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![
            Span::raw("Use the "),
            Span::styled(
                "Quit tab (Alt+q or End)",
                Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" to quit Finish It"),
        ]),
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

pub fn render_topics<'a>(
    event_list_state: &ListState,
    item_list_state: &TableState,
    conn: &Connection,
    active_block: &ActiveBlock,
) -> (List<'a>, Item, Topic, Table<'a>) {
    let (list_highlight, table_highlight) = match active_block {
        ActiveBlock::Event => (Color::Red, Color::Yellow),
        ActiveBlock::InstanceBlock => (Color::Yellow, Color::Red),
    };
    let topics = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Topics")
        .border_type(BorderType::Plain);

    let event_list = read_topics_from_db(conn).expect("can fetch Topic list");

    let items: Vec<_> = event_list
        .iter()
        .map(|item| {
            ListItem::new(Spans::from(vec![Span::styled(
                item.name.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let selected_event = event_list
        .get(event_list_state.selected().unwrap_or(0))
        .unwrap_or(&Topic::default())
        .clone();

    let list = List::new(items).block(topics).highlight_style(
        Style::default()
            .bg(list_highlight)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let item_list = read_items_from_db(conn, &selected_event.name).expect("can fetch Topic list");

    let selected_item = match item_list_state.selected() {
        Some(i) => match item_list.get(i) {
            Some(inst) => inst.to_owned(),
            None => Item::default(),
        },
        None => Item::default(),
    };

    let mut rows: Vec<Row<'a>> = Vec::new();
    for item in item_list {
        // rows.push(Row::new(item.as_cells()));
        rows.push(Row::new(vec![
            Cell::from(Span::raw(item.id.to_string())),
            Cell::from(Span::raw(item.name.to_string())),
            Cell::from(Span::raw(item.get_dot_vec())),
            Cell::from(Span::raw(Confirm::get_confirm_str(
                &item.isrecurring.to_string(),
            ))),
            Cell::from(Span::raw(item.percentage.to_string())),
            Cell::from(Span::raw(item.timesfinished.to_string())),
            Cell::from(Span::raw(item.days_left())),
            Cell::from(Span::raw(item.created.date_naive().to_string())),
        ]));
    }

    let item_detail = Table::new(rows)
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
                "Progress",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Recurring?",
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
                .title("Items")
                .border_type(BorderType::Plain),
        )
        .widths(&[
            Constraint::Percentage(5),
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(15),
        ])
        .highlight_style(
            Style::default()
                .bg(table_highlight)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    (list, selected_item, selected_event, item_detail)
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
