use serde::{Serialize, Deserialize};
use validator::Validate;


#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(default)]
pub struct TaskQueueConfig {
    #[validate(range(min = 1))]
    pub worker_count: usize,
    #[validate(range(min = 1))]
    pub max_queue_capacity: usize,
    #[validate(range(min = 1))]
    pub batch_size: usize,
    #[validate(range(min = 1))]
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
