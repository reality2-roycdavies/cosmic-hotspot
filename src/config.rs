use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub hotspot_interface: String,
    pub internet_interface: String,
    pub connection_name: String,
    pub ssid: String,
    pub password: String,
    pub band: String,
    pub gateway_ip: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotspot_interface: "wlan0".to_string(),
            internet_interface: "wlan1".to_string(),
            connection_name: "CosmicHotspot".to_string(),
            ssid: "CosmicHotspot".to_string(),
            password: "changeme123".to_string(),
            band: "bg".to_string(),
            gateway_ip: "192.168.44.1/24".to_string(),
        }
    }
}

impl Config {
    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("cosmic-hotspot").join("config.json"))
    }

    pub fn load() -> Self {
        Self::config_path()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path()
            .ok_or("Could not determine config path")?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {e}"))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {e}"))?;

        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write config: {e}"))?;

        Ok(())
    }
}
