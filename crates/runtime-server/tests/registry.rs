use runtime_server::registry::{BundleModelRegistry, ModelRegistry};

#[tokio::test]
async fn bundle_registry_resolves_spec() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bundles");
    let registry = BundleModelRegistry::new(root);

    let spec = registry.resolve_spec("fixture-model").await.unwrap();
    assert_eq!(spec.name, "fixture-model");
    assert!(spec
        .source
        .as_local_path()
        .map(|p| p.contains("fixture-model/models/fixture.gguf"))
        .unwrap_or(false));
}
