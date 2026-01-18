use std::{
    ops::Deref,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::RwLock;

use crate::{database::TestMerge, route::v3::server::Server};

type CacheEntry<T> = RwLock<Option<(Instant, T)>>;

#[derive(Clone, Default)]
pub struct Cache(Arc<InnerCache>);

impl Deref for Cache {
    type Target = InnerCache;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default)]
pub struct InnerCache {
    recent_test_merges: CacheEntry<Vec<TestMerge>>,
    server_status: CacheEntry<Vec<Server>>,
}

impl InnerCache {
    pub async fn get_recent_test_merges(&self) -> Option<Vec<TestMerge>> {
        if let Some(cached) = &*self.recent_test_merges.read().await
            && cached.0.elapsed() < Duration::from_secs(600)
        {
            return Some(cached.1.clone());
        }

        None
    }

    pub async fn set_recent_test_merges(&self, recent_test_merges: Vec<TestMerge>) {
        let mut cache_write = self.recent_test_merges.write().await;
        *cache_write = Some((Instant::now(), recent_test_merges));
    }

    pub async fn get_server_status(&self) -> Option<Vec<Server>> {
        if let Some(cached) = &*self.server_status.read().await
            && cached.0.elapsed() < Duration::from_secs(30)
        {
            return Some(cached.1.clone());
        }

        None
    }

    pub async fn set_server_status(&self, server_status: Vec<Server>) {
        let mut cache_write = self.server_status.write().await;
        *cache_write = Some((Instant::now(), server_status));
    }
}
