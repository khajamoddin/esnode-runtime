fn main() {
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_PROTO_GEN");
    if std::env::var("CARGO_FEATURE_PROTO_GEN").is_err() {
        return;
    }

    let proto_root = "../runtime-proto/proto";
    let protos = [
        format!("{proto_root}/esnode.runtime.v1.proto"),
        format!("{proto_root}/esnode.models.v1.proto"),
    ];

    tonic_build::configure()
        .compile(&protos, &[proto_root])
        .expect("failed to compile protos");
}
