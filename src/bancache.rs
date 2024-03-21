/* SPDX-License-Identifier: CC0-1.0
 *
 * src/bancache.rs
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

use std::{net::IpAddr, sync::Arc};

use ipnetwork::IpNetwork;
use moka::future::Cache;
use sea_orm::{DbConn, DbErr};
use time::Duration;
use tracing::trace;

use service::Query;

// This caches IP bans/allows so we don't hit the database so much.

#[derive(Debug, thiserror::Error)]
pub enum BanCacheError {
    #[error(transparent)]
    Db(#[from] DbErr),
}

#[derive(Clone)]
pub struct BanCache {
    cache: Cache<IpAddr, bool>,
    db: Arc<DbConn>,
}

impl BanCache {
    pub(crate) fn new(db: Arc<DbConn>, entries: u64, ttl: Duration, idle: Duration) -> Self {
        Self {
            // XXX - should these cache parameters be configurable?
            cache: Cache::builder()
                .max_capacity(entries)
                .time_to_live(ttl.unsigned_abs())
                .time_to_idle(idle.unsigned_abs())
                .support_invalidation_closures()
                .build(),
            db,
        }
    }

    // Check for a ban, if it's not present, then check the database.
    pub(crate) async fn check_ban(&self, ip: IpAddr) -> Result<bool, BanCacheError> {
        if let Some(result) = self.cache.get(&ip).await {
            trace!("{ip}: got a cache hit (banned: {result})");
            Ok(result)
        } else {
            let result = Query::check_ip_ban(&self.db, ip).await?;
            trace!("{ip}: got a cache miss (banned: {result})");
            self.cache.insert(ip, result).await;
            Ok(result)
        }
    }

    // Invalidate a ban in the cache
    // NOTE: this must be called after a ban is created *or* deleted.
    pub(crate) fn invalidate(&self, network: IpNetwork) {
        trace!("Invalidating cache for {network}");
        self.cache
            .invalidate_entries_if(move |k, _| network.contains(*k))
            .expect("Could not invalidate cache");
    }

    // Invalidate all bans in the cache
    pub(crate) fn invalidate_all(&self) {
        trace!("Invalidating entire ban cache");
        self.cache.invalidate_all();
    }
}
