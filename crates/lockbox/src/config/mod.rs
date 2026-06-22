pub mod errors;
pub mod builder;
pub mod de;
pub mod secret;
pub mod database;
pub mod server;
pub mod task_queue;
pub mod telemetry;

use serde::{Serialize, Deserialize};
use validator::Validate;

use builder::AppConfigBuilder;
use database::DatabaseConfig;
use server::ServerConfig;
use task_queue::TaskQueueConfig;
use telemetry::{TelemetryConfig, SentryConfig};


#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(default)]
pub struct Config {
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

impl Config {
    pub fn builder() -> AppConfigBuilder<Self> {
        AppConfigBuilder::new()
            .with_env()
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
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
