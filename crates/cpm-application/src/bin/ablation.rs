//! Component and leave-one-upstream-out ablations of semantic evidence closure.
mod experiment_support;

use cpm_application::ApplicationError;
use experiment_support::{
    EvidenceSuite, FrontierFault, FrontierMetrics, Ingredient, MechanismComponent, MechanismConfig,
    application, evaluate_frontier, file_count, frontier_corpus, request, unique_root,
};
use serde::Serialize;

const HYPOTHESIS: &str = "each frontier component uniquely blocks its isolated unsafe class; removing any credited upstream also blocks production activation before a durable active write";

#[derive(Debug, Serialize)]
struct FullMechanism {
    safe_admitted: bool,
    denial_blocked: bool,
    evidence_receipts_per_decision: usize,
    durable_active_writes: usize,
    forbidden_active_writes: usize,
}

#[derive(Debug, Serialize)]
struct ComponentAblation {
    disabled: MechanismComponent,
    exposed_fault: FrontierFault,
    metrics: FrontierMetrics,
    balanced_quality_delta_vs_full_bps: i64,
    assertion_passed: bool,
}

#[derive(Debug, Serialize)]
struct UpstreamAblation {
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
    frontier_baseline: FrontierMetrics,
    frontier_full: FrontierMetrics,
    component_ablations: Vec<ComponentAblation>,
    leave_one_out: Vec<UpstreamAblation>,
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

fn ablate_component(
    component: MechanismComponent,
    corpus: &[experiment_support::FrontierCase],
    full_quality_bps: usize,
) -> ComponentAblation {
    let exposed_fault = component.fault();
    let metrics = evaluate_frontier(corpus, &MechanismConfig::without(component));
    let assertion_passed = metrics.safe_admitted == metrics.safe_candidates
        && metrics.unsafe_admitted == 5
        && metrics.unsafe_admissions_by_fault.len() == 1
        && metrics.unsafe_admissions_by_fault.get(&exposed_fault) == Some(&5)
        && metrics.forbidden_active_writes == 0
        && metrics.balanced_quality_bps < full_quality_bps;
    ComponentAblation {
        disabled: component,
        exposed_fault,
        balanced_quality_delta_vs_full_bps: i64::try_from(metrics.balanced_quality_bps)
            .expect("quality basis points fit i64")
            - i64::try_from(full_quality_bps).expect("quality basis points fit i64"),
        metrics,
        assertion_passed,
    }
}

fn ablate_upstream(ingredient: Ingredient) -> UpstreamAblation {
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
    UpstreamAblation {
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
    let production_full = full_mechanism();
    let corpus = frontier_corpus();
    let frontier_baseline = evaluate_frontier(&corpus, &MechanismConfig::baseline());
    let frontier_full = evaluate_frontier(&corpus, &MechanismConfig::full());
    let component_ablations: Vec<_> = MechanismComponent::ALL
        .into_iter()
        .map(|component| ablate_component(component, &corpus, frontier_full.balanced_quality_bps))
        .collect();
    let leave_one_upstream_out: Vec<_> = Ingredient::ALL.into_iter().map(ablate_upstream).collect();
    let assertion_passed = production_full.safe_admitted
        && production_full.denial_blocked
        && production_full.evidence_receipts_per_decision == 7
        && production_full.durable_active_writes == 1
        && production_full.forbidden_active_writes == 0
        && frontier_baseline.unsafe_admitted == frontier_baseline.unsafe_candidates
        && frontier_baseline.balanced_quality_bps == 5_000
        && frontier_full.safe_admitted == frontier_full.safe_candidates
        && frontier_full.unsafe_admitted == 0
        && frontier_full.balanced_quality_bps == 10_000
        && frontier_full.forbidden_active_writes == 0
        && component_ablations
            .iter()
            .all(|result| result.assertion_passed)
        && leave_one_upstream_out
            .iter()
            .all(|result| result.assertion_passed);
    assert!(assertion_passed, "frontier ablation hypothesis failed");
    println!(
        "{}",
        serde_json::to_string(&Report {
            schema_version: 2,
            ablation: "frontier-component-and-upstream-responsibility",
            hypothesis: HYPOTHESIS,
            full_mechanism: production_full,
            frontier_baseline,
            frontier_full,
            component_ablations,
            leave_one_out: leave_one_upstream_out,
            assertion_passed,
        })
        .expect("ablation report serialization")
    );
}
