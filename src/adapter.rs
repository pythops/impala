use anyhow::Result;
use std::sync::Arc;

use anyhow::Context;

use iwdrs::{adapter::Adapter as iwdAdapter, session::Session};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Flex, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders, Cell, Clear, Padding, Row, Table},
};

use crate::config::Config;

#[derive(Debug)]
pub struct Adapter {
    adapter: iwdAdapter,
    pub is_powered: bool,
    pub name: String,
    pub model: Option<String>,
    pub vendor: Option<String>,
    pub supported_modes: Vec<String>,
    pub config: Arc<Config>,
}

impl Adapter {
    pub async fn new(session: Arc<Session>, config: Arc<Config>) -> Result<Self> {
        let adapter = session
            .adapters()
            .await?
            .pop()
            .context("No adapter found")?;

        let is_powered = adapter.is_powered().await?;
        let name = adapter.name().await?;
        let model = adapter.model().await.ok();
        let vendor = adapter.vendor().await.ok();
        let supported_modes = adapter.supported_modes().await?;

        Ok(Self {
            adapter,
            is_powered,
            name,
            model,
            vendor,
            supported_modes,
            config,
        })
    }

    pub async fn refresh(&mut self) -> Result<()> {
        self.is_powered = self.adapter.is_powered().await?;
        Ok(())
    }

    pub fn render(&self, frame: &mut Frame, device_addr: String) {
        let popup_layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(9),
                Constraint::Fill(1),
            ])
            .flex(Flex::Start)
            .split(frame.area());

        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Min(80),
                Constraint::Fill(1),
            ])
            .split(popup_layout[1])[1];

        let mut rows = vec![
            Row::new(vec![
                Cell::from("name").style(Style::default().bold().yellow()),
                Cell::from(self.name.clone()),
            ]),
            Row::new(vec![
                Cell::from("address").style(Style::default().bold().yellow()),
                Cell::from(device_addr),
            ]),
            Row::new(vec![
                Cell::from("Supported modes").style(Style::default().bold().yellow()),
                Cell::from(self.supported_modes.clone().join(" ")),
            ]),
        ];

        if let Some(model) = &self.model {
            rows.push(Row::new(vec![
                Cell::from("model").style(Style::default().bold().yellow()),
                Cell::from(model.clone()),
            ]));
        }

        if let Some(vendor) = &self.vendor {
            rows.push(Row::new(vec![
                Cell::from("vendor").style(Style::default().bold().yellow()),
                Cell::from(vendor.clone()),
            ]));
        }

        let widths = [Constraint::Length(20), Constraint::Fill(1)];

        let device_infos_table = Table::new(rows, widths)
            .block(
                Block::default()
                    .title(" Adapter Infos ")
                    .title_style(Style::default().bold())
                    .title_alignment(Alignment::Center)
                    .padding(Padding::uniform(1))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .border_type(BorderType::Thick),
            )
            .column_spacing(3)
            .row_highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White));

        frame.render_widget(Clear, area);
        frame.render_widget(device_infos_table, area);
    }
}
