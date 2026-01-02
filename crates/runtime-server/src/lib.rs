pub mod batching;
pub mod cache;
pub mod governance;
#[cfg(feature = "proto-gen")]
pub mod grpc;
#[cfg(not(feature = "proto-gen"))]
pub mod grpc {
    use crate::registry::BundleRegistry;

    pub fn service_with_registry(_registry: BundleRegistry) {}
}
pub mod http;
pub mod observability;
pub mod registry;
pub mod router;
