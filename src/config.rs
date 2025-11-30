use toml;

use dirs;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_switch_mode")]
    pub switch: char,

    #[serde(default = "default_device_mode")]
    pub mode: String,

    #[serde(default = "default_esc_quit")]
    pub esc_quit: bool,

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
    "station".to_string()
}

fn default_esc_quit() -> bool {
    false
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

    #[serde(default)]
    pub known_network: KnownNetwork,

    #[serde(default)]
    pub new_network: NewNetwork,
}

impl Default for Station {
    fn default() -> Self {
        Self {
            start_scanning: 's',
            known_network: KnownNetwork::default(),
            new_network: NewNetwork::default(),
        }
    }
}

fn default_station_start_scanning() -> char {
    's'
}

#[derive(Deserialize, Debug)]
pub struct KnownNetwork {
    #[serde(default = "default_station_remove_known_network")]
    pub remove: char,
    pub toggle_autoconnect: char,
    pub show_all: char,
    pub share: char,
}

impl Default for KnownNetwork {
    fn default() -> Self {
        Self {
            remove: 'd',
            toggle_autoconnect: 't',
            show_all: 'a',
            share: 'p',
        }
    }
}

fn default_station_remove_known_network() -> char {
    'd'
}

#[derive(Deserialize, Debug)]
pub struct NewNetwork {
    pub show_all: char,
}

impl Default for NewNetwork {
    fn default() -> Self {
        Self { show_all: 'a' }
    }
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
