use tui_textarea::TextArea;

use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders};

pub fn inactivate(textarea: &mut TextArea<'_>) {
    textarea.set_cursor_line_style(Style::default());
    textarea.set_cursor_style(Style::default());
    let b = textarea
        .block()
        .cloned()
        .unwrap_or_else(|| Block::default().borders(Borders::ALL));
    textarea.set_block(b.style(Style::default().fg(Color::DarkGray)));
}

pub fn activate(textarea: &mut TextArea<'_>) {
    textarea.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
    textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    let b = textarea
        .block()
        .cloned()
        .unwrap_or_else(|| Block::default().borders(Borders::ALL));
    textarea.set_block(b.style(Style::default()));
}

pub fn initialize_title(textarea: &mut TextArea<'_>, title: &str) {
    let b = textarea
        .block()
        .cloned()
        .unwrap_or_else(|| Block::default().borders(Borders::ALL));
    textarea.set_block(b.style(Style::default()).title(title.to_string()))
}
