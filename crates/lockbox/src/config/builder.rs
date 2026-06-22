#![allow(unused)]

use std::marker::PhantomData;
use serde::de::DeserializeOwned;
use validator::Validate;
use config::{ConfigBuilder, File, Environment, builder::AsyncState};

use crate::{constants::ENV_PREFIX, config::errors::ConfigError};


pub struct AppConfigBuilder<T> {
    builder: ConfigBuilder<AsyncState>,
    _marker: PhantomData<T>,
}

impl<T> AppConfigBuilder<T>
where
    T: DeserializeOwned + Validate,
{
    pub fn new() -> Self {
        Self {
            builder: ConfigBuilder::<AsyncState>::default(),
            _marker: PhantomData,
        }
    }

    pub fn with_file(mut self, path: impl AsRef<str>) -> Self {
        self.builder = self.builder
            .clone()
            .add_source(File::with_name(path.as_ref()));
        self
    }

    pub fn with_optional_file(self, path: Option<impl AsRef<str>>) -> Self {
        match path {
            Some(p) => self.with_file(p),
            None => self,
        }
    }

    pub fn with_env(mut self) -> Self {
        self.builder = self.builder
            .clone()
            .add_source(
                Environment::with_prefix(ENV_PREFIX)
                    .separator("__")
                    .try_parsing(true)
            );
        self
    }

    pub async fn build(self) -> Result<T, ConfigError> {
        let config = self.builder
            .build()
            .await?
            .try_deserialize::<T>()?;

        config.validate()?;
        
        Ok(config)
    }
}

impl<T> Default for AppConfigBuilder<T>
where
    T: DeserializeOwned + Validate,
{
    fn default() -> Self {
        Self::new()
    }
}