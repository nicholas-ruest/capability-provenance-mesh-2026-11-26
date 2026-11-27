//! Reproducible evidence-closed admission benchmark.
use cpm_application::{Application, IntakeRequest, UpstreamSuite};
use cpm_manifest::{Manifest, VERSION};
use cpm_store::DirectoryStore;
use std::{
    collections::BTreeSet,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

struct SevenReceipts;
impl UpstreamSuite for SevenReceipts {
    fn execute_all(&self, denial: bool) -> Result<Vec<String>, String> {
        let mode = if denial { "failure" } else { "success" };
        Ok((0..7)
            .map(|index| format!("upstream-{index}:{mode}"))
            .collect())
    }
}
fn request(version: String) -> IntakeRequest {
    IntakeRequest {
        plugin: "benchmark-plugin".into(),
        version,
        artifact: b"deterministic-wasm-artifact".to_vec(),
        manifest: Manifest {
            api_version: VERSION.into(),
            name: "benchmark-plugin".into(),
            entrypoint: "run".into(),
            capabilities: BTreeSet::from(["blob.read".into()]),
        },
        human_approved: true,
        denial_probe: false,
    }
}
fn main() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("cpm-benchmark-{}-{nonce}", std::process::id()));
    let app = Application::new(
        DirectoryStore::open(root).expect("store"),
        SevenReceipts,
        1024 * 1024,
    );
    let iterations = 100u128;
    let started = Instant::now();
    for index in 0..iterations {
        let response = app.submit(request(index.to_string())).expect("admission");
        assert!(response.admitted);
        assert_eq!(response.upstream_receipts.len(), 7);
    }
    let elapsed = started.elapsed().as_nanos();
    println!(
        "{{\"schema_version\":1,\"benchmark\":\"evidence_closed_admission\",\"iterations\":{iterations},\"elapsed_ns\":{elapsed},\"mean_ns\":{}}}",
        elapsed / iterations
    );
}
