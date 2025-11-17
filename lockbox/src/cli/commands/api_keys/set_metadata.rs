use std::collections::HashMap;
use async_trait::async_trait;
use anyhow::{Result, anyhow};
use uuid::Uuid;
use clap::{Args, ArgAction};

use lockbox_core::service::{api_key::ApiKeyServiceTrait, errors::ApiKeyServiceError};

use crate::cli::{context::Ctx, traits::Cmd, utils::parse_key_value};


#[derive(Clone, Args, Debug)]
pub struct CliCommandApiKeysSetMetadata {
    #[arg(help = "The API key ID")]
    pub id: String,
    #[arg(
        long,
        help = "Additional metadata to store with the API key (e.g., --metadata key1=value1 --metadata key2=value2)",
        value_parser = parse_key_value
    )]
    pub metadata: Vec<(String, String)>,
    #[arg(long, action = ArgAction::SetTrue, help = "Skip if the API key does not exist")]
    pub skip_missing: bool,
}

#[async_trait]
impl Cmd for CliCommandApiKeysSetMetadata {
    async fn update_ctx(&self, ctx: &mut Ctx) -> Result<()> {
        let api_keys_ctx = ctx.api_keys.as_mut().unwrap();

        api_keys_ctx.id = Some(self.id.clone().parse::<Uuid>()?);
        api_keys_ctx.metadata = Some(self.metadata.clone());
        api_keys_ctx.skip_missing = self.skip_missing;

        Ok(())
    }

    async fn run(&self, ctx: &mut Ctx) -> Result<()> {
        let api_keys_ctx = ctx.api_keys.as_ref().unwrap();

        if api_keys_ctx.metadata.is_none() || api_keys_ctx.metadata.as_ref().unwrap().is_empty() {
            return Err(anyhow!("No metadata provided"));
        }

        match api_keys_ctx
            .api_key_service
            .as_ref()
            .unwrap()
            .set_api_key_metadata(
                api_keys_ctx.id.clone().unwrap(),
                api_keys_ctx.metadata
                    .clone()
                    .map_or(
                        HashMap::new(),
                        |vec| vec
                            .into_iter()
                            .collect()
                    ),
                api_keys_ctx.tenant_id.clone(),
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(ApiKeyServiceError::KeyNotFound) if api_keys_ctx.skip_missing => Ok(()),
            Err(err) => return Err(err.into()),
        }
    }
}

