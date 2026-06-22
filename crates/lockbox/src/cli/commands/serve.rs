use std::{sync::Arc, time::Duration};
use async_trait::async_trait;
use anyhow::Result;
use clap::{Args, ArgAction};
use jobq::{BatchJobQueueSystemBuilder, BatchJobWorkerOptions};
use tokio::spawn;

use lockbox_core::{
    telemetry::info,
    database::DatabaseBuilder,
    service::{api_key::{ApiKeyService, ApiKeyServiceTrait}, errors::ApiKeyServiceError},
    repository::{
        api_key::ApiKeyRepository,
        api_key_namespace::ApiKeyNamespaceRepository,
        api_key_tag::ApiKeyTagRepository,
    },
    tasks::TaskOperations,
};
use lockbox_api::{server::{create_router, serve}, state::ApiState};

use crate::cli::{traits::Cmd, context::{Ctx, CtxServe, CtxTaskQueue}};


#[derive(Clone, Args, Debug)]
pub struct CliCommandServe {
    #[arg(long)]
    pub host: Option<String>,
    #[arg(long)]
    pub port: Option<u16>,
    #[arg(long, action = ArgAction::SetTrue)]
    pub migrations: bool,
}

#[async_trait]
impl Cmd for CliCommandServe {
    async fn update_ctx(&self, ctx: &mut Ctx) -> Result<()> {
        ctx.serve = Some(CtxServe {
            host: Some(self.host.clone().unwrap_or_else(|| ctx.app_config.server.host.clone())),
            port: Some(self.port.unwrap_or_else(|| ctx.app_config.server.port)),
            task_queue: CtxTaskQueue {
                worker_count: Some(ctx.app_config.task_queue.worker_count),
                max_queue_capacity: Some(ctx.app_config.task_queue.max_queue_capacity),
                batch_size: Some(ctx.app_config.task_queue.batch_size),
                batch_timeout_ms: Some(ctx.app_config.task_queue.batch_timeout_ms),
            },
            migrations: self.migrations,
        });

        Ok(())
    }

    async fn run(&self, ctx: &mut Ctx) -> Result<()> {
        ctx.telemetry.enable_logging();

        info!(
            event = "Starting",
            version = env!("CARGO_PKG_VERSION"),
        );

        // Create the dependencies
        let serve_ctx = ctx.serve.as_ref().unwrap();
        let (job_queue, worker_pool) = {
            BatchJobQueueSystemBuilder::<TaskOperations, _>::fifo(serve_ctx.task_queue.max_queue_capacity.unwrap())
                .with_num_workers(serve_ctx.task_queue.worker_count.unwrap())
                .with_worker_options(BatchJobWorkerOptions {
                    batch_size: serve_ctx.task_queue.batch_size.unwrap(),
                    batch_timeout: Duration::from_millis(serve_ctx.task_queue.batch_timeout_ms.unwrap() as u64)
                })
                .build()
        };
        let database = Arc::new(
            DatabaseBuilder::new()
                .with_primary(ctx.app_config.database.primary.clone())
                .with_replicas(ctx.app_config.database.replicas.clone())
                .with_options(ctx.app_config.database.options.clone())
                .with_migrations(serve_ctx.migrations)
                .build()
                .await?
        );
        let api_key_service = Arc::new(
            ApiKeyService::builder()
                .with_key_repo(ApiKeyRepository::new(database.clone()))
                .with_namespace_repo(ApiKeyNamespaceRepository::new(database.clone()))
                .with_tag_repo(ApiKeyTagRepository::new(database.clone()))
                .build()
        );
        let state = Arc::new(
            ApiState::builder()
                .with_default_tenant_id(&ctx.app_config.default_tenant_id)
                .with_default_namespace(&ctx.app_config.default_namespace)
                .with_job_queue_arc(job_queue.clone())
                .with_database_arc(database.clone())
                .with_api_key_service_arc(api_key_service.clone())
                .build()
        );
        let addr = format!(
            "{}:{}",
            serve_ctx.host.clone().unwrap(),
            serve_ctx.port.clone().unwrap()
        );
        let worker_pool_clone = worker_pool.clone();
        let worker_pool_handle = spawn(async move {
            worker_pool_clone.run().await;
        });

        // Bootstrap the default namespace to ensure we always have at least one namespace
        match api_key_service
            .get_namespace(ctx.app_config.default_namespace.clone(), ctx.app_config.default_tenant_id.clone())
            .await {
            Ok(ns) => {
                if !ns.is_default {
                    api_key_service
                        .set_default_namespace(ctx.app_config.default_namespace.clone(), ctx.app_config.default_tenant_id.clone())
                        .await?;
                }
            },
            Err(ApiKeyServiceError::NamespaceNotFound) => {
                api_key_service
                    .create_namespace(ctx.app_config.default_namespace.clone(), ctx.app_config.default_tenant_id.clone())
                    .await?;
                api_key_service
                    .set_default_namespace(ctx.app_config.default_namespace.clone(), ctx.app_config.default_tenant_id.clone())
                    .await?;
            },
            Err(e) => return Err(e.into())
        };

        // Start the server
        info!(event = "Listening", address = addr.as_str());
        serve(addr, create_router(state.clone())).await?;
        info!(event = "Stopping");

        worker_pool.shutdown().await;
        worker_pool_handle.await.unwrap();

        Ok(())
    }
}