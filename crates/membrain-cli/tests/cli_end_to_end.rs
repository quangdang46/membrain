use serde_json::Value;
use std::process::Command;

fn run_membrain(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_membrain"))
        .args(args)
        .output()
        .expect("membrain binary should run");

    (
        output.status.success(),
        String::from_utf8(output.stdout).expect("stdout should be utf8"),
        String::from_utf8(output.stderr).expect("stderr should be utf8"),
    )
}

fn parse_json(stdout: &str) -> Value {
    serde_json::from_str(stdout).expect("command should emit valid json")
}

#[test]
fn cli_encode_json_emits_expected_machine_readable_fields() {
    let (ok, stdout, stderr) = run_membrain(&[
        "encode",
        "Paris is the capital of France",
        "--namespace",
        "test_ns",
        "--kind",
        "semantic",
        "--source",
        "cli-e2e",
        "--json",
    ]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["ok"], true);
    assert_eq!(json["namespace"], "test_ns");
    assert_eq!(json["outcome_class"], "accepted");
    assert_eq!(json["result"]["outcome"], "accepted");
    assert_eq!(json["result"]["memory_id"], 1);
    assert_eq!(json["result"]["namespace"], "test_ns");
    assert_eq!(json["result"]["memory_type"], "observation");
    assert_eq!(
        json["result"]["compact_text"],
        "Paris is the capital of France"
    );
    assert_eq!(json["result"]["source"], "cli-e2e");
}

#[test]
fn cli_recall_json_preserves_route_and_result_fields() {
    let (ok, stdout, stderr) = run_membrain(&[
        "recall",
        "capital of France",
        "--namespace",
        "test_ns",
        "--top",
        "3",
        "--explain",
        "full",
        "--json",
    ]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["ok"], true);
    assert_eq!(json["namespace"], "test_ns");
    assert_eq!(json["outcome_class"], "accepted");
    assert!(json["route_summary"]["route_reason"].as_str().is_some());
    assert!(json["route_summary"]["tier1_consulted_first"]
        .as_bool()
        .is_some());
    assert!(json["result"]["evidence_pack"].is_array());
    assert!(json["result"]["action_pack"].is_null());
    assert_eq!(json["result"]["packaging_metadata"]["result_budget"], 3);
}

#[test]
fn cli_explain_json_preserves_trace_stages_and_patterns() {
    let (ok, stdout, stderr) = run_membrain(&[
        "explain",
        "how to deploy the service after the last incident?",
        "--namespace",
        "test_ns",
        "--json",
    ]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["ok"], true);
    assert_eq!(json["namespace"], "test_ns");
    assert_eq!(json["outcome_class"], "accepted");
    assert!(json["trace_stages"].is_array());
    assert_eq!(json["result"]["explain"]["ranking_profile"], "balanced");
    assert!(json["result"]["explain"]["result_reasons"]
        .as_array()
        .expect("result_reasons should be an array")
        .iter()
        .any(|value| value["detail"]
            .as_str()
            .is_some_and(|detail| detail.contains("matched_patterns=how to"))));
}

#[test]
fn cli_inspect_missing_memory_json_preserves_validation_failure_taxonomy() {
    let (ok, stdout, _stderr) =
        run_membrain(&["inspect", "--id", "123", "--namespace", "test_ns", "--json"]);

    assert!(
        ok,
        "inspect --json should return a failure envelope with exit 0"
    );
    let json = parse_json(&stdout);
    assert_eq!(json["ok"], false);
    assert_eq!(json["error_kind"], "validation_failure");
    assert_eq!(json["outcome_class"], "rejected");
    assert_eq!(json["request_id"], "inspect-not-found");
    assert_eq!(json["remediation"]["summary"], "validation_failure");
    assert_eq!(json["remediation"]["next_steps"][0], "fix_request");
}

#[test]
fn cli_maintenance_json_reports_rebuild_outputs() {
    let (ok, stdout, stderr) = run_membrain(&[
        "maintenance",
        "--action",
        "repair",
        "--namespace",
        "test_ns",
        "--json",
    ]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["outcome"], "accepted");
    assert_eq!(json["action"], "repair");
    assert_eq!(json["namespace"], "test_ns");
    assert_eq!(json["targets_checked"], 2);
    assert_eq!(json["rebuilt"], 2);
    assert_eq!(json["results"].as_array().map(Vec::len), Some(2));
}

#[test]
fn cli_audit_json_preserves_request_and_run_correlation_fields() {
    let (ok, stdout, stderr) = run_membrain(&[
        "audit",
        "--namespace",
        "team.alpha",
        "--id",
        "21",
        "--recent",
        "1",
        "--json",
    ]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["total_matches"], 3);
    assert_eq!(json["returned_rows"], 1);
    assert_eq!(json["truncated"], true);
    let rows = json["rows"]
        .as_array()
        .expect("audit output should include rows");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["request_id"], "req-migration-21");
    assert_eq!(rows[0]["related_run"], "migration-0042");
    assert_eq!(rows[0]["kind"], "maintenance_migration_applied");
}

#[test]
fn cli_doctor_json_reports_health_and_repair_state() {
    let (ok, stdout, stderr) = run_membrain(&["doctor"]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["action"], "doctor");
    assert!(json["metrics"].is_object());
    assert!(json["indexes"].is_array());
    assert!(json["repair_reports"].is_array());
    assert!(json["health"].is_object());
}

#[test]
fn cli_share_json_reports_visibility_policy_and_audit_fields() {
    let (ok, stdout, stderr) =
        run_membrain(&["share", "--id", "42", "--namespace", "team.beta", "--json"]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["ok"], true);
    assert_eq!(json["namespace"], "team.beta");
    assert_eq!(json["result"]["visibility"], "shared");
    assert_eq!(
        json["result"]["policy_summary"]["policy_family"],
        "visibility_sharing"
    );
    assert_eq!(
        json["result"]["audit_rows"][0]["request_id"],
        "req-share-42"
    );
    assert_eq!(json["result"]["audit_rows"][0]["kind"], "policy_redacted");
}

#[test]
fn cli_unshare_json_reports_redaction_and_audit_fields() {
    let (ok, stdout, stderr) = run_membrain(&[
        "unshare",
        "--id",
        "42",
        "--namespace",
        "team.alpha",
        "--json",
    ]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["ok"], true);
    assert_eq!(json["namespace"], "team.alpha");
    assert_eq!(json["result"]["visibility"], "private");
    assert_eq!(
        json["result"]["policy_summary"]["redaction_fields"][0],
        "sharing_scope"
    );
    assert_eq!(
        json["result"]["audit_rows"][0]["request_id"],
        "req-unshare-42"
    );
    assert_eq!(json["result"]["audit_rows"][0]["kind"], "policy_denied");
    assert_eq!(json["result"]["audit_rows"][0]["redacted"], true);
}

#[test]
fn cli_recall_human_output_logs_route_and_result_lines() {
    let (ok, stdout, stderr) = run_membrain(&[
        "recall",
        "capital of France",
        "--namespace",
        "test_ns",
        "--top",
        "3",
        "--explain",
        "full",
    ]);

    assert!(ok, "stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr should stay empty: {stderr}");
    assert!(stdout.contains("Recall 'capital of France' in 'test_ns' → 0 results"));
    assert!(stdout.contains("route: "));
    assert!(stdout.contains("tier1: consulted=true, answered_directly=true, deeper=false"));
}

#[test]
fn cli_audit_human_output_logs_redaction_and_run_correlation() {
    let (ok, stdout, stderr) = run_membrain(&[
        "audit",
        "--namespace",
        "team.alpha",
        "--id",
        "21",
        "--recent",
        "1",
    ]);

    assert!(ok, "stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr should stay empty: {stderr}");
    assert!(stdout.contains("matched=3 returned=1 truncated=true"));
    assert!(stdout.contains("#3 maintenance maintenance_migration_applied"));
    assert!(stdout.contains("request_id=Some(\"req-migration-21\")"));
    assert!(stdout.contains("redacted=false"));
    assert!(stdout.contains("run=Some(\"migration-0042\")"));
}

#[test]
fn cli_share_and_unshare_human_output_logs_visibility_transitions() {
    let (share_ok, share_stdout, share_stderr) =
        run_membrain(&["share", "--id", "42", "--namespace", "team.beta"]);
    assert!(share_ok, "stderr: {share_stderr}");
    assert!(
        share_stderr.is_empty(),
        "stderr should stay empty: {share_stderr}"
    );
    assert_eq!(
        share_stdout.trim(),
        "Shared memory #42 into 'team.beta' [shared]"
    );

    let (unshare_ok, unshare_stdout, unshare_stderr) =
        run_membrain(&["unshare", "--id", "42", "--namespace", "team.alpha"]);
    assert!(unshare_ok, "stderr: {unshare_stderr}");
    assert!(
        unshare_stderr.is_empty(),
        "stderr should stay empty: {unshare_stderr}"
    );
    assert_eq!(
        unshare_stdout.trim(),
        "Unshared memory #42 in 'team.alpha' [private]"
    );
}
