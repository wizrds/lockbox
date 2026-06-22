use serde::{Serialize, Deserialize};
use validator::Validate;


#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(default)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub endpoint: String,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        TelemetryConfig {
            enabled: false,
            endpoint: "".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(default)]
pub struct SentryConfig {
    pub enabled: bool,
    pub dsn: String,
}

impl Default for SentryConfig {
    fn default() -> Self {
        SentryConfig {
            enabled: false,
            dsn: "".to_string(),
        }
    }
}