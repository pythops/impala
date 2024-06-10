use anyhow::Result;
use iwdrs::adapter::Adapter as iwdAdapter;
use iwdrs::device::Device as iwdDevice;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders, Cell, Clear, Padding, Row, Table},
    Frame,
};

use crate::app::ColorMode;

#[derive(Debug, Clone)]
pub struct Device {
    pub d: iwdDevice,
    pub name: String,
    pub address: String,
    pub mode: String,
    pub is_powered: bool,
    pub adapter: Adapter,
}

#[derive(Debug, Clone)]
pub struct Adapter {
    pub a: iwdAdapter,
    pub name: String,
    pub model: Option<String>,
    pub vendor: Option<String>,
    pub supported_modes: Vec<String>,
}

impl Adapter {
    pub async fn new(a: iwdAdapter) -> Result<Self> {
        let name = a.name().await?;
        let model = a.model().await.ok();
        let vendor = a.vendor().await.ok();
        let supported_modes = a.supported_modes().await?;
        Ok(Self {
            a,
            name,
            model,
            vendor,
            supported_modes,
        })
    }
}

impl Device {
    pub async fn new(d: iwdDevice) -> Result<Self> {
        let name = d.name().await?;
        let address = d.address().await?;
        let mode = d.get_mode().await?;
        let is_powered = d.is_powered().await?;
        let adapter = {
            let iwd_adapter = d.adapter().await?;
            Adapter::new(iwd_adapter).await?
        };

        Ok(Self {
            d,
            name,
            address,
            mode,
            is_powered,
            adapter,
        })
    }
    pub async fn refresh(&mut self) -> Result<()> {
        let mode = self.d.get_mode().await?;
        let is_powered = self.d.is_powered().await?;
        self.mode = mode;
        self.is_powered = is_powered;
        Ok(())
    }

    pub fn render(&self, frame: &mut Frame, color_mode: ColorMode) {
        let popup_layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(10),
                    Constraint::Length(9),
                    Constraint::Fill(1),
                ]
                .as_ref(),
            )
            .split(frame.size());

        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length((frame.size().width - 80) / 2),
                    Constraint::Min(80),
                    Constraint::Length((frame.size().width - 80) / 2),
                ]
                .as_ref(),
            )
            .split(popup_layout[1])[1];

        let mut rows = vec![
            Row::new(vec![
                Cell::from("name").style(Style::default().bold().yellow()),
                Cell::from(self.adapter.name.clone()),
            ]),
            Row::new(vec![
                Cell::from("address").style(Style::default().bold().yellow()),
                Cell::from(self.address.clone()),
            ]),
            Row::new(vec![
                Cell::from("Supported modes").style(Style::default().bold().yellow()),
                Cell::from(self.adapter.supported_modes.clone().join(" ")),
            ]),
        ];

        if let Some(model) = &self.adapter.model {
            rows.push(Row::new(vec![
                Cell::from("model").style(Style::default().bold().yellow()),
                Cell::from(model.clone()),
            ]))
        }

        if let Some(vendor) = &self.adapter.vendor {
            rows.push(Row::new(vec![
                Cell::from("vendor").style(Style::default().bold().yellow()),
                Cell::from(vendor.clone()),
            ]))
        }

        let widths = [Constraint::Length(20), Constraint::Fill(1)];

        let device_infos_table = Table::new(rows, widths)
            .block(
                Block::default()
                    .title(" Device Infos ")
                    .title_style(Style::default().bold())
                    .title_alignment(Alignment::Center)
                    .padding(Padding::uniform(1))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .border_type(BorderType::Thick),
            )
            .column_spacing(3)
            .style(match color_mode {
                ColorMode::Dark => Style::default().fg(Color::White),
                ColorMode::Light => Style::default().fg(Color::Black),
            })
            .highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_widget(Clear, area);
        frame.render_widget(device_infos_table, area);
    }
}
