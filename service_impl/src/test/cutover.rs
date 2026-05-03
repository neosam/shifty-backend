//! Phase 4 — service-level cutover tests.
//! Wave 0 scaffolds with `#[ignore] + unimplemented!()` so `cargo test --list`
//! makes the test surface visible immediately. Wave 1 implements the heuristic
//! tests, Wave 2 implements the gate-tolerance tests; both flip `#[ignore]` off.

#[tokio::test]
#[ignore = "wave-1-implements-heuristic-cluster"]
async fn cluster_merges_consecutive_workdays_with_exact_match() {
    unimplemented!("wave-1: implement Heuristik-Cluster-Algorithmus per RESEARCH.md Operation 1");
}

#[tokio::test]
#[ignore = "wave-1-implements-quarantine"]
async fn quarantine_amount_below_contract() {
    unimplemented!("wave-1");
}

#[tokio::test]
#[ignore = "wave-1-implements-quarantine"]
async fn quarantine_amount_above_contract() {
    unimplemented!("wave-1");
}

#[tokio::test]
#[ignore = "wave-1-implements-quarantine"]
async fn quarantine_weekend_entry_workday_contract() {
    unimplemented!("wave-1");
}

#[tokio::test]
#[ignore = "wave-1-implements-quarantine"]
async fn quarantine_contract_not_active() {
    unimplemented!("wave-1");
}

#[tokio::test]
#[ignore = "wave-1-implements-quarantine"]
async fn quarantine_iso_53_gap() {
    unimplemented!("wave-1");
}

#[tokio::test]
#[ignore = "wave-1-implements-idempotence"]
async fn idempotent_rerun_skips_mapped() {
    unimplemented!("wave-1");
}

#[tokio::test]
#[ignore = "wave-2-implements-gate-tolerance"]
async fn gate_tolerance_pass_below_threshold() {
    unimplemented!("wave-2");
}

#[tokio::test]
#[ignore = "wave-2-implements-gate-tolerance"]
async fn gate_tolerance_fail_above_threshold() {
    unimplemented!("wave-2");
}

#[tokio::test]
#[ignore = "wave-1-implements-forbidden-tests"]
async fn run_forbidden_for_unprivileged_user() {
    unimplemented!("wave-1");
}

#[tokio::test]
#[ignore = "wave-1-implements-forbidden-tests"]
async fn run_forbidden_for_hr_only_when_committing() {
    unimplemented!("wave-1");
}
