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

use super::db::{read_events_from_db, read_instances_from_db};
use super::InstanceItem;
use crate::ActiveBlock;

pub fn render_home<'a>() -> Paragraph<'a> {
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

pub fn render_events<'a>(
    event_list_state: &ListState,
    instance_list_state: &TableState,
    conn: &Connection,
    active_block: &ActiveBlock,
) -> (List<'a>, InstanceItem, Table<'a>) {
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

    let selected_instance = match instance_list_state.selected() {
        Some(i) => match instance_list.get(i) {
            Some(inst) => inst.to_owned(),
            None => InstanceItem::default(),
        },
        None => InstanceItem::default(),
    };

    let mut rows: Vec<Row<'a>> = Vec::new();
    for instance in instance_list {
        rows.push(Row::new(vec![
            Cell::from(Span::raw(instance.instanceid.to_string())),
            Cell::from(Span::raw(instance.name.to_string())),
            Cell::from(Span::raw(instance.get_dot_vec())),
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
                "Progress",
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
            Constraint::Percentage(35),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(15),
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

    (list, selected_instance, instance_detail)
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
