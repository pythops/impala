use toml;

use dirs;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_start_scanning")]
    pub start_scanning: char,

    #[serde(default = "default_toggle_connect")]
    pub toggle_connect: char,

    #[serde(default)]
    pub known_network: KnownNetwork,

    #[serde(default)]
    pub device: Device,
}

fn default_start_scanning() -> char {
    's'
}

fn default_toggle_connect() -> char {
    ' '
}

#[derive(Deserialize, Debug)]
pub struct KnownNetwork {
    #[serde(default = "default_remove_known_network")]
    pub remove: char,
}

impl Default for KnownNetwork {
    fn default() -> Self {
        Self { remove: 'd' }
    }
}

fn default_remove_known_network() -> char {
    'd'
}

// Device
#[derive(Deserialize, Debug)]
pub struct Device {
    #[serde(default = "default_show_device_infos")]
    pub infos: char,
}

impl Default for Device {
    fn default() -> Self {
        Self { infos: 'i' }
    }
}

fn default_show_device_infos() -> char {
    'i'
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
