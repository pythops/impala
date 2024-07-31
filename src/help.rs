use std::sync::Arc;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, Padding, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState,
    },
    Frame,
};

use crate::config::{ColorMode, Config};

#[derive(Debug, Clone)]
pub struct Help {
    config: Arc<Config>,
    block_height: usize,
    state: TableState,
    keys: Vec<(Cell<'static>, &'static str)>,
}

impl Help {
    pub fn new(config: Arc<Config>) -> Self {
        let mut state = TableState::new().with_offset(0);
        state.select(Some(0));

        Self {
            config: config.clone(),
            block_height: 0,
            state,
            keys: vec![
                (
                    Cell::from("## Global").style(Style::new().bold().fg(Color::Yellow)),
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
                    Cell::from("## Device").style(Style::new().bold().fg(Color::Yellow)),
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
                    Cell::from("## Station").style(Style::new().bold().fg(Color::Yellow)),
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
                    Cell::from("### Known Networks").style(Style::new().bold().fg(Color::Yellow)),
                    "",
                ),
                (
                    Cell::from(config.station.known_network.remove.to_string()).bold(),
                    "Remove the network from the known networks list",
                ),
                (Cell::from(""), ""),
                (
                    Cell::from("## Access Point").style(Style::new().bold().fg(Color::Yellow)),
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

    pub fn render(&mut self, frame: &mut Frame) {
        let block = help_rect(frame.size());

        self.block_height = block.height as usize;

        let style = match self.config.color_mode {
            ColorMode::Light => Style::default().fg(Color::Black),
            _ => Style::default().fg(Color::White),
        };

        let narrow_mode = frame.size().width < self.config.small_layout_cols;
        let row_title_width = 20;
        let table_block_padding = if narrow_mode { 0 } else { 2 };

        let widths = [Constraint::Length(row_title_width), Constraint::Fill(1)];
        let max_row_detail_length = (
            block.width - row_title_width
            - 3 * table_block_padding // Cell padding
            - 3
            // Borders
        ) as usize;

        let rows: Vec<Row> = self
            .keys
            .iter()
            .flat_map(|key| {
                // Split row details into new rows to emulate line wrap
                let mut lines = Vec::new();
                let mut remainder = key.1;

                // Keep splitting
                while remainder.len() > max_row_detail_length {
                    let (line, rest) = remainder.split_at(max_row_detail_length);
                    lines.push(line.to_owned());
                    remainder = rest;
                }

                // Add the last split
                lines.push(remainder.to_owned());

                // Create a row for each line
                let mut rows = Vec::new();

                // First row: key.0 and first line of key.1
                if !lines.is_empty() {
                    rows.push(
                        Row::new(vec![key.0.to_owned(), Cell::from(lines[0].clone())]).style(style),
                    );
                }

                // Rest of rows: only remaining lines of key.1
                for line in lines.iter().skip(1) {
                    rows.push(Row::new(vec!["".to_owned(), line.clone()]).style(style));
                }

                rows
            })
            .collect();
        let rows_len = self.keys.len().saturating_sub(self.block_height - 6);

        let table = Table::new(rows, widths).block(
            Block::default()
                .padding(Padding::uniform(table_block_padding))
                .title(" Help ")
                .title_style(Style::default().bold().fg(Color::Green))
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .style(Style::default())
                .border_type(BorderType::Thick)
                .border_style(Style::default().fg(Color::Green)),
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
                Constraint::Length((r.width - 80) / 2),
                Constraint::Min(80),
                Constraint::Length((r.width - 80) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
