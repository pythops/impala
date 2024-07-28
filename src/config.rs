use toml;

use dirs;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorMode {
    Auto,
    Dark,
    Light,
}

impl<'de> Deserialize<'de> for ColorMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>, {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "auto" => Ok(ColorMode::Auto),
            "light" => Ok(ColorMode::Light),
            _ => Ok(ColorMode::Dark),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_switch_mode")]
    pub switch: char,

    #[serde(default = "default_device_mode")]
    pub mode: String,

    #[serde(default = "default_unicode")]
    pub unicode: bool,

    #[serde(default = "default_color_mode")]
    pub color_mode: ColorMode,

    #[serde(default)]
    pub device: Device,

    #[serde(default)]
    pub station: Station,

    #[serde(default)]
    pub ap: AccessPoint,
}

fn default_switch_mode() -> char {
    'r'
}

fn default_device_mode() -> String {
    String::from("station")
}

fn default_unicode() -> bool {
    true
}

fn default_color_mode() -> ColorMode {
    ColorMode::Auto
}

// Device
#[derive(Deserialize, Debug)]
pub struct Device {
    #[serde(default = "default_show_device_infos")]
    pub infos: char,
    pub toggle_power: char,
}

impl Default for Device {
    fn default() -> Self {
        Self {
            infos: 'i',
            toggle_power: 'o',
        }
    }
}

fn default_show_device_infos() -> char {
    'i'
}

// Station
#[derive(Deserialize, Debug)]
pub struct Station {
    #[serde(default = "default_station_start_scanning")]
    pub start_scanning: char,

    #[serde(default = "default_station_toggle_connect")]
    pub toggle_connect: char,

    #[serde(default = "default_station_auto_scan")]
    pub auto_scan: bool,

    #[serde(default)]
    pub known_network: KnownNetwork,
}

impl Default for Station {
    fn default() -> Self {
        Self {
            start_scanning: 's',
            toggle_connect: ' ',
            known_network: KnownNetwork::default(),
            auto_scan: true,
        }
    }
}

fn default_station_start_scanning() -> char {
    's'
}

fn default_station_toggle_connect() -> char {
    ' '
}

#[derive(Deserialize, Debug)]
pub struct KnownNetwork {
    #[serde(default = "default_station_remove_known_network")]
    pub remove: char,
    pub toggle_autoconnect: char,
}

impl Default for KnownNetwork {
    fn default() -> Self {
        Self {
            remove: 'd',
            toggle_autoconnect: 'a',
        }
    }
}

fn default_station_remove_known_network() -> char {
    'd'
}

fn default_station_auto_scan() -> bool {
    true
}

// Access Point
#[derive(Deserialize, Debug)]
pub struct AccessPoint {
    #[serde(default = "default_ap_start")]
    pub start: char,

    #[serde(default = "default_ap_stop")]
    pub stop: char,
}

impl Default for AccessPoint {
    fn default() -> Self {
        Self {
            start: 'n',
            stop: 'x',
        }
    }
}

fn default_ap_start() -> char {
    'n'
}

fn default_ap_stop() -> char {
    'x'
}

impl Config {
    pub fn new() -> Self {
        let conf_path = dirs::config_dir()
            .unwrap()
            .join("impala")
            .join("config.toml");

        let config = std::fs::read_to_string(conf_path).unwrap_or_default();
        let app_config: Config = toml::from_str(&config).unwrap();

        app_config
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
