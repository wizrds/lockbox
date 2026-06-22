use async_trait::async_trait;
use anyhow::{Result, anyhow};
use clap::{Args, ArgAction};
use uuid::Uuid;
use validator::Validate;

use lockbox_core::service::api_key::{ApiKeyServiceTrait, FindApiKeyParams};

use crate::{
    cli::{context::Ctx, traits::Cmd, utils::{parse_key_value, validate_vec_string_len}},
    manifest::{to_serialized_string_pretty, ManifestFormat},
};

#[derive(Clone, Args, Debug, Validate)]
pub struct CliCommandApiKeysFind {
    #[arg(long, help = "The page number", default_value = "1")]
    pub page: u64,
    #[arg(long, help = "The number of items per page", default_value = "10")]
    pub per_page: u64,
    #[arg(long, help = "API key IDs to include")]
    pub include_ids: Option<Vec<String>>,
    #[arg(long, help = "API key IDs to exclude")]
    pub exclude_ids: Option<Vec<String>>,
    #[arg(long, help = "The tenant IDs to filter")]
    pub tenant_ids: Option<Vec<String>>,
    #[arg(long, help = "The namespaces to filter")]
    #[validate(custom(function = validate_vec_string_len::<6>))]
    pub namespaces: Option<Vec<String>>,
    #[arg(long, help = "The tags to filter")]
    #[validate(custom(function = validate_vec_string_len::<6>))]
    pub tags: Option<Vec<String>>,
    #[arg(long, help = "The short keys to filter")]
    pub short_keys: Option<Vec<String>>,
    #[arg(long, help = "The owners to filter")]
    pub owners: Option<Vec<String>>,
    #[arg(long, action = ArgAction::Set, help = "Filter by revoked API keys")]
    pub revoked: Option<bool>,
    #[arg(
        long,
        help = "Additional metadata to store with the API key (e.g., --metadata key1=value1 --metadata key2=value2)",
        value_parser = parse_key_value
    )]
    pub metadata: Option<Vec<(String, String)>>,
    #[arg(long, short, help = "The serialization format", default_value = "json")]
    pub format: ManifestFormat,
}

#[async_trait]
impl Cmd for CliCommandApiKeysFind {
    async fn update_ctx(&self, ctx: &mut Ctx) -> Result<()> {
        if let Err(err) = self.validate() {
            return Err(anyhow!("Validation error: {}", err))
        }

        let api_keys_ctx = ctx.api_keys.as_mut().unwrap();

        api_keys_ctx.page = self.page;
        api_keys_ctx.per_page = self.per_page;
        api_keys_ctx.include_ids = self.include_ids
            .as_ref()
            .map(|ids| ids
                .into_iter()
                .map(|id| id.parse::<Uuid>())
                .collect::<Result<Vec<Uuid>, _>>()
            )
            .transpose()?;
        api_keys_ctx.exclude_ids = self.exclude_ids
            .as_ref()
            .map(|ids| ids
                .into_iter()
                .map(|id| id.parse::<Uuid>())
                .collect::<Result<Vec<Uuid>, _>>()
            )
            .transpose()?;
        api_keys_ctx.tenant_ids = self.tenant_ids.clone();
        api_keys_ctx.namespaces = self.namespaces.clone();
        api_keys_ctx.tags = self.tags.clone();
        api_keys_ctx.short_keys = self.short_keys.clone();
        api_keys_ctx.owners = self.owners.clone();
        api_keys_ctx.revoked = self.revoked.clone();
        api_keys_ctx.metadata = self.metadata.clone();
        api_keys_ctx.format = Some(self.format.clone());

        Ok(())
    }

    async fn run(&self, ctx: &mut Ctx) -> Result<()> {
        let api_keys_ctx = ctx.api_keys.as_ref().unwrap();

        let api_keys = api_keys_ctx
            .api_key_service
            .as_ref()
            .unwrap()
            .find_api_keys(
                FindApiKeyParams {
                    page: api_keys_ctx.page,
                    per_page: api_keys_ctx.per_page,
                    include_ids: api_keys_ctx.include_ids.clone(),
                    exclude_ids: api_keys_ctx.exclude_ids.clone(),
                    tenant_ids: api_keys_ctx.tenant_ids.clone(),
                    namespaces: api_keys_ctx.namespaces.clone(),
                    tags: api_keys_ctx.tags.clone(),
                    short_keys: api_keys_ctx.short_keys.clone(),
                    owners: api_keys_ctx.owners.clone(),
                    revoked: api_keys_ctx.revoked.clone(),
                    metadata: match api_keys_ctx.metadata.clone() {
                        Some(val) => Some(val.into_iter().collect()),
                        None => None,
                    },
                    created_after: None,
                    created_before: None,
                },
            )
            .await?;

        println!("{}", to_serialized_string_pretty(&api_keys, api_keys_ctx.format.clone().unwrap())?);

        Ok(())
    }
}