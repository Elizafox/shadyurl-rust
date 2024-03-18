/* SPDX-License-Identifier: CC0-1.0
 *
 * src/urlcache.rs
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

// Compiled regex caching

use std::sync::Arc;

use moka::future::Cache;
use regex::Regex;
use sea_orm::{DbConn, DbErr};
use time::Duration;
use tokio::sync::RwLock;
use tracing::trace;

use service::Query;

// TODO: configurable
const CACHE_ENTRIES: u64 = 1000;

#[derive(Debug, thiserror::Error)]
pub enum UrlCacheError {
    #[error(transparent)]
    Db(#[from] DbErr),

    #[error(transparent)]
    Regex(#[from] regex::Error),

    #[error("Regex not found: {}", .0)]
    RegexNotFound(String),

    #[error("Regex is duplicate: {}", .0)]
    RegexDuplicated(String),
}

#[derive(Clone, Debug)]
pub struct UrlCache {
    // This makes sure we can still clone, yet we still point to the same Vec
    regex: Arc<RwLock<Vec<Regex>>>,
    cache: Cache<String, bool>, // Caches addresses
    db: Arc<DbConn>,
}

impl UrlCache {
    // Create a new UrlCache instance and initalise regexes from the database.
    pub(crate) async fn new(db: Arc<DbConn>) -> Result<Self, UrlCacheError> {
        let mut regex = Vec::new();
        for url_filter in Query::fetch_all_url_filters(&db).await? {
            trace!("Adding URL filter {}", url_filter.0.filter);
            let cmpreg = Regex::new(&url_filter.0.filter)?;
            regex.push(cmpreg);
        }

        Ok(Self {
            // XXX - should these cache parameters be configurable?
            regex: Arc::new(RwLock::new(regex)),
            cache: Cache::builder()
                .max_capacity(CACHE_ENTRIES)
                .time_to_live(Duration::days(7).unsigned_abs())
                .time_to_idle(Duration::days(1).unsigned_abs())
                .support_invalidation_closures()
                .build(),
            db,
        })
    }

    // Sync the regex cache with the database, flushing all old entries
    // This also flushes the URL cache.
    pub(crate) async fn sync_regex_cache(&mut self) -> Result<(), UrlCacheError> {
        let mut regex_vec = self.regex.write().await;
        let mut new_regex = Vec::with_capacity(regex_vec.len());

        for url_filter in Query::fetch_all_url_filters(&self.db).await? {
            let cmpreg = Regex::new(&url_filter.0.filter)?;
            new_regex.push(cmpreg);
        }

        regex_vec.clear();
        regex_vec.extend(new_regex);
        drop(regex_vec);
        self.cache.invalidate_all();
        Ok(())
    }

    // Add one regex without flushing the entire cache.
    // This operation also removes any matching URL's from the cache.
    // NOTE: this does not update the database
    pub(crate) async fn add_regex_cache(&mut self, cmpreg: Regex) -> Result<(), UrlCacheError> {
        let mut regex_vec = self.regex.write().await;
        if regex_vec
            .iter()
            .any(|ocmpreg| ocmpreg.as_str() == cmpreg.as_str())
        {
            trace!("URL is duplicated: {}", cmpreg.as_str());
            return Err(UrlCacheError::RegexDuplicated(cmpreg.as_str().to_string()));
        }

        trace!("Adding URL filter to regex cache {}", cmpreg.as_str());

        regex_vec.push(cmpreg.clone());
        drop(regex_vec);
        self.cache
            .invalidate_entries_if(move |k, _| cmpreg.is_match(k))
            .expect("Could not invalidate cache");
        Ok(())
    }

    // Remove one regex without flushing the entire cache.
    // This operation also removes any matching URL's from the cache.
    // NOTE: this does not update the database
    pub(crate) async fn remove_regex_cache(&mut self, regstr: &str) -> Result<(), UrlCacheError> {
        // Invariant: regexes are not duplicated
        let mut regex_vec = self.regex.write().await;
        let pos = regex_vec
            .iter()
            .position(|cmpreg| cmpreg.as_str() == regstr)
            .ok_or_else(|| UrlCacheError::RegexNotFound(regstr.to_string()))?;
        let cmpreg = regex_vec[pos].clone();
        regex_vec.swap_remove(pos);
        drop(regex_vec);
        self.cache
            .invalidate_entries_if(move |k, _| cmpreg.is_match(k))
            .expect("Could not invalidate cache");

        trace!("Removing URL filter from regex cache {}", regstr);
        Ok(())
    }

    // Check URL against cache.
    // If not found, it will check the regexes, and cache the result.
    // Returns true if found in the cache, false otherwise.
    pub(crate) async fn check_url_banned(&self, url: &str) -> Result<bool, UrlCacheError> {
        if let Some(is_match) = self.cache.get(url).await {
            trace!("Cached URL ban result for \"{}\": {:?}", url, is_match);
            return Ok(is_match);
        }

        let regex_vec = self.regex.read().await;
        let is_match = regex_vec
            .iter()
            .map(|cmpreg| cmpreg.is_match(url))
            .any(|x| x);
        drop(regex_vec);

        trace!("Uncached URL ban result for \"{}\": {:?}", url, is_match);
        self.cache.insert(url.to_string(), is_match).await;
        Ok(is_match)
    }
}
