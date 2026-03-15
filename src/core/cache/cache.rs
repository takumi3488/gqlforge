use std::hash::Hash;
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, PoisonError, RwLock};
use std::time::Duration;

use ttl_cache::TtlCache;

use super::error::Result;

pub struct InMemoryCache<K: Hash + Eq, V> {
    data: Arc<RwLock<TtlCache<K, V>>>,
    hits: AtomicUsize,
    miss: AtomicUsize,
}

impl<K: Hash + Eq, V: Clone> Default for InMemoryCache<K, V> {
    fn default() -> Self {
        Self::new(100_000)
    }
}

impl<K: Hash + Eq, V: Clone> InMemoryCache<K, V> {
    #[must_use] 
    pub fn new(capacity: usize) -> Self {
        InMemoryCache {
            data: Arc::new(RwLock::new(TtlCache::new(capacity))),
            hits: AtomicUsize::new(0),
            miss: AtomicUsize::new(0),
        }
    }
}

#[async_trait::async_trait]
impl<K: Hash + Eq + Send + Sync, V: Clone + Send + Sync> crate::core::Cache
    for InMemoryCache<K, V>
{
    type Key = K;
    type Value = V;
    async fn set<'a>(&'a self, key: K, value: V, ttl: NonZeroU64) -> Result<()> {
        let ttl = Duration::from_millis(ttl.get());
        self.data.write().unwrap_or_else(PoisonError::into_inner).insert(key, value, ttl);
        Ok(())
    }

    async fn get<'a>(&'a self, key: &'a K) -> Result<Option<Self::Value>> {
        let val = self.data.read().unwrap_or_else(PoisonError::into_inner).get(key).cloned();
        if val.is_some() {
            self.hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.miss.fetch_add(1, Ordering::Relaxed);
        }
        Ok(val)
    }

    fn hit_rate(&self) -> Option<f64> {
        let cache = self.data.read().unwrap_or_else(PoisonError::into_inner);
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.miss.load(Ordering::Relaxed);

        drop(cache);

        if hits + misses > 0 {
            return Some(f64::from(u32::try_from(hits).unwrap_or(u32::MAX)) / f64::from(u32::try_from(hits + misses).unwrap_or(u32::MAX)));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "test code")]
    use std::num::NonZeroU64;
    use std::time::Duration;

    use crate::core::Cache;

    #[tokio::test]
    async fn test_native_chrono_cache_set_get() {
        let cache: crate::core::cache::InMemoryCache<u64, String> =
            crate::core::cache::InMemoryCache::default();
        let ttl = NonZeroU64::new(100).unwrap();
        assert_eq!(cache.get(&10).await.ok(), Some(None));

        cache.set(10, "hello".into(), ttl).await.unwrap();
        assert_eq!(cache.get(&10).await.ok(), Some(Some("hello".into())));

        cache.set(10, "bye".into(), ttl).await.ok();
        tokio::time::sleep(Duration::from_millis(ttl.get())).await;
        assert_eq!(cache.get(&10).await.ok(), Some(None));
    }
}
