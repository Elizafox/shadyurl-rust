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

use std::{net::SocketAddr, sync::Arc};

use aes_gcm_siv::aead::KeyInit;
use axum::Router;
use axum_login::AuthManagerLayerBuilder;
use axum_messages::MessagesManagerLayer;
use sea_orm::ConnectOptions;
use time::Duration;
use tokio::task::JoinHandle;
use tower::ServiceBuilder;
use tower_http::{normalize_path::NormalizePathLayer, timeout::TimeoutLayer};
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_redis_store::{fred::prelude::*, RedisStore};
use tracing::info;

use migration::{Migrator, MigratorTrait};
use service::Database;

use crate::{
    auth::Backend,
    bancache::BanCache,
    csrf::CryptoEngine,
    env::Vars,
    state::AppState,
    web::{admin, fallback, files, submission, url},
};

pub struct App {
    state: AppState,
    redis_pool: RedisPool,
    redis_conn: JoinHandle<Result<(), RedisError>>,
}

impl App {
    pub(crate) async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let env = Vars::load_env()?;

        let redis_config = RedisConfig::from_url(&env.redis_url)?;

        let redis_pool = RedisPool::new(redis_config, None, None, None, env.redis_pool_size)?;
        let redis_conn = redis_pool.connect();
        redis_pool.wait_for_connect().await?;

        let mut opt = ConnectOptions::new(&env.database_url);
        opt.max_connections(100)
            .min_connections(5)
            .sqlx_logging(false);
        let db = Arc::new(Database::get_with_connect_options(opt).await?);

        Migrator::up(db.as_ref(), None).await?;

        let bancache = BanCache::new(db.clone());

        let csrf_crypto_engine = CryptoEngine::new(&env.csrf_key.into());

        Ok(Self {
            state: AppState {
                db: db.clone(),
                env,
                bancache,
                csrf_crypto_engine,
            },
            redis_pool,
            redis_conn,
        })
    }

    pub(crate) async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        let session_store = RedisStore::new(self.redis_pool);
        let session_layer = SessionManagerLayer::new(session_store)
            .with_expiry(Expiry::OnInactivity(Duration::days(1)));

        let backend = Backend::new(self.state.db.clone());
        let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer.clone()).build();

        let services = ServiceBuilder::new()
            .layer(TimeoutLayer::new(Duration::seconds(15).unsigned_abs()))
            .layer(NormalizePathLayer::trim_trailing_slash())
            .layer(auth_layer)
            .layer(session_layer)
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
