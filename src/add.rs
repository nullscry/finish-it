use std::str::FromStr;

use tui_textarea::{CursorMove, TextArea};

use tui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::Confirm;
pub enum AreaType {
    UInt,
    Percentage,
    Confirm,
    String,
}

pub struct TextAreaContainer<'a> {
    pub text_area: TextArea<'a>,
    pub title: String,
    ok: bool,
    area_type: AreaType,
}

impl TextAreaContainer<'_> {
    pub fn new(title: String, area_type: AreaType) -> Self {
        Self {
            text_area: TextArea::default(),
            title,
            ok: false,
            area_type,
        }
    }
    pub fn initialize_title(&mut self) {
        let b = self
            .text_area
            .block()
            .cloned()
            .unwrap_or_else(|| Block::default().borders(Borders::ALL));
        self.text_area
            .set_block(b.style(Style::default()).title(self.title.to_string()));
    }

    pub fn clear_text(&mut self) {
        self.text_area.move_cursor(CursorMove::Head);
        self.text_area.delete_line_by_end();
        self.ok = false;
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

    fn set_border_error(&mut self) {
        self.text_area
            .set_style(Style::default().fg(Color::LightRed));
    }

    fn set_border_ok(&mut self) {
        self.text_area
            .set_style(Style::default().fg(Color::LightGreen));
    }

    pub fn validate(&mut self) {
        self.ok = match self.area_type {
            AreaType::UInt => self.validate_value_uint(),
            AreaType::Percentage => self.validate_value_float(),
            AreaType::Confirm => self.validate_value_confirm(),
            AreaType::String => self.validate_value_string(),
        }
    }

    fn validate_value_string(&mut self) -> bool {
        if !self.text_area.lines()[0].is_empty() {
            self.set_border_ok();
            true
        } else {
            self.set_border_error();
            false
        }
    }

    fn validate_value_float(&mut self) -> bool {
        match self.text_area.lines()[0].parse::<u8>() {
            Ok(x) => match x {
                0..=100 => {
                    self.set_border_ok();
                    true
                }
                _ => {
                    self.set_border_error();
                    false
                }
            },
            Err(_) => {
                self.set_border_error();
                false
            }
        }
    }

    fn validate_value_uint(&mut self) -> bool {
        match self.text_area.lines()[0].parse::<usize>() {
            Ok(_) => {
                self.set_border_ok();
                true
            }
            Err(_) => {
                self.set_border_error();
                false
            }
        }
    }

    fn validate_value_confirm(&mut self) -> bool {
        match Confirm::from_str(&self.text_area.lines()[0]) {
            Ok(_) => {
                self.set_border_ok();
                true
            }
            Err(_) => {
                self.set_border_error();
                false
            }
        }
    }

    pub fn is_ok(&self) -> u8 {
        self.ok as u8
    }

    pub fn get_inner_data(&self) -> String {
        let default_str = String::from("0");
        let inner_data = self.text_area.lines().get(0).unwrap_or(&default_str).trim();

        match self.area_type {
            AreaType::Confirm => Confirm::get_confirm_u8_str(inner_data),
            _ => inner_data.to_string(),
        }
    }
}

pub fn get_text_areas() -> [TextAreaContainer<'static>; 6] {
    let mut text_areas = [
        TextAreaContainer::new("Topic Name".to_string(), AreaType::String),
        TextAreaContainer::new("Item Name".to_string(), AreaType::String),
        TextAreaContainer::new("Is Recurring? (Yes OR No)".to_string(), AreaType::Confirm),
        TextAreaContainer::new("% Completed [0, 100]".to_string(), AreaType::Percentage),
        TextAreaContainer::new("# Completed [0, ...]".to_string(), AreaType::UInt),
        TextAreaContainer::new("Day Limit [0, ...]".to_string(), AreaType::UInt),
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

pub fn validate_text_areas(text_areas: &[TextAreaContainer<'static>; 6]) -> bool {
    let ok_sum = text_areas
        .iter()
        .map(TextAreaContainer::is_ok)
        .collect::<Vec<u8>>()
        .iter()
        .sum::<u8>();

    ok_sum == text_areas.len() as u8
}

pub fn get_add_err_text() -> Paragraph<'static> {
    let err_text = vec![
        Spans::from(vec![Span::raw("Fill out the form to the left")]),
        Spans::from(vec![Span::raw(
            "until all the text in the boxes turns green, including this one.",
        )]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw(
            "Type out your desired new Item as laid out by the boxes.",
        )]),
        Spans::from(vec![Span::raw("Topic and Item cannot be blank.")]),
        Spans::from(vec![Span::raw(
            "The rest all show which values are allowed next to their names.",
        )]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw(
            "Press Enter key when you are done with a box to move on to the next one.",
        )]),
        Spans::from(vec![Span::raw("Press Esc key to go back one box")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw(
            "When everything is green and you are given the ok, ",
        )]),
        Spans::from(vec![Span::raw(
            "press Enter key at the last box to add your new item!",
        )]),
    ];
    let err_color = Color::LightRed;

    Paragraph::new(err_text).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(err_color))
            .title("Instructions")
            .border_type(BorderType::Plain),
    )
}

pub fn get_add_ok_text() -> Paragraph<'static> {
    let ok_text = vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Everything is in order!")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Confirm your item by pressing the Enter key when the last box is selected.")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("You can see your new addition by going back to the Topics screen with Alt+e after adding it.")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("")]),
    ];
    let ok_color = Color::LightGreen;

    Paragraph::new(ok_text).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(ok_color))
            .title("Instructions")
            .border_type(BorderType::Plain),
    )
}
