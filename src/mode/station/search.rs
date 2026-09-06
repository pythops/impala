use std::sync::Arc;

use ratatui::{
    Frame,
    layout::Rect,
    style::Stylize,
    text::{Line, Span},
    widgets::Paragraph,
};
use tui_input::Input;

use crate::config::Config;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchTarget {
    KnownNetworks,
    NewNetworks,
}

#[derive(Clone)]
pub struct Search {
    pub target: SearchTarget,
    pub input: Input,
    pub previous_selection: Option<usize>,
    pub case_sensitive: bool,
}

impl Search {
    pub fn new(
        target: SearchTarget,
        previous_selection: Option<usize>,
        case_sensitive: bool,
    ) -> Self {
        Self {
            target,
            input: Input::default(),
            previous_selection,
            case_sensitive,
        }
    }

    pub fn query(&self) -> &str {
        self.input.value()
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, config: Arc<Config>) {
        let prompt = Span::from(config.station.search.to_string()).bold();
        let query = Span::from(self.query()).bg(config.theme.background);

        let line = Line::from(vec![prompt, Span::raw(" "), query]);

        let paragraph = Paragraph::new(line).centered().blue();
        frame.render_widget(paragraph, area);

        let query_len = self.query().len();
        let line_len = query_len + 2;
        let pad = if area.width as usize > line_len {
            (area.width as usize - line_len) / 2
        } else {
            0
        };

        let visual_cursor = self.input.visual_cursor().min(query_len);
        let cursor_x = area.x + pad as u16 + 2 + visual_cursor as u16;
        frame.set_cursor_position((cursor_x, area.y));
    }
}

pub fn name_matches(name: &str, query: &str, case_sensitive: bool) -> bool {
    if query.is_empty() {
        return true;
    }

    if case_sensitive {
        name.contains(query)
    } else {
        name.to_lowercase().contains(&query.to_lowercase())
    }
}
