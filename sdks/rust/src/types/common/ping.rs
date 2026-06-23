use serde::Deserialize;


#[derive(Debug, Clone, Deserialize)]
pub struct PingResponse {
    pub name: String,
    pub version: String,
}
