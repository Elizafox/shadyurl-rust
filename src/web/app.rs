/* SPDX-License-Identifier: CC0-1.0
 *
 * src/web/app.rs
 *
 * This file is a component of ShadyURL by Elizabeth Myers.
 *
 * To the extent possible under law, the person who associated CC0 with
 * ShadyURL has waived all copyright and related or neighboring rights
 * to ShadyURL.
 *
 * You should have received a copy of the CC0 legalcode along with this
 * work.  If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
 */

// App entrypoint stuff

use std::{net::SocketAddr, sync::Arc};

use axum::Router;
use axum_login::AuthManagerLayerBuilder;
use axum_messages::MessagesManagerLayer;
use sea_orm::ConnectOptions;
use time::Duration;
use tokio::task::JoinHandle;
use tower::ServiceBuilder;
use tower_http::{normalize_path::NormalizePathLayer, timeout::TimeoutLayer};
use tower_sessions::{cookie::Key, CachingSessionStore, Expiry, SessionManagerLayer};
use tower_sessions_moka_store::MokaStore;
use tower_sessions_redis_store::{fred::prelude::*, RedisStore};
use tracing::info;

use service::Database;

use crate::{
    auth::Backend,
    bancache::BanCache,
    env::Vars,
    state::AppState,
    urlcache::UrlCache,
    web::{admin, fallback, files, submission, url},
};

// This holds our app state that we need later
pub struct App {
    state: AppState,
    redis_pool: RedisPool,
    redis_conn: JoinHandle<Result<(), RedisError>>,
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error(transparent)]
    Env(#[from] crate::env::EnvError),

    #[error(transparent)]
    UrlCache(#[from] crate::urlcache::UrlCacheError),

    #[error(transparent)]
    Db(#[from] sea_orm::DbErr),

    #[error(transparent)]
    RedisStore(#[from] tower_sessions_redis_store::RedisStoreError),

    #[error(transparent)]
    RedisFred(#[from] tower_sessions_redis_store::fred::error::RedisError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    TaskJoin(#[from] tokio::task::JoinError),
}

impl App {
    // Load configuration and begin initalising DB connections, etc.
    pub(crate) async fn new() -> Result<Self, RuntimeError> {
        let env = Vars::load_env()?;

        let redis_config = RedisConfig::from_url(&env.redis_url)?;

        // TODO: more configuration for redis?
        let redis_pool = RedisPool::new(redis_config, None, None, None, env.redis_pool_size)?;
        let redis_conn = redis_pool.connect();
        redis_pool.wait_for_connect().await?;

        // TODO: more connection options?
        let mut opt = ConnectOptions::new(&env.database_url);
        opt.max_connections(100)
            .min_connections(5)
            .sqlx_logging(false);
        let db = Arc::new(Database::get_with_connect_options(opt).await?);

        // TODO: configurable
        let bancache = BanCache::new(
            db.clone(),
            env.ban_cache_max_entries,
            env.ban_cache_ttl,
            env.ban_cache_idle,
        );
        let urlcache = UrlCache::new(
            db.clone(),
            env.url_cache_max_entries,
            env.url_cache_ttl,
            env.url_cache_idle,
        )
        .await?;

        Ok(Self {
            state: AppState {
                db: db.clone(),
                env,
                bancache,
                urlcache,
            },
            redis_pool,
            redis_conn,
        })
    }

    // Begin serving
    pub(crate) async fn serve(self) -> Result<(), RuntimeError> {
        let redis_store = RedisStore::new(self.redis_pool);
        let moka_store = MokaStore::new(Some(100));
        let caching_store = CachingSessionStore::new(moka_store, redis_store);
        let session_layer = SessionManagerLayer::new(caching_store)
            .with_expiry(Expiry::OnInactivity(Duration::days(1)))
            .with_private(Key::from(&self.state.env.csrf_key));

        let backend = Backend::new(self.state.db.clone());
        let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

        let services = ServiceBuilder::new()
            .layer(TimeoutLayer::new(Duration::seconds(15).unsigned_abs()))
            .layer(NormalizePathLayer::trim_trailing_slash())
            .layer(auth_layer)
            .layer(MessagesManagerLayer)
            .layer(self.state.env.ip_source.clone().into_extension());

        let bind = self.state.env.bind.clone();

        let app = Router::new()
            .merge(admin::router())
            .merge(files::router())
            .merge(submission::router())
            .merge(url::router())
            .merge(fallback::router())
            .layer(services)
            .with_state(self.state);

        info!("Preparing to serve on {bind}");
        let listener = tokio::net::TcpListener::bind(bind).await?;
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await?;

        self.redis_conn.await??;

        info!("Server terminating");

        Ok(())
    }
}
