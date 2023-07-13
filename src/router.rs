/* SPDX-License-Identifier: CC0-1.0
 *
 * src/router.rs
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

use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Error, Result};
use async_fred_session::{
    fred::{pool::RedisPool, types::RedisConfig},
    RedisSessionStore,
};
use axum::{
    error_handling::HandleErrorLayer,
    middleware::map_response,
    routing::{get, post},
    BoxError, Router,
};
use axum_client_ip::SecureClientIpSource;
use axum_login::{
    axum_sessions::SessionLayer, memory_store::MemoryStore as AuthMemoryStore, AuthLayer, AuthUser,
};
use tokio::{sync::RwLock, time::Duration};
use tower::ServiceBuilder;
use tower_governor::{errors::display_error, governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::{
    compression::CompressionLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use crate::{
    controllers::{
        accept_form, admin_handler, delete_handler, get_shady, handle_timeout_error, login_handler,
        login_page_handler, logout_handler, root, transform_error, RequireAuth, User,
    },
    loadenv::EnvVars,
    AppState,
};

async fn create_session_layer(env: &EnvVars) -> Result<SessionLayer<RedisSessionStore>> {
    let redis_config = RedisConfig::from_url(env.redis_url())
        .map_err(|e| Error::new(e).context("Failed to parse Redis URL"))?;
    let rds_pool = RedisPool::new(redis_config, None, None, 6).unwrap();
    rds_pool.connect();
    rds_pool
        .wait_for_connect()
        .await
        .map_err(|e| Error::new(e).context("Could not connect to redis"))?;

    let cookie_domain = env
        .hostname()
        .split(':')
        .next()
        .ok_or(anyhow!("Failed to parse cookie domain"))?;
    let session_store = RedisSessionStore::from_pool(rds_pool, Some("async-fred-session/".into()));
    Ok(SessionLayer::new(session_store, env.secret()).with_cookie_domain(cookie_domain))
}

async fn create_auth_layer(
    env: &EnvVars,
    user: &User,
) -> AuthLayer<AuthMemoryStore<usize, User>, usize, User> {
    let store = Arc::new(RwLock::new(HashMap::default()));

    store.write().await.insert(user.get_id(), user.clone());

    let user_store = AuthMemoryStore::new(&store);
    AuthLayer::new(user_store, env.secret())
}

pub(crate) async fn get_router(env: &EnvVars, state: AppState) -> Result<Router> {
    let session_layer = create_session_layer(env).await?;
    let auth_layer = create_auth_layer(env, &state.user).await;

    let services = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(HandleErrorLayer::new(handle_timeout_error))
        .timeout(Duration::from_secs(10))
        .layer(SecureClientIpSource::RightmostForwarded.into_extension())
        .layer(map_response(transform_error))
        .layer(session_layer)
        .layer(auth_layer);

    let governor_default_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(5)
            .finish()
            .unwrap(),
    );

    let governor_login_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_second(5)
            .burst_size(3)
            .finish()
            .unwrap(),
    );

    let services_governor_regular = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            display_error(e)
        }))
        .layer(GovernorLayer {
            // We can leak this because it is created once
            config: Box::leak(governor_default_conf),
        });

    let services_governor_login = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            display_error(e)
        }))
        .layer(GovernorLayer {
            config: Box::leak(governor_login_conf),
        });

    let login_router = Router::new()
        .route("/login", get(login_page_handler).post(login_handler))
        .route_layer(services_governor_login)
        .route("/logout", get(logout_handler))
        .route_layer(services_governor_regular.clone());

    let admin_router = Router::new()
        .route("/admin", get(admin_handler).post(delete_handler))
        .route_layer(RequireAuth::login());

    let shady_router = Router::new()
        .route("/makeurl", post(accept_form))
        .nest_service("/robots.txt", ServeFile::new("static/robots.txt"))
        .nest_service("/favicon.ico", ServeFile::new("static/favicon.ico"))
        .nest_service("/assets", ServeDir::new("static/assets"))
        .route("/*shady", get(get_shady))
        .route("/", get(root))
        .route_layer(services_governor_regular);

    Ok(Router::new()
        .merge(login_router)
        .merge(admin_router)
        .merge(shady_router)
        .layer(services)
        .with_state(state))
}
