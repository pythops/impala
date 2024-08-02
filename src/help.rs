use std::sync::Arc;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::Stylize,
    widgets::{
        Block, BorderType, Borders, Cell, Clear, Padding, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState,
    },
    Frame,
};

use crate::{
    tui::Palette,
    config::Config,
};

#[derive(Debug, Clone)]
pub struct Help {
    block_height: usize,
    state: TableState,
    keys: Vec<(Cell<'static>, &'static str)>,
}

impl Help {
    pub fn new(config: Arc<Config>, palette: &Palette) -> Self {
        let mut state = TableState::new().with_offset(0);
        state.select(Some(0));

        Self {
            block_height: 0,
            state,
            keys: vec![
                (
                    Cell::from("## Global").style(palette.active_table_header.bold()),
                    "",
                ),
                (Cell::from("Esc").bold(), "Dismiss different pop-ups"),
                (Cell::from("Tab, Right, l").bold(), "Move to next section"),
                (
                    Cell::from("Shift+Tab, Left, h").bold(),
                    "Move to previous section",
                ),
                (Cell::from("j or Down").bold(), "Scroll down"),
                (Cell::from("k or Up").bold(), "Scroll up"),
                (
                    Cell::from(format!("ctrl {}", config.switch)).bold(),
                    "Switch adapter mode",
                ),
                (Cell::from("?").bold(), "Show help"),
                (Cell::from("q or ctrl+c").bold(), "Quit"),
                (Cell::from(""), ""),
                (
                    Cell::from("## Device").style(palette.active_table_header.bold()),
                    "",
                ),
                (
                    Cell::from(config.device.infos.to_string()).bold(),
                    "Show device information",
                ),
                (
                    Cell::from(config.device.toggle_power.to_string()).bold(),
                    "Toggle device power",
                ),
                (Cell::from(""), ""),
                (
                    Cell::from("## Station").style(palette.active_table_header.bold()),
                    "",
                ),
                (
                    Cell::from(config.station.start_scanning.to_string()).bold(),
                    "Start scanning",
                ),
                (
                    Cell::from({
                        if config.station.toggle_connect == ' ' {
                            "Space".to_string()
                        } else {
                            config.station.toggle_connect.to_string()
                        }
                    })
                    .bold(),
                    "Connect/Disconnect the network",
                ),
                (
                    Cell::from("### Known Networks").style(palette.active_table_header.bold()),
                    "",
                ),
                (
                    Cell::from(config.station.known_network.remove.to_string()).bold(),
                    "Remove the network from the known networks list",
                ),
                (Cell::from(""), ""),
                (
                    Cell::from("## Access Point").style(palette.active_table_header.bold()),
                    "",
                ),
                (
                    Cell::from(config.ap.start.to_string()).bold(),
                    "Start a new access point",
                ),
                (
                    Cell::from(config.ap.stop.to_string()).bold(),
                    "Stop the running access point",
                ),
            ],
        }
    }

    pub fn scroll_down(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.keys.len().saturating_sub(self.block_height - 6) {
                    i
                } else {
                    i + 1
                }
            }
            None => 1,
        };
        *self.state.offset_mut() = i;
        self.state.select(Some(i));
    }
    pub fn scroll_up(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i > 1 {
                    i - 1
                } else {
                    0
                }
            }
            None => 1,
        };
        *self.state.offset_mut() = i;
        self.state.select(Some(i));
    }

    pub fn render(&mut self, palette: &Palette, frame: &mut Frame) {
        let block = help_rect(frame.size());

        self.block_height = block.height as usize;

        let row_title_width = 20;
        let table_block_padding = 1;

        let widths = [Constraint::Length(row_title_width), Constraint::Fill(1)];

        // Calculate maximum length for the help text detail
        let max_row_detail_length = (block.width - row_title_width
            // Cell padding
            - 3 * table_block_padding
            // Borders
            - 3) as usize;

        let rows: Vec<Row> = self
            .keys
            .iter()
            .flat_map(|key| {
                // Split help text into new lines to emulate line wrap
                let mut lines = Vec::new();

                // Start with whole text
                let mut remainder = key.1;

                // Keep splitting at maximum row length
                while remainder.len() > max_row_detail_length {
                    let (line, rest) = remainder.split_at(max_row_detail_length);

                    // Add new split line
                    lines.push(line.to_owned());
                    remainder = rest;
                }

                // Add the rest of the line
                lines.push(remainder.to_owned());

                // Create a table row for each line
                let mut rows = Vec::new();

                // First row: key.0 and first line of help text
                if !lines.is_empty() {
                    rows.push(
                        Row::new(vec![key.0.to_owned(), Cell::from(lines[0].clone())]).style(palette.text),
                    );
                }

                // Rest of rows: only the split lines of help text
                for line in lines.iter().skip(1) {
                    rows.push(Row::new(vec!["".to_owned(), line.clone()]).style(palette.text));
                }

                rows
            })
            .collect();
        let rows_len = self.keys.len().saturating_sub(self.block_height - 6);

        let table = Table::new(rows, widths).block(
            Block::default()
                .padding(Padding::uniform(table_block_padding))
                .title(" Help ")
                .title_style(palette.active_border)
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .style(palette.text)
                .border_type(BorderType::Thick)
                .border_style(palette.active_border),
        );

        frame.render_widget(Clear, block);
        frame.render_stateful_widget(table, block, &mut self.state);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let mut scrollbar_state =
            ScrollbarState::new(rows_len).position(self.state.selected().unwrap_or_default());
        frame.render_stateful_widget(
            scrollbar,
            block.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

pub fn help_rect(r: Rect) -> Rect {

    let width = if r.width > 80 { (r.width - 80) / 2 } else { r.width };

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(35),
                Constraint::Min(30),
                Constraint::Percentage(35),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(width),
                Constraint::Min(80),
                Constraint::Length(width),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
