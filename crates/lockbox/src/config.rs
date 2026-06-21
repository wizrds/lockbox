#![allow(unused)]

use std::path::Path;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use validator::{Validate, ValidationErrors};
use figment::{Figment, Error as FigmentError, providers::{Format, Json, Yaml, Env, Serialized}};

use lockbox_core::database::DatabaseOptions;

use crate::constants::ENV_PREFIX;


#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Figment error: {0}")]
    Figment(#[from] FigmentError),
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationErrors),
}


#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
#[serde(default)]
pub struct AppConfig {
    #[validate(length(min=1, max=64))]
    pub default_tenant_id: String,
    #[validate(length(min=1, max=6))]
    pub default_namespace: String,
    pub task_queue: TaskQueueConfig,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub sentry: SentryConfig,
    pub telemetry: TelemetryConfig,
}

impl AppConfig {
    pub fn builder() -> AppConfigBuilder {
        AppConfigBuilder::default()
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            default_tenant_id: "default".to_string(),
            default_namespace: "apik".to_string(),
            task_queue: TaskQueueConfig::default(),
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            sentry: SentryConfig::default(),
            telemetry: TelemetryConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct TaskQueueConfig {
    pub worker_count: usize,
    pub max_queue_capacity: usize,
    pub batch_size: usize,
    pub batch_timeout_ms: usize,
}

impl Default for TaskQueueConfig {
    fn default() -> Self {
        TaskQueueConfig {
            worker_count: 4,
            max_queue_capacity: 100,
            batch_size: 16,
            batch_timeout_ms: 10,
        }
    }
}


#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct ServerConfig {
    pub host: String,
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

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct DatabaseConfig {
    pub primary: String,
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

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
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

pub struct AppConfigBuilder {
    figment: Figment,
}

impl AppConfigBuilder {
    pub fn with_file(&mut self, path: &str) -> &mut Self {
        let extension = Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default();

        self.figment = match extension {
            "json" => self.figment
                .clone()
                .merge(Json::file(path).nested()),
            "yaml" | "yml" => self.figment
                .clone()
                .merge(Yaml::file(path).nested()),
            _ => self.figment.clone(),
        };

        self
    }

    pub fn with_optional_file(&mut self, path: Option<&str>) -> &mut Self {
        match path {
            Some(p) => self.with_file(p),
            None => self,
        }
    }

    pub fn with_env(&mut self) -> &mut Self {
        self.figment = self.figment
            .clone()
            .merge(Env::prefixed(&format!("{}__", ENV_PREFIX))
            .split("__"));

        self
    }

    pub fn with_override_option(&mut self, key: &str, value: &str) -> &mut Self {
        self.figment = self.figment
            .clone()
            .merge(Serialized::default(key, value));

        self
    }

    pub fn with_optional_override_option(&mut self, key: &str, value: Option<&str>) -> &mut Self {
        match value {
            Some(v) => self.with_override_option(key, v),
            None => self,
        }
    }

    pub fn build(&self) -> Result<AppConfig, ConfigError> {
        let config = self.figment.extract::<AppConfig>()?;
        config.validate()?;
        
        Ok(config)
    }
}

impl Default for AppConfigBuilder {
    fn default() -> Self {
        AppConfigBuilder {
            figment: Figment::from(Serialized::defaults(AppConfig::default()))
        }
    }
}