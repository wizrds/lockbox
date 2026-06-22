use async_trait::async_trait;
use anyhow::Result;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use clap::{Args, ArgAction};

use lockbox_core::service::{api_key::ApiKeyServiceTrait, errors::ApiKeyServiceError};

use crate::cli::{context::Ctx, traits::Cmd, utils::parse_nl_duration};


#[derive(Clone, Args, Debug)]
pub struct CliCommandApiKeysSetExpiration {
    #[arg(help = "The API Key ID")]
    pub id: String,
    #[arg(long, help = "The new expiration date (e.g. 15m, 2h, 1d2h)", value_parser = parse_nl_duration)]
    pub expiration: DateTime<Utc>,
    #[arg(long, action = ArgAction::SetTrue, help = "Skip if the API key does not exist")]
    pub skip_missing: bool,
}

#[async_trait]
impl Cmd for CliCommandApiKeysSetExpiration {
    async fn update_ctx(&self, ctx: &mut Ctx) -> Result<()> {
        let api_keys_ctx = ctx.api_keys.as_mut().unwrap();

        api_keys_ctx.id = Some(self.id.clone().parse::<Uuid>()?);
        api_keys_ctx.expiration = Some(self.expiration.clone());
        api_keys_ctx.skip_missing = self.skip_missing;

        Ok(())
    }

    async fn run(&self, ctx: &mut Ctx) -> Result<()> {
        let api_keys_ctx = ctx.api_keys.as_ref().unwrap();

        match api_keys_ctx
            .api_key_service
            .as_ref()
            .unwrap()
            .set_api_key_expiration(
                api_keys_ctx.id.clone().unwrap(),
                api_keys_ctx.expiration.clone().unwrap(),
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