use serde::{Serialize, Deserialize};
use validator::Validate;

use lockbox_core::database::DatabaseOptions;

use crate::config::de::nested;


#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(default)]
pub struct DatabaseConfig {
    #[validate(length(min=1, max=255))]
    pub primary: String,
    #[serde(deserialize_with = "nested")]
    pub replicas: Vec<String>,
    pub options: DatabaseOptions,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            primary: "sqlite://database.db?mode=rwc".to_string(),
            replicas: vec![],
            options: DatabaseOptions::default(),
        }
    }
}
