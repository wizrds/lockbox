use serde::{Serialize, Deserialize};
use validator::Validate;


#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
#[serde(default)]
pub struct ServerConfig {
    #[validate(length(min=1, max=255))]
    pub host: String,
    #[validate(range(min = 1, max = 65535))]
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 8087,
        }
    }
}
