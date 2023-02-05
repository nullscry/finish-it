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
use crate::ActiveBlock;

pub fn render_home<'a>() -> Paragraph<'a> {
    let home = Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Welcome")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("to")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "Topic-CLI",
            Style::default().fg(Color::LightBlue),
        )]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Press 'Alt+e' to access topics, 'Alt+a' to add items and 'Alt+u' to update and 'Alt+d' to delete the currently selected Topic.")]),
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
        rows.push(Row::new(vec![
            Cell::from(Span::raw(item.id.to_string())),
            Cell::from(Span::raw(item.name.to_string())),
            Cell::from(Span::raw(item.get_dot_vec())),
            Cell::from(Span::raw(item.isrecurring.to_string())),
            Cell::from(Span::raw(item.percentage.to_string())),
            Cell::from(Span::raw(item.timesfinished.to_string())),
            Cell::from(Span::raw(item.daylimit.to_string())),
            Cell::from(Span::raw(item.created.to_string())),
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
                "Recur",
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
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(20),
        ])
        .highlight_style(
            Style::default()
                .bg(table_highlight)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    // let selected_item = item_list
    //     .get(
    //         item_list_state
    //             .selected()
    //             .expect("there is always a selected Topic"),
    //     )
    //     .expect("exists")
    //     .clone();

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
