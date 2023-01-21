use tui_textarea::TextArea;

use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders};
pub struct TextAreaContainer<'a> {
    pub text_area: TextArea<'a>,
    pub title: String,
    // validator: Fn
}

impl TextAreaContainer<'_> {
    pub fn initialize_title(&mut self) {
        let b = self
            .text_area
            .block()
            .cloned()
            .unwrap_or_else(|| Block::default().borders(Borders::ALL));
        self.text_area
            .set_block(b.style(Style::default()).title(self.title.to_string()))
    }

    pub fn inactivate(&mut self) {
        self.text_area.set_cursor_line_style(Style::default());
        self.text_area.set_cursor_style(Style::default());
        let b = self
            .text_area
            .block()
            .cloned()
            .unwrap_or_else(|| Block::default().borders(Borders::ALL));
        self.text_area
            .set_block(b.style(Style::default().fg(Color::DarkGray)));
    }

    pub fn activate(&mut self) {
        self.text_area
            .set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
        self.text_area
            .set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        let b = self
            .text_area
            .block()
            .cloned()
            .unwrap_or_else(|| Block::default().borders(Borders::ALL));
        self.text_area.set_block(b.style(Style::default()));
    }
}

pub fn get_text_areas() -> [TextAreaContainer<'static>; 8] {
    let mut text_areas = [
        TextAreaContainer {
            text_area: TextArea::default(),
            title: "Event Name".to_string(),
        },
        TextAreaContainer {
            text_area: TextArea::default(),
            title: "Event Type".to_string(),
        },
        TextAreaContainer {
            text_area: TextArea::default(),
            title: "Instance Name".to_string(),
        },
        TextAreaContainer {
            text_area: TextArea::default(),
            title: "Is Recurring?".to_string(),
        },
        TextAreaContainer {
            text_area: TextArea::default(),
            title: "Is Finished?".to_string(),
        },
        TextAreaContainer {
            text_area: TextArea::default(),
            title: "% Completed".to_string(),
        },
        TextAreaContainer {
            text_area: TextArea::default(),
            title: "# Completed".to_string(),
        },
        TextAreaContainer {
            text_area: TextArea::default(),
            title: "Remaining Days".to_string(),
        },
    ];

    for ta in text_areas.iter_mut() {
        ta.initialize_title();
    }

    text_areas[0].activate();
    for ta in text_areas.iter_mut().skip(1) {
        ta.inactivate();
    }

    text_areas
}
