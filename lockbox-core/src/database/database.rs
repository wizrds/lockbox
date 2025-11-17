use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
    pin::Pin,
};
use log::LevelFilter;
use futures::future::try_join_all;
use serde::{Serialize, Deserialize};

use lockbox_migrations::{Migrator, MigratorTrait};
use crate::database::{
    orm::{
        Database as SeaORMDatabase,
        DatabaseBackend,
        DatabaseConnection,
        ConnectOptions,
        ConnectionTrait,
        DatabaseTransaction,
        TransactionTrait,
        TransactionError
    },
    errors::{DatabaseResult, DatabaseError},
};


#[derive(Debug)]
pub struct Database {
    primary: DatabaseConnection,
    replicas: Vec<DatabaseConnection>,
    replica_index: AtomicUsize,
}

impl Database {
    pub fn new(primary: DatabaseConnection, replicas: Vec<DatabaseConnection>) -> Self {
        Self {
            primary,
            replicas,
            replica_index: AtomicUsize::new(0),
        }
    }

    pub fn backend(&self) -> DatabaseBackend {
        self.primary.get_database_backend()
    }

    pub fn get_write_connection(&self) -> &DatabaseConnection {
        &self.primary
    }

    pub fn get_read_connection(&self) -> &DatabaseConnection {
        if self.replicas.is_empty() {
            return self.get_write_connection();
        }

        &self.replicas[self.replica_index.fetch_add(1, Ordering::SeqCst) % self.replicas.len()]
    }

    pub async fn run_migrations<M: MigratorTrait>(&self) -> DatabaseResult<()> {
        M::up(&self.primary, None)
            .await
            .map_err(DatabaseError::from)
    }

    pub async fn rw_transaction<F, T>(&self, f: F) -> DatabaseResult<T>
    where
        F: for<'b> FnOnce(&'b DatabaseTransaction) -> Pin<Box<dyn Future<Output = DatabaseResult<T>> + Send + 'b>> + Send,
        T: Send,
    {
        match self.get_write_connection()
            .transaction(f)
            .await
        {
            Ok(result) => Ok(result),
            Err(e) => match e {
                TransactionError::Connection(conn_err) => Err(DatabaseError::DatabaseError(conn_err)),
                TransactionError::Transaction(txn_err) => Err(txn_err),
            }
        }
    }

    pub async fn ro_transaction<F, T>(&self, f: F) -> DatabaseResult<T>
    where
        F: for<'b> FnOnce(&'b DatabaseTransaction) -> Pin<Box<dyn Future<Output = DatabaseResult<T>> + Send + 'b>> + Send,
        T: Send,
    {
        match self.get_read_connection()
            .transaction(f)
            .await
        {
            Ok(result) => Ok(result),
            Err(e) => match e {
                TransactionError::Connection(conn_err) => Err(DatabaseError::DatabaseError(conn_err)),
                TransactionError::Transaction(txn_err) => Err(txn_err),
            }
        }
    }

    pub fn builder() -> DatabaseBuilder {
        DatabaseBuilder::new()
    }
}


#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DatabaseOptions {
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
    pub connect_timeout: Option<Duration>,
    pub idle_timeout: Option<Duration>,
    pub acquire_timeout: Option<Duration>,
    pub max_lifetime: Option<Duration>,
    pub enable_logging: bool,
}

impl DatabaseOptions {
    pub fn new() -> Self {
        Self {
            max_connections: None,
            min_connections: None,
            connect_timeout: None,
            idle_timeout: None,
            acquire_timeout: None,
            max_lifetime: None,
            enable_logging: true,
        }
    }

    pub fn with_max_connections(&mut self, max: impl Into<u32>) -> &mut Self {
        self.max_connections = Some(max.into());
        self
    }

    pub fn with_min_connections(&mut self, min: impl Into<u32>) -> &mut Self {
        self.min_connections = Some(min.into());
        self
    }

    pub fn with_connect_timeout(&mut self, timeout: impl Into<Duration>) -> &mut Self {
        self.connect_timeout = Some(timeout.into());
        self
    }

    pub fn with_idle_timeout(&mut self, timeout: impl Into<Duration>) -> &mut Self {
        self.idle_timeout = Some(timeout.into());
        self
    }

    pub fn with_acquire_timeout(&mut self, timeout: impl Into<Duration>) -> &mut Self {
        self.acquire_timeout = Some(timeout.into());
        self
    }

    pub fn with_max_lifetime(&mut self, lifetime: impl Into<Duration>) -> &mut Self {
        self.max_lifetime = Some(lifetime.into());
        self
    }

    pub fn with_logging(&mut self, enable: bool) -> &mut Self {
        self.enable_logging = enable;
        self
    }

    pub fn build(self, dsn: &str) -> ConnectOptions {
        let mut options = ConnectOptions::new(dsn);

        if let Some(max) = self.max_connections {
            options.max_connections(max);
        }
        if let Some(min) = self.min_connections {
            options.min_connections(min);
        }
        if let Some(timeout) = self.connect_timeout {
            options.connect_timeout(timeout);
        }
        if let Some(timeout) = self.idle_timeout {
            options.idle_timeout(timeout);
        }
        if let Some(timeout) = self.acquire_timeout {
            options.acquire_timeout(timeout);
        }
        if let Some(lifetime) = self.max_lifetime {
            options.max_lifetime(lifetime);
        }

        options
            .sqlx_logging(self.enable_logging)
            .sqlx_logging_level(LevelFilter::Debug);

        options
    }
}


pub struct DatabaseBuilder {
    primary: Option<String>,
    replicas: Option<Vec<String>>,
    options: Option<DatabaseOptions>,
    run_migrations: bool,
}

impl DatabaseBuilder {
    pub fn new() -> Self {
        Self {
            primary: None,
            replicas: None,
            options: None,
            run_migrations: false,
        }
    }

    pub fn with_primary(mut self, primary: impl Into<String>) -> Self {
        self.primary = Some(primary.into());
        self
    }

    pub fn with_replicas(mut self, replicas: Vec<impl Into<String>>) -> Self {
        if replicas.is_empty() {
            return self;
        }

        self.replicas = Some(replicas.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_options(mut self, options: DatabaseOptions) -> Self {
        self.options = Some(options);
        self
    }

    pub fn with_migrations(mut self, run: bool) -> Self {
        self.run_migrations = run;
        self
    }

    pub async fn build(self) -> DatabaseResult<Database> {
        let primary = self.primary.ok_or(DatabaseError::MissingPrimaryDsn)?;
        let replicas = self.replicas.unwrap_or_default();
        let options = self.options.unwrap_or_default();

        let database = Database::new(
            SeaORMDatabase::connect(options.clone().build(&primary))
                .await
                .map_err(DatabaseError::from)?,
            try_join_all(replicas.into_iter()
                .map(|dsn| SeaORMDatabase::connect(options.clone().build(&dsn)))
                .collect::<Vec<_>>())
                .await
                .map_err(DatabaseError::from)?,
        );

        if self.run_migrations {
            database.run_migrations::<Migrator>().await?;
        }

        Ok(database)
    }
}