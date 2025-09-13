use crossterm::event::{KeyEvent, KeyModifiers};
use toml;

use dirs;
use serde::{Deserialize, de::IntoDeserializer};

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_switch_mode")]
    pub switch: KeyBind,

    #[serde(default = "default_device_mode")]
    pub mode: String,

    #[serde(default)]
    pub device: Device,

    #[serde(default)]
    pub station: Station,

    #[serde(default)]
    pub ap: AccessPoint,
}

fn default_switch_mode() -> KeyBind {
    (KeyModifiers::CONTROL, 'r').into()
}

fn default_device_mode() -> String {
    "station".to_string()
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum KeyBind {
    SingleChar(char),
    WithModKey((KeyModifiers, char)),
}

impl core::fmt::Display for KeyBind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SingleChar(ch) => write!(f, "{ch}"),
            Self::WithModKey((modifier, ch)) => write!(f, "{modifier} + {ch}"),
        }
    }
}

impl From<char> for KeyBind {
    fn from(value: char) -> Self {
        Self::SingleChar(value)
    }
}

impl From<(KeyModifiers, char)> for KeyBind {
    fn from(value: (KeyModifiers, char)) -> Self {
        KeyBind::WithModKey((value.0, value.1))
    }
}
impl PartialEq<KeyEvent> for KeyBind {
    // Assumption is that we're using this in event loop, where we're already matching KeyCode::Char
    fn eq(&self, other: &KeyEvent) -> bool {
        match self {
            Self::SingleChar(ch) => *ch == other.code.as_char().unwrap(),
            Self::WithModKey((modkey, ch)) => {
                *ch == other.code.as_char().unwrap() && *modkey == other.modifiers
            }
        }
    }
}

// Device
#[derive(Deserialize, Debug)]
pub struct Device {
    #[serde(default = "default_show_device_infos")]
    pub infos: KeyBind,
    pub toggle_power: KeyBind,
}

impl Default for Device {
    fn default() -> Self {
        Self {
            infos: 'i'.into(),
            toggle_power: 'o'.into(),
        }
    }
}

fn default_show_device_infos() -> KeyBind {
    'i'.into()
}

// Station
#[derive(Deserialize, Debug)]
pub struct Station {
    #[serde(default = "default_station_start_scanning")]
    pub start_scanning: KeyBind,

    #[serde(default = "default_station_toggle_connect")]
    pub toggle_connect: KeyBind,

    #[serde(default)]
    pub known_network: KnownNetwork,
}

impl Default for Station {
    fn default() -> Self {
        Self {
            start_scanning: 's'.into(),
            toggle_connect: ' '.into(),
            known_network: KnownNetwork::default(),
        }
    }
}

fn default_station_start_scanning() -> KeyBind {
    's'.into()
}

fn default_station_toggle_connect() -> KeyBind {
    ' '.into()
}

#[derive(Deserialize, Debug)]
pub struct KnownNetwork {
    #[serde(default = "default_station_remove_known_network")]
    pub remove: KeyBind,
    pub toggle_autoconnect: KeyBind,
}

impl Default for KnownNetwork {
    fn default() -> Self {
        Self {
            remove: 'd'.into(),
            toggle_autoconnect: 'a'.into(),
        }
    }
}

fn default_station_remove_known_network() -> KeyBind {
    'd'.into()
}

// Access Point
#[derive(Deserialize, Debug)]
pub struct AccessPoint {
    #[serde(default = "default_ap_start")]
    pub start: KeyBind,

    #[serde(default = "default_ap_stop")]
    pub stop: KeyBind,
}

impl Default for AccessPoint {
    fn default() -> Self {
        Self {
            start: 'n'.into(),
            stop: 'x'.into(),
        }
    }
}

fn default_ap_start() -> KeyBind {
    'n'.into()
}

fn default_ap_stop() -> KeyBind {
    'x'.into()
}

impl Config {
    pub fn new() -> Self {
        let conf_path = "./config.toml";
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
