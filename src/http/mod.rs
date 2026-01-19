//! HTTP client and other utilities.
//!
//! Provides a shared HTTP client and token bucket rate limiter for controlling
//! API request rates, particularly for Discord API calls.

pub mod discord;

use std::{sync::Arc, time::Duration};

use once_cell::sync::Lazy;
use reqwest::Client;
use tokio::{
    sync::{Mutex, MutexGuard},
    time::{Instant, sleep},
};

type Result<T> = std::result::Result<T, Error>;

/// Global HTTP client for making requests.
pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(Client::new);

/// Internal state for the token bucket rate limiter.
struct TokenBucketInner {
    /// Current number of available tokens
    tokens: usize,
    /// Current capacity (reduced temporarily when tokens are acquired)
    capacity: usize,
    /// Maximum capacity that can be restored
    max_capacity: usize,
    /// Last time tokens were refilled
    last_refill: Instant,
    /// Duration between refills
    refill_interval: Duration,
}

/// RAII guard that holds a token bucket permit.
///
/// When dropped, the permit returns the token to the bucket.
#[must_use = "must hold permit to keep token reserved"]
pub struct BucketPermit<'a> {
    /// Reference to the locked token bucket inner state
    bucket: MutexGuard<'a, TokenBucketInner>,
}

impl<'a> Drop for BucketPermit<'a> {
    fn drop(&mut self) {
        self.bucket.capacity = (self.bucket.capacity + 1).min(self.bucket.max_capacity);
    }
}

/// Token bucket rate limiter for controlling API request rates.
///
/// This implements a token bucket algorithm where tokens are refilled at regular
/// intervals. Each request consumes a token, and if no tokens are available,
/// the request must wait until the next refill.
pub struct TokenBucket {
    /// Shared inner state of the token bucket
    inner: Arc<Mutex<TokenBucketInner>>,
}

impl TokenBucket {
    /// Creates a new token bucket with the specified capacity and refill interval.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of tokens in the bucket
    /// * `refill_interval` - Time in seconds between refills
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

    /// Refills tokens if enough time has elapsed since the last refill.
    fn refill(&self, inner: &mut TokenBucketInner) {
        let now = Instant::now();
        let elapsed = now.duration_since(inner.last_refill);

        if elapsed >= inner.refill_interval {
            inner.tokens = inner.capacity;
            inner.last_refill = now;
        }
    }

    /// Acquires a token from the bucket, waiting if necessary.
    ///
    /// This method will wait until a token becomes available. When the returned
    /// permit is dropped, the token is returned to the bucket.
    ///
    /// # Returns
    ///
    /// A `BucketPermit` that must be held to keep the token reserved
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

/// Errors that can occur during HTTP operations.
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
