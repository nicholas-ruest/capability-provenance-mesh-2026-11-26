//! Leave-one-upstream-out ablation of semantic evidence closure.
mod experiment_support;

use cpm_application::ApplicationError;
use experiment_support::{
    EvidenceSuite, Ingredient, application, file_count, request, unique_root,
};
use serde::Serialize;

const HYPOTHESIS: &str = "admission requires a complete seven-responsibility evidence closure; removing any one responsibility blocks activation before a durable active write";

#[derive(Debug, Serialize)]
struct FullMechanism {
    safe_admitted: bool,
    denial_blocked: bool,
    evidence_receipts_per_decision: usize,
    durable_active_writes: usize,
    forbidden_active_writes: usize,
}

#[derive(Debug, Serialize)]
struct AblationResult {
    omitted: Ingredient,
    remaining_receipts: usize,
    admission: &'static str,
    durable_decision_writes: usize,
    durable_active_writes: usize,
    forbidden_active_writes: usize,
    assertion_passed: bool,
}

#[derive(Debug, Serialize)]
struct Report {
    schema_version: u8,
    ablation: &'static str,
    hypothesis: &'static str,
    full_mechanism: FullMechanism,
    leave_one_out: Vec<AblationResult>,
    assertion_passed: bool,
}

fn full_mechanism() -> FullMechanism {
    let root = unique_root("ablation-full");
    let app = application(&root, EvidenceSuite::full());
    let safe = app
        .submit(request("1.0.0", false))
        .expect("full mechanism safe decision");
    let denied = app
        .submit(request("1.0.1", true))
        .expect("full mechanism denial decision");
    let forbidden_active_writes =
        usize::from(app.retrieve("fixed-workload-plugin", "1.0.1").is_ok());
    FullMechanism {
        safe_admitted: safe.admitted,
        denial_blocked: !denied.admitted,
        evidence_receipts_per_decision: safe.upstream_receipts.len(),
        durable_active_writes: file_count(&root, "active"),
        forbidden_active_writes,
    }
}

fn ablate(ingredient: Ingredient) -> AblationResult {
    let root = unique_root("ablation-one");
    let app = application(&root, EvidenceSuite::without(ingredient));
    let result = app.submit(request("1.0.0", false));
    let blocked = matches!(result, Err(ApplicationError::Admission));
    let durable_decision_writes = file_count(&root, "decisions");
    let durable_active_writes = file_count(&root, "active");
    let forbidden_active_writes = durable_active_writes;
    let assertion_passed = blocked
        && durable_decision_writes == 0
        && durable_active_writes == 0
        && forbidden_active_writes == 0;
    AblationResult {
        omitted: ingredient,
        remaining_receipts: 6,
        admission: if blocked { "blocked" } else { "unexpected" },
        durable_decision_writes,
        durable_active_writes,
        forbidden_active_writes,
        assertion_passed,
    }
}

fn main() {
    let full_mechanism = full_mechanism();
    let leave_one_out: Vec<_> = Ingredient::ALL.into_iter().map(ablate).collect();
    let assertion_passed = full_mechanism.safe_admitted
        && full_mechanism.denial_blocked
        && full_mechanism.evidence_receipts_per_decision == 7
        && full_mechanism.durable_active_writes == 1
        && full_mechanism.forbidden_active_writes == 0
        && leave_one_out.iter().all(|result| result.assertion_passed);
    assert!(assertion_passed, "upstream ablation hypothesis failed");
    println!(
        "{}",
        serde_json::to_string(&Report {
            schema_version: 2,
            ablation: "leave-one-upstream-responsibility-out",
            hypothesis: HYPOTHESIS,
            full_mechanism,
            leave_one_out,
            assertion_passed,
        })
        .expect("ablation report serialization")
    );
}
