use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    pub devices: Vec<DeviceConfig>,
    #[serde(default)]
    pub tray: TrayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_update_interval")]
    pub update_interval: u64, // seconds
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub name: String,
    pub address: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_low_battery_threshold")]
    pub low_battery_threshold: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_false")]
    pub show_percentage_in_tray: bool,
    #[serde(default = "default_icon_theme")]
    pub icon_theme: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            update_interval: default_update_interval(),
            log_level: default_log_level(),
        }
    }
}

impl Default for TrayConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            show_percentage_in_tray: default_false(),
            icon_theme: default_icon_theme(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            devices: vec![],
            tray: TrayConfig::default(),
        }
    }
}

// Default value functions for serde
fn default_update_interval() -> u64 {
    60
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_low_battery_threshold() -> u8 {
    20
}

fn default_icon_theme() -> String {
    "battery".to_string()
}

impl Config {
    /// Load config from the default location or create a default one
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            Self::load_from_file(&config_path)
        } else {
            // Create default config if none exists
            println!("No config file found at {}", config_path.display());
            println!("Creating default config...");
            let config = Self::default_with_example();
            config.save(&config_path)?;
            Ok(config)
        }
    }

    /// Load config from a specific file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        Ok(config)
    }

    /// Save config to a file
    pub fn save(&self, path: &Path) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory: {}", parent.display())
            })?;
        }

        let toml_string =
            toml::to_string_pretty(self).context("Failed to serialize config to TOML")?;

        fs::write(path, toml_string)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        println!("Config saved to: {}", path.display());
        Ok(())
    }

    /// Get the default config path
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("zmk-battery-monitor");

        Ok(config_dir.join("config.toml"))
    }

    /// Create a default config with example device
    pub fn default_with_example() -> Self {
        Self {
            general: GeneralConfig::default(),
            devices: vec![
                DeviceConfig {
                    name: "Example Keyboard".to_string(),
                    address: "00:00:00:00:00:00".to_string(),
                    enabled: false,
                    low_battery_threshold: 20,
                },
                DeviceConfig {
                    name: "Krypton-KBD".to_string(),
                    address: "D2:75:8A:E6:6A:FD".to_string(),
                    enabled: true,
                    low_battery_threshold: 20,
                },
            ],
            tray: TrayConfig::default(),
        }
    }

    /// Get the first enabled device
    pub fn get_primary_device(&self) -> Option<&DeviceConfig> {
        self.devices.iter().find(|d| d.enabled)
    }

    /// Get all enabled devices
    pub fn get_enabled_devices(&self) -> Vec<&DeviceConfig> {
        self.devices.iter().filter(|d| d.enabled).collect()
    }

    /// Generate a template config file
    pub fn generate_template() -> String {
        let template = r#"# ZMK Battery Monitor Configuration

[general]
# Update interval in seconds
update_interval = 60
# Log level: trace, debug, info, warn, error
log_level = "info"

# Define your keyboards here
# You can have multiple devices and enable/disable them individually

[[devices]]
name = "My ZMK Keyboard"
address = "00:00:00:00:00:00"  # Replace with your keyboard's MAC address
enabled = true
low_battery_threshold = 20

# Example of a second keyboard (disabled)
# [[devices]]
# name = "Second Keyboard"
# address = "11:11:11:11:11:11"
# enabled = false
# low_battery_threshold = 15

[tray]
enabled = true
show_percentage_in_tray = false
icon_theme = "battery"  # Icon name for system tray
"#;
        template.to_string()
    }
}
