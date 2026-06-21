#![allow(unused)]

use std::path::Path;
use tokio::fs::{read_to_string as read_file, write as write_file};
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;
use clap::ValueEnum;


#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse YAML: {0}")]
    Yml(#[from] serde_norway::Error),

    #[error("Failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Unknown manifest format: {0}")]
    UnknownFormat(String)
}


#[derive(ValueEnum, Debug, Clone, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ManifestFormat {
    #[default]
    Yaml,
    Json,
}


/// Detect the format to use based on the file extension
/// 
/// # Arguments
/// * `path` - The path to the manifest file
/// 
/// # Returns
/// The detected manifest format
pub fn detect_format(path: impl AsRef<Path>) -> Result<ManifestFormat, ManifestError> {
    match path.as_ref().extension().and_then(|s| s.to_str()) {
        Some("yml") | Some("yaml") => Ok(ManifestFormat::Yaml),
        Some("json") => Ok(ManifestFormat::Json),
        _ => Err(ManifestError::UnknownFormat(path.as_ref().to_string_lossy().into_owned())),
    }
}


/// Load a manifest file and deserialize to type T
/// 
/// # Arguments
/// * `path` - The path to the manifest file
/// * `format` - The format of the manifest file
/// 
/// # Returns
/// The deserialized manifest
pub async fn load_manifest<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T, ManifestError> {
    let path = path.as_ref();
    let content = read_file(path).await?;
    match detect_format(path) {
        Ok(ManifestFormat::Yaml) => serde_norway::from_str(&content).map_err(ManifestError::Yml),
        Ok(ManifestFormat::Json) => serde_json::from_str(&content).map_err(ManifestError::Json),
        Err(e) => Err(e),
    }
}


/// Write a manifest file from a value of type T
/// 
/// # Arguments
/// * `value` - The value to serialize
/// * `path` - The path to the manifest file
/// * `format` - The format of the manifest file
/// 
/// # Returns
/// The result of the write operation
pub async fn save_manifest<T: Serialize>(value: &T, path: impl AsRef<Path>) -> Result<(), ManifestError> {
    let path = path.as_ref();
    let content = to_serialized_string(value, detect_format(path)?)?;
    write_file(path, content).await.map_err(ManifestError::Io)
}

/// Serialize a value of type T into a pretty string
/// 
/// # Arguments
/// * `value` - The value to serialize
/// * `format` - The format to serialize to
/// 
/// # Returns
/// The serialized pretty string
pub fn to_serialized_string_pretty<T: Serialize>(value: &T, format: ManifestFormat) -> Result<String, ManifestError> {
    match format {
        ManifestFormat::Yaml => serde_norway::to_string(value).map_err(ManifestError::Yml),
        ManifestFormat::Json => serde_json::to_string_pretty(value).map_err(ManifestError::Json),
    }
}


/// Serialize a value of type T into a string
/// 
/// # Arguments
/// * `value` - The value to serialize
/// * `format` - The format to serialize to
/// 
/// # Returns
/// The serialized string
pub fn to_serialized_string<T: Serialize>(value: &T, format: ManifestFormat) -> Result<String, ManifestError> {
    match format {
        ManifestFormat::Yaml => serde_norway::to_string(value).map_err(ManifestError::Yml),
        ManifestFormat::Json => serde_json::to_string(value).map_err(ManifestError::Json),
    }
}

