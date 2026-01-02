use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use runtime_core::contract::engine::{BackendFactory, InferenceBackend};
use runtime_core::contract::errors::RuntimeError;

pub mod dynamic;

#[derive(Default)]
pub struct BackendRegistry {
    factories: RwLock<HashMap<String, Arc<dyn BackendFactory>>>,
}

impl BackendRegistry {
    pub fn register(&self, factory: Arc<dyn BackendFactory>) {
        self.factories
            .write()
            .unwrap()
            .insert(factory.backend_type().to_string(), factory);
    }

    pub fn create(
        &self,
        backend_type: &str,
        settings: serde_json::Value,
    ) -> Result<Arc<dyn InferenceBackend>, RuntimeError> {
        let factory = self
            .factories
            .read()
            .unwrap()
            .get(backend_type)
            .cloned()
            .ok_or_else(|| RuntimeError::NotFound(format!("backend factory: {backend_type}")))?;

        factory.create(settings)
    }
}
