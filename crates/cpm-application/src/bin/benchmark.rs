//! Fixed-workload baseline-versus-mechanism benchmark for admission evidence closure.
mod experiment_support;

use experiment_support::{
    EvidenceSuite, application, file_count, manifest_only_admits, request, unique_root,
};
use serde::Serialize;
use std::time::Instant;

const WORKLOAD_SIZE: usize = 40;
const HYPOTHESIS: &str = "source-bound semantic evidence closure eliminates unsafe manifest-only admissions while preserving safe admissions with bounded latency";

#[derive(Debug, Serialize)]
struct Metrics {
    decisions: usize,
    admitted: usize,
    false_admits: usize,
    evidence_receipts: usize,
    durable_active_writes: usize,
    forbidden_active_writes: usize,
    elapsed_ns: u128,
    mean_ns: u128,
    p95_ns: u128,
}

#[derive(Debug, Serialize)]
struct Report {
    schema_version: u8,
    benchmark: &'static str,
    hypothesis: &'static str,
    workload: &'static str,
    baseline: Metrics,
    mechanism: Metrics,
    assertion_passed: bool,
}

fn percentile_95(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index]
}

fn baseline() -> Metrics {
    let mut admitted = 0;
    let mut false_admits = 0;
    let mut samples = Vec::with_capacity(WORKLOAD_SIZE);
    let begin = Instant::now();
    for index in 0..WORKLOAD_SIZE {
        let denial = index % 2 == 1;
        let candidate = request(format!("baseline-{index}"), denial);
        let sample = Instant::now();
        let decision = manifest_only_admits(&candidate);
        samples.push(sample.elapsed().as_nanos());
        admitted += usize::from(decision);
        false_admits += usize::from(decision && denial);
    }
    let elapsed_ns = begin.elapsed().as_nanos();
    Metrics {
        decisions: WORKLOAD_SIZE,
        admitted,
        false_admits,
        evidence_receipts: 0,
        durable_active_writes: 0,
        forbidden_active_writes: 0,
        elapsed_ns,
        mean_ns: elapsed_ns / WORKLOAD_SIZE as u128,
        p95_ns: percentile_95(samples),
    }
}

fn mechanism() -> Metrics {
    let root = unique_root("benchmark");
    let app = application(&root, EvidenceSuite::full());
    let mut admitted = 0;
    let mut false_admits = 0;
    let mut evidence_receipts = 0;
    let mut forbidden_active_writes = 0;
    let mut samples = Vec::with_capacity(WORKLOAD_SIZE);
    let begin = Instant::now();
    for index in 0..WORKLOAD_SIZE {
        let denial = index % 2 == 1;
        let version = format!("1.0.{index}");
        let sample = Instant::now();
        let response = app
            .submit(request(version.clone(), denial))
            .expect("complete mechanism must decide fixed workload");
        samples.push(sample.elapsed().as_nanos());
        admitted += usize::from(response.admitted);
        false_admits += usize::from(response.admitted && denial);
        evidence_receipts += response.upstream_receipts.len();
        forbidden_active_writes +=
            usize::from(denial && app.retrieve("fixed-workload-plugin", &version).is_ok());
    }
    let elapsed_ns = begin.elapsed().as_nanos();
    Metrics {
        decisions: WORKLOAD_SIZE,
        admitted,
        false_admits,
        evidence_receipts,
        durable_active_writes: file_count(&root, "active"),
        forbidden_active_writes,
        elapsed_ns,
        mean_ns: elapsed_ns / WORKLOAD_SIZE as u128,
        p95_ns: percentile_95(samples),
    }
}

fn main() {
    let baseline = baseline();
    let mechanism = mechanism();
    let assertion_passed = baseline.false_admits == WORKLOAD_SIZE / 2
        && mechanism.false_admits == 0
        && mechanism.forbidden_active_writes == 0
        && mechanism.admitted == WORKLOAD_SIZE / 2
        && mechanism.evidence_receipts == WORKLOAD_SIZE * 7;
    assert!(assertion_passed, "frontier benchmark hypothesis failed");
    println!(
        "{}",
        serde_json::to_string(&Report {
            schema_version: 2,
            benchmark: "manifest-only-vs-semantic-evidence-closure",
            hypothesis: HYPOTHESIS,
            workload: "40 deterministic candidates: 20 safe and 20 with an undeclared observed network effect",
            baseline,
            mechanism,
            assertion_passed,
        })
        .expect("benchmark report serialization")
    );
}
