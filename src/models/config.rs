use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub window: WindowConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WindowConfig {
    pub width: f64,
    pub height: f64,
    pub x: Option<f64>,
    pub y: Option<f64>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig::default(),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 1100.0,
            height: 600.0,
            x: None,
            y: None,
        }
    }
}

impl AppConfig {
    /// Load configuration from disk
    pub fn load() -> Result<Self, confy::ConfyError> {
        confy::load("dxbleuio", None)
    }

    /// Save configuration to disk
    pub fn save(&self) -> Result<(), confy::ConfyError> {
        confy::store("dxbleuio", None, self)
    }
}
