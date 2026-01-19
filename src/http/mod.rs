pub mod discord;

use std::{sync::Arc, time::Duration};

use once_cell::sync::Lazy;
use reqwest::Client;
use tokio::{
    sync::{Mutex, MutexGuard},
    time::{Instant, sleep},
};

type Result<T> = std::result::Result<T, Error>;

pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(Client::new);

struct TokenBucketInner {
    tokens: usize,
    capacity: usize,
    max_capacity: usize,
    last_refill: Instant,
    refill_interval: Duration,
}

#[must_use = "must hold permit to keep token reserved"]
pub struct BucketPermit<'a> {
    bucket: MutexGuard<'a, TokenBucketInner>,
}

impl<'a> Drop for BucketPermit<'a> {
    fn drop(&mut self) {
        self.bucket.capacity = (self.bucket.capacity + 1).min(self.bucket.max_capacity);
    }
}

pub struct TokenBucket {
    inner: Arc<Mutex<TokenBucketInner>>,
}

impl TokenBucket {
    pub fn new(capacity: usize, refill_interval: f32) -> Self {
        let inner = TokenBucketInner {
            tokens: capacity,
            capacity,
            max_capacity: capacity,
            last_refill: Instant::now(),
            refill_interval: Duration::from_secs_f32(refill_interval),
        };

        TokenBucket {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    fn refill(&self, inner: &mut TokenBucketInner) {
        let now = Instant::now();
        let elapsed = now.duration_since(inner.last_refill);

        if elapsed >= inner.refill_interval {
            inner.tokens = inner.capacity;
            inner.last_refill = now;
        }
    }

    pub async fn acquire(&self) -> BucketPermit<'_> {
        loop {
            let mut inner = self.inner.lock().await;

            self.refill(&mut inner);

            if inner.tokens > 0 {
                inner.tokens -= 1;
                inner.capacity -= 1;
                return BucketPermit { bucket: inner };
            }

            let elapsed = Instant::now().duration_since(inner.last_refill);
            let wait_time = inner.refill_interval.saturating_sub(elapsed);
            drop(inner);

            sleep(wait_time).await;
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to perform HTTP request: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Discord API returned an error: {code} - {message}")]
    Discord { code: u32, message: String },
    #[error("rate limited by Discord API")]
    RateLimited,
}
