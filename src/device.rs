use std::sync::Arc;

use iwdrs::{device::Device as iwdDevice, modes::Mode, session::Session};
use tokio::sync::mpsc::UnboundedSender;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Row, Table, TableState},
};

use crate::{
    app::{AppResult, FocusedBlock},
    config::Config,
    event::Event,
    mode::{ap::AccessPoint, station::Station},
};

#[derive(Debug, Clone)]
pub struct Device {
    device: iwdDevice,
    session: Arc<Session>,
    pub name: String,
    pub address: String,
    pub mode: Mode,
    pub is_powered: bool,
    pub station: Option<Station>,
    pub ap: Option<AccessPoint>,
}

impl Device {
    pub async fn new(session: Arc<Session>) -> AppResult<Self> {
        let device = session.devices().await.unwrap().pop().unwrap();
        let name = device.name().await?;
        let address = device.address().await?;
        let mode = device.get_mode().await?;
        let is_powered = device.is_powered().await?;

        let (station, ap) = match mode {
            Mode::Station => {
                if let Ok(station) = Station::new(session.clone()).await {
                    (Some(station), None)
                } else {
                    (None, None)
                }
            }
            Mode::Ap => {
                if let Ok(ap) = AccessPoint::new(session.clone()).await {
                    (None, Some(ap))
                } else {
                    (None, None)
                }
            }
        };

        Ok(Self {
            device,
            session,
            name,
            address,
            mode,
            is_powered,
            station,
            ap,
        })
    }

    pub async fn set_mode(&self, mode: Mode) -> AppResult<()> {
        self.device.set_mode(mode).await?;
        Ok(())
    }

    pub async fn power_off(&self) -> AppResult<()> {
        self.device.set_power(false).await?;
        Ok(())
    }

    pub async fn power_on(&self) -> AppResult<()> {
        self.device.set_power(true).await?;
        Ok(())
    }

    pub async fn refresh(&mut self, sender: UnboundedSender<Event>) -> AppResult<()> {
        self.is_powered = self.device.is_powered().await?;
        self.mode = self.device.get_mode().await?;
        if self.is_powered {
            match self.mode {
                Mode::Station => {
                    if let Some(station) = &mut self.station {
                        if station.diagnostic.is_none() {
                            sender.send(Event::Reset(Mode::Station))?;
                        } else {
                            station.refresh().await?;
                        }
                    } else {
                        self.station = Station::new(self.session.clone()).await.ok();
                    }
                }
                Mode::Ap => {
                    if let Some(ap) = &mut self.ap {
                        ap.refresh().await?;
                    } else {
                        self.ap = AccessPoint::new(self.session.clone()).await.ok();
                    }
                }
            }
        }
        Ok(())
    }

    pub fn render(&mut self, frame: &mut Frame, focused_block: FocusedBlock, config: Arc<Config>) {
        let (device_block, help_block) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Fill(1),
                    Constraint::Length(5),
                    Constraint::Length(1),
                ])
                .margin(1)
                .split(frame.area());
            (chunks[1], chunks[2])
        };

        //
        // Device
        //
        let row = Row::new(vec![Line::from(self.name.clone()).centered(), {
            if self.is_powered {
                Line::from("On").centered()
            } else {
                Line::from("Off").centered()
            }
        }]);

        let widths = [Constraint::Length(10), Constraint::Length(8)];

        let device_table = Table::new(vec![row], widths)
            .header({
                Row::new(vec![
                    Line::from("Name").yellow().centered(),
                    Line::from("Powered").yellow().centered(),
                ])
                .style(Style::new().bold())
                .bottom_margin(1)
            })
            .block(
                Block::default()
                    .title(" Device ")
                    .title_style(Style::default().bold())
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .border_type(BorderType::Thick)
                    .padding(Padding::horizontal(1)),
            )
            .column_spacing(1)
            .flex(Flex::SpaceAround)
            .row_highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White));

        let mut device_state = TableState::default().with_selected(0);
        frame.render_stateful_widget(device_table, device_block, &mut device_state);

        let help_message = match focused_block {
            FocusedBlock::Device => Line::from(vec![
                Span::from(config.device.infos.to_string()).bold(),
                Span::from(" Infos"),
                Span::from(" | "),
                Span::from(config.device.toggle_power.to_string()).bold(),
                Span::from(" Toggle Power"),
            ]),
            FocusedBlock::AdapterInfos => {
                Line::from(vec![Span::from("󱊷 ").bold(), Span::from(" Discard")])
            }
            _ => Line::from(""),
        };

        let help_message = help_message.centered().blue();

        frame.render_widget(help_message, help_block);
    }
}
