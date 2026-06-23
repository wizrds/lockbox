use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub count: u64,
    pub next_page: Option<u64>,
    pub previous_page: Option<u64>,
}
