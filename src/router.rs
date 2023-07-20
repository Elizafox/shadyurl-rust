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
    Router,
};
use axum_csrf::{CsrfConfig, CsrfLayer, Key};
use axum_login::{
    axum_sessions::SessionLayer, memory_store::MemoryStore as AuthMemoryStore, AuthLayer, AuthUser,
};
use tokio::{sync::RwLock, time::Duration};
use tower::ServiceBuilder;
use tower_http::services::{ServeDir, ServeFile};

use crate::{
    auth::{RequireAuth, User},
    controllers::{
        accept_form, admin_handler, delete_handler, get_shady, handle_timeout_error, login_handler,
        login_page_handler, logout_handler, root, transform_error,
    },
    loadenv::EnvVars,
    AppState,
};

async fn create_session_layer(env: &EnvVars) -> Result<SessionLayer<RedisSessionStore>> {
    let redis_config = RedisConfig::from_url(env.redis_url.as_str())
        .map_err(|e| Error::new(e).context("Failed to parse Redis URL"))?;
    let rds_pool = RedisPool::new(redis_config, None, None, 6).unwrap();
    rds_pool.connect();
    rds_pool
        .wait_for_connect()
        .await
        .map_err(|e| Error::new(e).context("Could not connect to redis"))?;

    let cookie_domain = env
        .hostname
        .split(':')
        .next()
        .ok_or(anyhow!("Failed to parse cookie domain"))?;
    let session_store = RedisSessionStore::from_pool(rds_pool, Some("async-fred-session/".into()));
    Ok(SessionLayer::new(session_store, &env.secret_key).with_cookie_domain(cookie_domain))
}

async fn create_auth_layer(
    env: &EnvVars,
    user: &User,
) -> AuthLayer<AuthMemoryStore<usize, User>, usize, User> {
    let store = Arc::new(RwLock::new(HashMap::default()));

    store.write().await.insert(user.get_id(), user.clone());

    let user_store = AuthMemoryStore::new(&store);
    AuthLayer::new(user_store, &env.secret_key)
}

pub(crate) async fn get_router(env: &EnvVars, state: AppState) -> Result<Router> {
    let session_layer = create_session_layer(env).await?;
    let auth_layer = create_auth_layer(env, &state.user).await;

    let cookie_key = Key::from(&env.secret_key);
    let csrf_config = CsrfConfig::default().with_key(Some(cookie_key));

    let services = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_timeout_error))
        .timeout(Duration::from_secs(10))
        .layer(env.ip_source.clone().into_extension())
        .layer(map_response(transform_error))
        .layer(CsrfLayer::new(csrf_config))
        .layer(session_layer)
        .layer(auth_layer);

    let login_router = Router::new()
        .route("/login", get(login_page_handler).post(login_handler))
        .route("/logout", get(logout_handler));

    let admin_router = Router::new()
        .route("/admin", get(admin_handler).post(delete_handler))
        .route_layer(RequireAuth::login());

    let shady_router = Router::new()
        .route("/makeurl", post(accept_form))
        .nest_service("/robots.txt", ServeFile::new("static/robots.txt"))
        .nest_service("/favicon.ico", ServeFile::new("static/favicon.ico"))
        .nest_service("/assets", ServeDir::new("static/assets"))
        .route("/*shady", get(get_shady))
        .route("/", get(root));

    Ok(Router::new()
        .merge(login_router)
        .merge(admin_router)
        .merge(shady_router)
        .layer(services)
        .with_state(state))
}
