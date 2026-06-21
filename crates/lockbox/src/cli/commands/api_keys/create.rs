use std::collections::HashMap;
use async_trait::async_trait;
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use clap::{Args, ArgAction};
use validator::Validate;

use lockbox_core::service::{api_key::ApiKeyServiceTrait, errors::ApiKeyServiceError, types::CreateApiKeyPayload};

use crate::{
    cli::{context::Ctx, traits::Cmd, utils::{parse_nl_duration, parse_key_value}},
    manifest::{to_serialized_string_pretty, ManifestFormat}
};


#[derive(Clone, Args, Debug, Validate)]
pub struct CliCommandApiKeysCreate {
    #[arg(long, help = "The owner of the API key")]
    pub owner: String,
    #[arg(long, help = "The scope of the API key")]
    pub scope: Option<String>,
    #[arg(long, help = "The optional tag")]
    #[validate(length(max = 6))]
    pub tag: Option<String>,
    #[arg(long, help = "The optional expiration date (e.g., 15m, 2h, 1d2h)", value_parser = parse_nl_duration)]
    pub expiration: Option<DateTime<Utc>>,
    #[arg(
        long,
        help = "Additional metadata to store with the API key (e.g., --metadata key1=value1 --metadata key2=value2)",
        value_parser = parse_key_value
    )]
    pub metadata: Option<Vec<(String, String)>>,
    #[arg(long, action = ArgAction::SetTrue, help = "Skip if the API key already exists")]
    pub skip_exists: bool,

    #[arg(long, short, help = "The serialization format", default_value = "json")]
    pub format: ManifestFormat,
}

#[async_trait]
impl Cmd for CliCommandApiKeysCreate {
    async fn update_ctx(&self, ctx: &mut Ctx) -> Result<()> {
        if let Err(err) = self.validate() {
            return Err(anyhow!("Validation error: {}", err));
        }
        
        let api_keys_ctx = ctx.api_keys.as_mut().unwrap();

        api_keys_ctx.owner = Some(self.owner.clone());
        api_keys_ctx.scope = self.scope.clone();
        api_keys_ctx.tag = self.tag.clone();
        api_keys_ctx.format = Some(self.format.clone());
        api_keys_ctx.expiration = self.expiration.clone();
        api_keys_ctx.metadata = self.metadata.clone();
        api_keys_ctx.skip_exists = self.skip_exists;

        Ok(())
    }

    async fn run(&self, ctx: &mut Ctx) -> Result<()> {
        let api_keys_ctx = ctx.api_keys.as_ref().unwrap();

        match api_keys_ctx
            .api_key_service
            .as_ref()
            .unwrap()
            .create_api_key(
                CreateApiKeyPayload {
                    owner: api_keys_ctx.owner.clone().expect("Missing owner"),
                    scope: api_keys_ctx.scope.clone(),
                    tag: api_keys_ctx.tag.clone(),
                    expires_at: api_keys_ctx.expiration.clone(),
                    metadata: api_keys_ctx.metadata
                        .clone()
                        .map_or(
                            HashMap::new(),
                            |vec| vec
                                .into_iter()
                                .collect()
                        )
                },
                api_keys_ctx.tenant_id.clone(),
            )
            .await
        {
            Ok(api_key) => {
                println!("{}", to_serialized_string_pretty(&api_key, api_keys_ctx.format.clone().unwrap())?);
                Ok(())
            },
            Err(ApiKeyServiceError::KeyAlreadyExists) if api_keys_ctx.skip_exists => Ok(()),
            Err(err) => return Err(err.into()),
        }
    }
}
