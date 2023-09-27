#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]
#![deny(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

//! Library which implements async session for deadpool redis pool

use async_session::{Session, SessionStore};
use deadpool_redis::redis::AsyncCommands;
use deadpool_redis::{Config, Connection, ConnectionInfo, Pool, PoolError};
pub use {async_session, deadpool_redis};

/// Error enum
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// error for non alphanumeric or underscore character
    #[error("only ascii alphanumeric and underscore is supported as prefix")]
    NonAlphaNumeric,
    /// dead pool redis build error
    #[error(transparent)]
    DeadpoolBuild(#[from] deadpool_redis::BuildError),
    /// dead pool redis config error
    #[error(transparent)]
    DeadPoolConfig(#[from] deadpool_redis::ConfigError),
}

/// Struct for deadpool pool store
#[derive(Clone)]
pub struct RedisSessionStore {
    pool: Pool,
    prefix: Option<String>,
}

impl std::fmt::Debug for RedisSessionStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisSessionStore")
            .field("pool", &self.pool.manager())
            .field("prefix", &self.prefix)
            .finish()
    }
}

impl RedisSessionStore {
    /// Create new deadpool redis store from redis pool
    #[must_use]
    pub fn new(pool: &Pool) -> Self {
        Self {
            pool: pool.clone(),
            prefix: None,
        }
    }

    /// Create new deadpool store from url
    ///
    /// # Errors
    /// When session store cannot be created from given url
    pub fn from_url(url: impl Into<String>) -> Result<Self, Error> {
        let pool = Config::from_url(url).builder()?.build()?;
        Ok(Self { pool, prefix: None })
    }

    /// Create new deadpool store from connection info
    ///
    /// # Errors
    /// When session store cannot be created from given connection info
    pub fn from_connection_info(connection_info: impl Into<ConnectionInfo>) -> Result<Self, Error> {
        let pool = Config::from_connection_info(connection_info)
            .builder()?
            .build()?;
        Ok(Self { pool, prefix: None })
    }

    /// Set prefix of pool consume redis session store and return new session
    /// store only alphanumeric or underscore is supported as prefix
    ///
    /// # Errors
    /// When passed prefix consists of non alphanumeric or underscore character
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Result<Self, Error> {
        let prefix = prefix.into();
        if !prefix
            .chars()
            .all(|c| char::is_ascii_alphanumeric(&c) || c == '_')
        {
            return Err(Error::NonAlphaNumeric);
        };
        self.prefix = Some(prefix);
        Ok(self)
    }

    /// Create value to session key which join
    fn key(&self, value: impl AsRef<str>) -> String {
        let value = value.as_ref().to_string();
        if let Some(p) = &self.prefix {
            format!("{p}/{value}")
        } else {
            value
        }
    }

    /// Get connection
    async fn connection(&self) -> Result<Connection, PoolError> {
        self.pool.get().await
    }
}

#[async_trait::async_trait]
impl SessionStore for RedisSessionStore {
    async fn load_session(&self, cookie_value: String) -> async_session::Result<Option<Session>> {
        let id = Session::id_from_cookie_value(&cookie_value)?;
        let mut conn = self.connection().await?;
        let value = conn.get::<_, Option<String>>(self.key(id)).await?;
        Ok(match value {
            Some(val) => serde_json::from_str(&val)?,
            None => None,
        })
    }

    async fn store_session(&self, session: Session) -> async_session::Result<Option<String>> {
        let key = self.key(session.id());
        let value = serde_json::to_string(&session)?;
        let mut conn = self.connection().await?;

        match session.expires_in() {
            Some(expiry) => {
                conn.set_ex(key, value, usize::try_from(expiry.as_secs())?)
                    .await?;
            }
            None => conn.set(key, value).await?,
        };

        Ok(session.into_cookie_value())
    }

    async fn destroy_session(&self, session: Session) -> async_session::Result {
        let key = self.key(session.id());
        let mut conn = self.connection().await?;
        conn.del(key).await?;
        Ok(())
    }

    async fn clear_store(&self) -> async_session::Result {
        let mut conn = self.connection().await?;
        match &self.prefix {
            Some(_) => {
                let keys = conn.keys::<_, Vec<String>>(self.key("*")).await?;
                if !keys.is_empty() {
                    conn.del(keys).await?;
                }
            }
            None => {
                deadpool_redis::redis::cmd("FLUSHDB")
                    .query_async(&mut conn)
                    .await?;
            }
        };
        Ok(())
    }
}
