pub mod app;

pub mod event;

pub mod ui;

pub mod tui;

pub mod handler;

pub mod config;

pub mod notification;

pub mod device;

pub mod adapter;

pub mod cli;

pub mod rfkill;

pub mod mode;

pub mod reset;

pub mod agent;

pub fn iwd_network_name(name: &str) -> String {
    match name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ' ')
    {
        true => name.to_string(),
        false => format!("={}", hex::encode(name)),
    }
}
