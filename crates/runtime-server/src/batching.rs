use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::{
    sync::{oneshot, Mutex, Notify},
    time::{sleep_until, Instant},
};

use runtime_core::contract::{
    engine::{InferenceBackend, ModelHandle},
    errors::RuntimeError,
    io::{InferRequest, InferResponse},
};

type BatchKey = String;

struct Pending {
    req: InferRequest,
    respond_to: oneshot::Sender<Result<InferResponse, RuntimeError>>,
    deadline: Instant,
    priority: u8,
}

struct QueueState {
    running: bool,
    items: Vec<Pending>,
}

#[derive(Clone)]
pub struct BatchScheduler {
    inner: Arc<Mutex<HashMap<BatchKey, (QueueState, Arc<Notify>)>>>,
    pub batch_size: usize,
    pub max_wait: Duration,
}

impl Default for BatchScheduler {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            batch_size: 8,
            max_wait: Duration::from_millis(25),
        }
    }
}

impl BatchScheduler {
    pub async fn submit(
        &self,
        key: BatchKey,
        backend: Arc<dyn InferenceBackend>,
        model: ModelHandle,
        req: InferRequest,
    ) -> Result<InferResponse, RuntimeError> {
        let hint = req
            .params
            .batch_hint
            .clone()
            .ok_or_else(|| RuntimeError::Invalid("batch_hint missing".into()))?;
        let priority = hint.priority.unwrap_or(5).min(10);
        let sla_ms = hint.latency_sla_ms.unwrap_or(200).max(1);
        let deadline = Instant::now() + Duration::from_millis(sla_ms).min(self.max_wait);

        let (tx, rx) = oneshot::channel();
        let notify = {
            let mut m = self.inner.lock().await;
            let entry = m.entry(key.clone()).or_insert_with(|| {
                (
                    QueueState {
                        running: false,
                        items: Vec::new(),
                    },
                    Arc::new(Notify::new()),
                )
            });

            entry.0.items.push(Pending {
                req,
                respond_to: tx,
                deadline,
                priority,
            });

            if !entry.0.running {
                entry.0.running = true;
                let inner = self.inner.clone();
                let notify_clone = entry.1.clone();
                let key_clone = key.clone();
                tokio::spawn(async move {
                    worker_loop(inner, notify_clone, key_clone, backend, model).await;
                });
            }

            entry.1.clone()
        };

        notify.notify_one();

        rx.await
            .map_err(|_| RuntimeError::Internal("batch scheduler dropped".into()))?
    }
}

async fn worker_loop(
    inner: Arc<Mutex<HashMap<BatchKey, (QueueState, Arc<Notify>)>>>,
    notify: Arc<Notify>,
    key: BatchKey,
    backend: Arc<dyn InferenceBackend>,
    model: ModelHandle,
) {
    loop {
        notify.notified().await;

        let mut batch: Vec<Pending> = Vec::new();
        let mut earliest_deadline: Option<Instant> = None;

        {
            let mut m = inner.lock().await;
            let Some((state, _)) = m.get_mut(&key) else { return; };

            state.items.sort_by(|a, b| {
                b.priority
                    .cmp(&a.priority)
                    .then_with(|| a.deadline.cmp(&b.deadline))
            });

            let take_n = state.items.len().min(8);
            for _ in 0..take_n {
                if let Some(p) = state.items.pop() {
                    earliest_deadline = Some(match earliest_deadline {
                        None => p.deadline,
                        Some(d) => d.min(p.deadline),
                    });
                    batch.push(p);
                }
            }

            if batch.is_empty() && state.items.is_empty() {
                state.running = false;
                m.remove(&key);
                return;
            }
        }

        if !batch.is_empty() {
            if let Some(d) = earliest_deadline {
                sleep_until(d).await;
            }
        }

        let mut extra: Vec<Pending> = Vec::new();
        {
            let mut m = inner.lock().await;
            if let Some((state, _)) = m.get_mut(&key) {
                let room = 8usize.saturating_sub(batch.len());
                if room > 0 {
                    state.items.sort_by(|a, b| {
                        b.priority
                            .cmp(&a.priority)
                            .then_with(|| a.deadline.cmp(&b.deadline))
                    });
                    for _ in 0..room {
                        if let Some(p) = state.items.pop() {
                            extra.push(p);
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        batch.extend(extra);

        execute_batch(&backend, &model, batch).await;
    }
}

async fn execute_batch(
    backend: &Arc<dyn InferenceBackend>,
    model: &ModelHandle,
    batch: Vec<Pending>,
) {
    for p in batch {
        let resp = backend.infer(model, p.req).await;
        let _ = p.respond_to.send(resp);
    }
}
