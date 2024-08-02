use crate::app::{App, AppResult};
use crate::event::EventHandler;
use crate::ui;
use ratatui::style::{Color, Style};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::Backend;
use ratatui::Terminal;
use std::io;
use std::panic;

#[derive(Debug)]
pub struct Palette {
    pub text: Style,

    pub input_text: Style,
    pub input_box: Style,

    pub status_bar: Style,

    pub active_border: Style,
    pub inactive_border: Style,

    pub active_table_header: Style,
    pub inactive_table_header: Style,
    pub active_table_row: Style,

    pub notification_info: Style,
    pub notification_warning: Style,
    pub notification_error: Style,
}

#[derive(Debug)]
pub struct Tui<B: Backend> {
    terminal: Terminal<B>,
    pub events: EventHandler,
    pub palette: Palette,
}

// Build palette from color configuration
pub fn generate_palette(light_mode:  bool, monochrome: bool) -> Palette {

    // Light color scheme
    if light_mode {

        // Monochrome black-on-white color scheme
        if monochrome {
            Palette {
                text: Style::default().fg(Color::Black),

                input_text: Style::default().fg(Color::White),
                input_box: Style::default().bg(Color::Black),

                status_bar: Style::default().fg(Color::White).bg(Color::Black),
            
                active_border: Style::default().fg(Color::Black),
                inactive_border: Style::default().fg(Color::Black),

                active_table_header: Style::default().fg(Color::Black),
                inactive_table_header: Style::default().fg(Color::Black),
                active_table_row: Style::default().fg(Color::White).bg(Color::Black),

                notification_info: Style::default().fg(Color::Black),
                notification_warning: Style::default().fg(Color::Black),
                notification_error: Style::default().fg(Color::Black),
            }

        // Colorful light scheme
        } else {
            Palette {
                text: Style::default().fg(Color::Black),

                input_text: Style::default().fg(Color::Black),
                input_box: Style::default().bg(Color::DarkGray),

                status_bar: Style::default().fg(Color::White).bg(Color::Black),
            
                active_border: Style::default().fg(Color::Green),
                inactive_border: Style::default().fg(Color::Black),

                active_table_header: Style::default().fg(Color::Yellow),
                inactive_table_header: Style::default().fg(Color::Black),
                active_table_row: Style::default().fg(Color::White).bg(Color::DarkGray),

                notification_info: Style::default().fg(Color::Green),
                notification_warning: Style::default().fg(Color::Yellow),
                notification_error: Style::default().fg(Color::Red),
            }
        }

    // Dark color scheme
    } else {

        // Monochrome white-on-black color scheme
        if monochrome {
            Palette {
                text: Style::default().fg(Color::White),

                input_text: Style::default().fg(Color::Black),
                input_box: Style::default().bg(Color::White),

                status_bar: Style::default().fg(Color::Black).bg(Color::White),
            
                active_border: Style::default().fg(Color::White),
                inactive_border: Style::default().fg(Color::White),

                active_table_header: Style::default().fg(Color::White),
                inactive_table_header: Style::default().fg(Color::White),
                active_table_row: Style::default().fg(Color::Black).bg(Color::White),

                notification_info: Style::default().fg(Color::White),
                notification_warning: Style::default().fg(Color::White),
                notification_error: Style::default().fg(Color::White),
            }

        // Colorful dark scheme
        } else {
            Palette {
                text: Style::default().fg(Color::White),

                input_text: Style::default().fg(Color::White),
                input_box: Style::default().bg(Color::DarkGray),

                status_bar: Style::default().fg(Color::Black).bg(Color::White),
            
                active_border: Style::default().fg(Color::Green),
                inactive_border: Style::default().fg(Color::White),

                active_table_header: Style::default().fg(Color::Yellow),
                inactive_table_header: Style::default().fg(Color::White),
                active_table_row: Style::default().fg(Color::White).bg(Color::DarkGray),

                notification_info: Style::default().fg(Color::Green),
                notification_warning: Style::default().fg(Color::Yellow),
                notification_error: Style::default().fg(Color::Red),
            }
        }
    }
}

impl<B: Backend> Tui<B> {
    pub fn new(terminal: Terminal<B>, events: EventHandler, palette: Palette) -> Self {
        Self { terminal, events, palette }
    }

    pub fn init(&mut self) -> AppResult<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;
        crossterm::execute!(io::stdout(), EnterAlternateScreen)?;

        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("failed to reset the terminal");
            panic_hook(panic);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    pub fn draw(&mut self, app: &mut App) -> AppResult<()> {
        self.terminal.draw(|frame| ui::render(app, &self.palette, frame))?;
        Ok(())
    }

    fn reset() -> AppResult<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        crossterm::execute!(io::stderr(), LeaveAlternateScreen)?;
        Ok(())
    }

    pub fn exit(&mut self) -> AppResult<()> {
        Self::reset()?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}
