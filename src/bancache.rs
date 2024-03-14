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
use tracing::debug;

use service::Query;

const CACHE_ENTRIES: u64 = 10_000;

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
    pub(crate) fn new(db: Arc<DbConn>) -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(CACHE_ENTRIES)
                .time_to_live(Duration::days(1).unsigned_abs())
                .time_to_idle(Duration::minutes(10).unsigned_abs())
                .support_invalidation_closures()
                .build(),
            db,
        }
    }

    pub(crate) async fn check_ban(&self, ip: IpAddr) -> Result<bool, BanCacheError> {
        if let Some(result) = self.cache.get(&ip).await {
            debug!("Got a cache hit");
            Ok(result)
        } else {
            debug!("Cache miss");
            let result = Query::check_ip_ban(&self.db, ip).await?;
            self.cache.insert(ip, result).await;
            Ok(result)
        }
    }

    pub(crate) fn invalidate(&self, network: IpNetwork) {
        self.cache
            .invalidate_entries_if(move |k, _| network.contains(*k))
            .expect("Could not invalidate cache");
    }
}