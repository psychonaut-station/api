use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::RwLock;

use crate::database::TestMerge;

pub type Cache = Arc<InnerCache>;
type CacheEntry<T> = RwLock<Option<(Instant, T)>>;

#[derive(Default)]
pub struct InnerCache {
    recent_test_merges: CacheEntry<Vec<TestMerge>>,
}

impl InnerCache {
    pub async fn get_recent_test_merges(&self) -> Option<Vec<TestMerge>> {
        if let Some(cached) = &*self.recent_test_merges.read().await {
            if cached.0.elapsed() < Duration::from_secs(600) {
                return Some(cached.1.clone());
            }
        }

        None
    }

    pub async fn set_recent_test_merges(&self, recent_test_merges: Vec<TestMerge>) {
        let mut cache_write = self.recent_test_merges.write().await;
        *cache_write = Some((Instant::now(), recent_test_merges));
    }
}
