//! In-memory caching module.
//!
//! Provides time-based caching for frequently accessed data to
//! reduce database and network load.

use std::{
    ops::Deref,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::RwLock;

use crate::{database::TestMerge, route::v3::server::Server};

/// Internal cache entry type storing timestamp and cached data.
type CacheEntry<T> = RwLock<Option<(Instant, T)>>;

/// Thread-safe cache wrapper for frequently accessed data.
///
/// Provides time-based caching to reduce database and network load.
#[derive(Clone, Default)]
pub struct Cache(Arc<InnerCache>);

impl Deref for Cache {
    type Target = InnerCache;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Internal cache storage for various data types.
#[derive(Default)]
pub struct InnerCache {
    /// Cached recent test merges with 10-minute TTL.
    recent_test_merges: CacheEntry<Vec<TestMerge>>,
    /// Cached server status with 30-second TTL.
    server_status: CacheEntry<Vec<Server>>,
}

impl InnerCache {
    /// Retrieves cached recent test merges if available and not expired.
    ///
    /// # Returns
    ///
    /// - `Some(Vec<TestMerge>)` if cache is valid and not older than 10 minutes.
    /// - `None` if cache is empty or expired.
    pub async fn get_recent_test_merges(&self) -> Option<Vec<TestMerge>> {
        if let Some(cached) = &*self.recent_test_merges.read().await
            && cached.0.elapsed() < Duration::from_secs(600)
        {
            return Some(cached.1.clone());
        }

        None
    }

    /// Updates the recent test merges cache with new data.
    ///
    /// # Arguments
    ///
    /// * `recent_test_merges` - New test merge data to cache.
    pub async fn set_recent_test_merges(&self, recent_test_merges: Vec<TestMerge>) {
        let mut cache_write = self.recent_test_merges.write().await;
        *cache_write = Some((Instant::now(), recent_test_merges));
    }

    /// Retrieves cached server status if available and not expired.
    ///
    /// # Returns
    ///
    /// - `Some(Vec<Server>)` if cache is valid and not older than 30 seconds.
    /// - `None` if cache is empty or expired.
    pub async fn get_server_status(&self) -> Option<Vec<Server>> {
        if let Some(cached) = &*self.server_status.read().await
            && cached.0.elapsed() < Duration::from_secs(30)
        {
            return Some(cached.1.clone());
        }

        None
    }

    /// Updates the server status cache with new data.
    ///
    /// # Arguments
    ///
    /// * `server_status` - New server status data to cache.
    pub async fn set_server_status(&self, server_status: Vec<Server>) {
        let mut cache_write = self.server_status.write().await;
        *cache_write = Some((Instant::now(), server_status));
    }
}
