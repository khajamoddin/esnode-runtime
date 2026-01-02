use std::{num::NonZeroUsize, sync::Arc, time::Instant};

use lru::LruCache;
use tokio::sync::Mutex;

use runtime_core::contract::{
    engine::{InferenceBackend, ModelHandle},
    errors::RuntimeError,
};

#[derive(Clone)]
pub struct CacheLimits {
    pub max_entries: usize,
    pub max_bytes: u64,
}

impl Default for CacheLimits {
    fn default() -> Self {
        Self {
            max_entries: 4,
            max_bytes: 8 * 1024 * 1024 * 1024,
        }
    }
}

struct CacheEntry {
    backend: Arc<dyn InferenceBackend>,
    model: ModelHandle,
    loaded_at: Instant,
    last_used: Instant,
    bytes: u64,
}

pub struct LruModelCache {
    limits: CacheLimits,
    inner: Mutex<LruCache<String, CacheEntry>>,
    current_bytes: Mutex<u64>,
}

impl LruModelCache {
    pub fn new(limits: CacheLimits) -> Self {
        let cap = NonZeroUsize::new(limits.max_entries.max(1)).unwrap();
        Self {
            limits,
            inner: Mutex::new(LruCache::new(cap)),
            current_bytes: Mutex::new(0),
        }
    }

    pub async fn get(&self, key: &str) -> Option<ModelHandle> {
        let mut inner = self.inner.lock().await;
        inner.get_mut(key).map(|e| {
            e.last_used = Instant::now();
            e.model.clone()
        })
    }

    pub async fn get_or_load<F, Fut>(
        &self,
        key: &str,
        backend: Arc<dyn InferenceBackend>,
        loader: F,
    ) -> Result<ModelHandle, RuntimeError>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<ModelHandle, RuntimeError>> + Send,
    {
        if let Some(m) = self.get(key).await {
            return Ok(m);
        }

        let model = loader().await?;
        let bytes = model.memory_footprint_bytes().unwrap_or(0);

        let mut evicted: Vec<(Arc<dyn InferenceBackend>, ModelHandle)> = vec![];

        {
            let mut inner = self.inner.lock().await;
            let mut cur = self.current_bytes.lock().await;

            if let Some(existing) = inner.get_mut(key) {
                existing.last_used = Instant::now();
                let model_clone = existing.model.clone();
                drop(inner);
                drop(cur);
                let _ = backend.unload(&model).await;
                return Ok(model_clone);
            }

            inner.put(
                key.to_string(),
                CacheEntry {
                    backend: backend.clone(),
                    model: model.clone(),
                    loaded_at: Instant::now(),
                    last_used: Instant::now(),
                    bytes,
                },
            );
            *cur = cur.saturating_add(bytes);

            while let Some((_, e)) = inner.pop_lru_if_over_cap() {
                *cur = cur.saturating_sub(e.bytes);
                evicted.push((e.backend, e.model));
            }

            while *cur > self.limits.max_bytes {
                if let Some((_k, e)) = inner.pop_lru() {
                    *cur = cur.saturating_sub(e.bytes);
                    evicted.push((e.backend, e.model));
                } else {
                    break;
                }
            }
        }

        for (b, m) in evicted {
            let _ = b.unload(&m).await;
        }

        Ok(model)
    }
}

trait LruCacheExt<K, V> {
    fn pop_lru_if_over_cap(&mut self) -> Option<(K, V)>;
}

impl<K: std::hash::Hash + Eq, V> LruCacheExt<K, V> for LruCache<K, V> {
    fn pop_lru_if_over_cap(&mut self) -> Option<(K, V)> {
        if self.len() > self.cap().get() {
            self.pop_lru()
        } else {
            None
        }
    }
}
