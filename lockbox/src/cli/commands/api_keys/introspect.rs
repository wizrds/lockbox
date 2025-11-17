use async_trait::async_trait;
use anyhow::{Result, anyhow};
use clap::Args;
use validator::Validate;

use lockbox_core::service::api_key::ApiKeyServiceTrait;

use crate::{
    cli::{context::Ctx, traits::Cmd, utils::{NullableString, validate_vec_nullable_string_len}},
    manifest::{to_serialized_string_pretty, ManifestFormat},
};

#[derive(Clone, Args, Debug, Validate)]
pub struct CliCommandApiKeysIntrospect {
    #[arg(help = "The API Key")]
    pub api_key: String,
    #[arg(long, help = "The required scope")]
    pub scope: Option<String>,
    #[arg(
        long,
        help = "Only allow keys with one of these tags",
    )]
    #[validate(custom(function = validate_vec_nullable_string_len::<6>))]
    pub tags: Vec<NullableString>,
    #[arg(
        long,
        help = "Require the key to have no tags (conflicts with --tags)",
        default_value_t = false,
        conflicts_with = "tags"
    )]
    pub no_tag: bool,
    #[arg(long, short, help = "The serialization format", default_value = "json")]
    pub format: ManifestFormat,
}

#[async_trait]
impl Cmd for CliCommandApiKeysIntrospect {
    async fn update_ctx(&self, ctx: &mut Ctx) -> Result<()> {
        if let Err(err) = self.validate() {
            return Err(anyhow!("Validation error: {}", err));
        }

        let api_keys_ctx = ctx.api_keys.as_mut().unwrap();

        api_keys_ctx.api_key = Some(self.api_key.clone());
        api_keys_ctx.scope = self.scope.clone();
        api_keys_ctx.required_tags = match self.tags.is_empty() {
            true => None,
            false => Some(
                self.tags
                    .iter()
                    .map(|t| t.as_inner().clone())
                    .collect()
            ),
        };
        api_keys_ctx.no_tag = self.no_tag;
        api_keys_ctx.format = Some(self.format.clone());

        if api_keys_ctx.no_tag && api_keys_ctx.required_tags.is_some() {
            return Err(anyhow!("Conflicting options: --no-tag and --tag are mutually exclusive"));
        }

        Ok(())
    }

    async fn run(&self, ctx: &mut Ctx) -> Result<()> {
        let api_keys_ctx = ctx.api_keys.as_ref().unwrap();

        let introspection = api_keys_ctx
            .api_key_service
            .as_ref()
            .unwrap()
            .introspect_api_key(
                api_keys_ctx.api_key.clone().unwrap(),
                api_keys_ctx.scope.clone(),
                match api_keys_ctx.no_tag {
                    true => Some(vec![]),
                    false
                        if api_keys_ctx.required_tags
                            .as_ref()
                            .filter(|tags| !tags.is_empty())
                            .is_some() => Some(api_keys_ctx.required_tags.clone().unwrap()),
                    false => None,
                },
                api_keys_ctx.tenant_id.clone(),
            )
            .await?;

        println!("{}", to_serialized_string_pretty(&introspection, api_keys_ctx.format.clone().unwrap())?);

        Ok(())
    }
}