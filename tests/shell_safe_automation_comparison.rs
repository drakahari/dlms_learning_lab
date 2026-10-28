//! Opt-in fixed public9→current11 comparison, not a producer or saved recording.
//! No Bash, process, learner source/path options, or old evidence writes.
#[path = "support/teaching_amendment.rs"]
mod teaching_amendment;
use dlms_learning_lab::{
    engine::Goal,
    pack,
    shell::{self, Case, Outcome, Vm},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::{Duration, Instant},
};
#[path = "support/automation_comparison.rs"]
mod proof;
// Reuse this binary's existing historical reader digest without another include.
mod amendment_sha {
    pub(super) fn hash(bytes: &[u8]) -> String {
        super::proof::sha(bytes)
    }
}
const SLICES: [usize; 4] = [1, 16, 32, 64];
const STOP: &str = "held=$(printf entered > case-child.txt; while true; do printf ''; done)";
const CHANGED: [&str; 23] = [
    "read-leading-trailing",
    "read-backslashes-percent",
    "read-blank-line",
    "read-empty-eof",
    "read-partial-final",
    "read-cr-utf8",
    "read-literal-empty-quotes",
    "read-double-empty-quotes",
    "read-quoted-head",
    "read-helper-scope",
    "read-capture-input-isolation",
    "read-redirect-capture-order",
    "pipe-upstream-false",
    "pipe-all-success",
    "pipe-rightmost-distinct",
    "pipe-last-distinct",
    "pipe-disable",
    "pipe-and-or",
    "pipe-capture-inherit",
    "pipe-capture-isolate",
    "pipe-function-shares",
    "pipe-child-toggle",
    "pipe-empty-prefix-read-stage",
];
const EXCLUDED: [&str; 2] = ["read-dynamic-empty-value", "prefix-other-variable"];
const CONTROLS: [&str; 3] = [
    "read-default-control",
    "pipe-default-control",
    "pipe-control-quoted-data",
];
#[derive(Deserialize)]
struct Input {
    args: Vec<String>,
    stdin: String,
    env: BTreeMap<String, String>,
    files: BTreeMap<String, String>,
}
impl Input {
    fn case(self, id: &str) -> Case {
        Case {
            name: id.into(),
            args: self.args,
            stdin: self.stdin,
            env: self.env,
            after: self.files.clone(),
            files: self.files,
            stdout: String::new(),
            stderr: String::new(),
            status: 0,
        }
    }
}
struct Vector {
    source: String,
    fixture: Case,
}
type Key = (String, String);
fn add(map: &mut BTreeMap<Key, Vector>, kind: &str, id: String, source: String, fixture: Case) {
    assert!(source.len() <= shell::MAX_SOURCE && fixture.valid());
    assert!(
        map.insert((kind.into(), id), Vector { source, fixture })
            .is_none()
    );
}
fn vectors(authored: &Value, fixed: &Value, boundaries: &Value) -> BTreeMap<Key, Vector> {
    let mut map = BTreeMap::new();
    let scripts = authored["scripts"].as_array().unwrap();
    assert_eq!(scripts.len(), 43);
    for s in scripts {
        let id = s["id"].as_str().unwrap();
        let source = s["source"].as_str().unwrap();
        for (i, c) in s["cases"].as_array().unwrap().iter().enumerate() {
            add(
                &mut map,
                "authored-case",
                format!("{id}/{i}"),
                source.into(),
                serde_json::from_value(c.clone()).unwrap(),
            );
        }
        add(
            &mut map,
            "worked-example",
            id.into(),
            s["example_source"].as_str().unwrap().into(),
            serde_json::from_value(s["example_case"].clone()).unwrap(),
        );
        if matches!(id, "shell-handover" | "shell-capture-handover") {
            for (i, c) in s["cases"].as_array().unwrap().iter().enumerate() {
                add(
                    &mut map,
                    "unfinished-assignment",
                    format!("{id}/{i}"),
                    format!("{source}\nif"),
                    serde_json::from_value(c.clone()).unwrap(),
                );
            }
            add(
                &mut map,
                "unfinished-example",
                id.into(),
                format!("{}\nif", s["example_source"].as_str().unwrap()),
                serde_json::from_value(s["example_case"].clone()).unwrap(),
            );
        }
    }
    let old8: Value = serde_json::from_slice(include_bytes!(
        "fixtures/shell-v8-synthetic-writer-progress.json"
    ))
    .unwrap();
    for (id, a) in old8["attempts"].as_object().unwrap() {
        if let Some(source) = a["workspace"]["script_learning"]["example_source"].as_str() {
            let s = scripts.iter().find(|s| s["id"] == id.as_str()).unwrap();
            if source != s["example_source"].as_str().unwrap() {
                add(
                    &mut map,
                    "imported-example",
                    id.clone(),
                    source.into(),
                    serde_json::from_value(s["example_case"].clone()).unwrap(),
                );
            }
        }
    }
    for (kind, table, count) in [
        ("fixed-automation-public9", fixed, 28),
        ("vm-boundary-public9", boundaries, 8),
    ] {
        let rows = table["cases"].as_array().unwrap();
        assert_eq!(rows.len(), count);
        for row in rows {
            let id = row["id"].as_str().unwrap();
            let input: Input = serde_json::from_value(row["input"].clone()).unwrap();
            add(
                &mut map,
                kind,
                id.into(),
                row["source"].as_str().unwrap().into(),
                input.case(id),
            );
        }
    }
    assert_eq!(map.len(), 380);
    map
}
fn observe(source: &str, fixture: &Case, slice: usize, deadline: Instant) -> (Outcome, usize) {
    assert!(Instant::now() < deadline);
    match Vm::new(source, fixture) {
        Err(_) => (shell::run(source, fixture), 0),
        Ok(mut vm) => {
            let mut ticks = 0;
            while !vm.done() {
                assert!(Instant::now() < deadline, "Fixed comparison deadline");
                assert!(ticks < 1_000_000, "Fixed tick ceiling");
                vm.tick(slice);
                ticks += 1;
            }
            (vm.outcome, ticks)
        }
    }
}
fn stop(slice: usize, deadline: Instant) -> (Outcome, usize) {
    let fixture = Case {
        name: "fixed-stop-capture".into(),
        args: vec![],
        stdin: String::new(),
        env: BTreeMap::new(),
        files: BTreeMap::new(),
        stdout: String::new(),
        stderr: String::new(),
        status: 0,
        after: BTreeMap::new(),
    };
    let mut vm = Vm::new(STOP, &fixture).unwrap();
    let mut ticks = 0;
    while vm
        .outcome
        .files
        .get("/work/case-child.txt")
        .is_none_or(|v| v != "entered")
    {
        assert!(Instant::now() < deadline && ticks < 256);
        assert!(!vm.done());
        vm.tick(slice);
        ticks += 1;
    }
    assert!(ticks > 0);
    vm.stop();
    let actual = vm.outcome.clone();
    for _ in 0..8 {
        vm.tick(slice);
        assert!(vm.done());
        proof::exact(&vm.outcome, &actual, "stopped outcome cannot resume");
    }
    (actual, ticks)
}
fn profile_metadata() -> Value {
    json!({"kind":"profile-evidence","synthetic_only":true,"catalog_contexts":133,"current9_actual_records":43,"genuine6_history_records":33,"genuine7_history_records":37,"genuine8_history_records":40,"ungraded_assignment_drafts":["shell-handover","shell-capture-handover"],"ungraded_example_drafts":["shell-handover","shell-capture-handover"],"writer_receipts":[7,8,9],"before9_exact_original8":true,"before8_exact_original7":true,"repeat_write_exact":true,"preview_cancel_preserve_target":true,"restore_exact_current9_safety_copy":true,"reset_first_exact_original8":true,"occupied_whitespace_slot_rejected":true,"future10_not_implemented":true})
}
fn verify_identity(baseline: &proof::Baseline, row: &Value, authored: &Value) {
    assert_eq!(row["revision"], 9);
    proof::exact(
        &row["linked_catalog"],
        &authored["catalog"],
        "historical catalog identity",
    );
    proof::exact(
        &row["linked_scripts"],
        &authored["scripts"],
        "historical exact43 input bundles",
    );
    let compiled = row["compiled_inputs"].as_array().unwrap();
    assert_eq!(compiled.len(), 125);
    let mut names = BTreeSet::new();
    for input in compiled {
        let name = input["file"].as_str().unwrap();
        assert!(
            !name.starts_with('/') && !name.split('/').any(|p| p == "..") && names.insert(name)
        );
        let suffix = format!("/{name}");
        let pins = baseline.manifest["pins"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|p| p["path"].as_str().is_some_and(|p| p.ends_with(&suffix)))
            .collect::<Vec<_>>();
        assert_eq!(pins.len(), 1);
        assert_eq!(input["bytes"], pins[0]["bytes"]);
        assert_eq!(input["sha256"], pins[0]["sha256"]);
    }
    for (key, name) in [
        (
            "probe_source_sha256",
            "examples/automation_before10_capture.rs",
        ),
        (
            "digest_source_sha256",
            "examples/support/automation_digest.rs",
        ),
        (
            "attestation_source_sha256",
            "examples/support/automation_attestation.rs",
        ),
    ] {
        let suffix = format!("/{name}");
        let pin = baseline.manifest["pins"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["path"].as_str().is_some_and(|p| p.ends_with(&suffix)))
            .unwrap();
        assert_eq!(row[key], pin["sha256"]);
    }
}

#[test]
#[ignore = "Requires exact accepted private public9 COMPLETE; explicit MAIN-guarded current11 comparison only"]
fn all1570_historical_rows_are_classified_and380_vectors_compared_without_relabeling() {
    assert_eq!(shell::EVALUATOR_REVISION, 11);
    let baseline = proof::Baseline::open();
    for (name, raw) in [
        (
            "tests/fixtures/automation-before10-authored-inputs.json",
            include_bytes!("fixtures/automation-before10-authored-inputs.json").as_slice(),
        ),
        (
            "tests/fixtures/automation-oracle-inputs.json",
            include_bytes!("fixtures/automation-oracle-inputs.json").as_slice(),
        ),
        (
            "tests/fixtures/automation-vm-boundaries.json",
            include_bytes!("fixtures/automation-vm-boundaries.json").as_slice(),
        ),
        (
            "tests/fixtures/shell-v8-synthetic-writer-progress.json",
            include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json").as_slice(),
        ),
    ] {
        baseline.committed_input(name, raw);
    }
    let authored: Value = serde_json::from_slice(include_bytes!(
        "fixtures/automation-before10-authored-inputs.json"
    ))
    .unwrap();
    let fixed: Value =
        serde_json::from_slice(include_bytes!("fixtures/automation-oracle-inputs.json")).unwrap();
    let boundaries: Value =
        serde_json::from_slice(include_bytes!("fixtures/automation-vm-boundaries.json")).unwrap();
    let current = pack::bundled().unwrap();
    assert_eq!(current.len(), 164);
    // Compare the exact original133 input context; the7 newly authored tasks
    // have no public9 observation and must never acquire fabricated old rows.
    let labs = &current[..133];
    let current_catalog = labs
        .iter()
        .map(|s| (s.id.clone(), s.version))
        .collect::<BTreeMap<_, _>>();
    proof::exact(
        &current_catalog,
        &authored["catalog"],
        "unchanged catalog133",
    );
    for s in authored["scripts"].as_array().unwrap() {
        let lab = labs
            .iter()
            .find(|l| l.id == s["id"].as_str().unwrap())
            .unwrap();
        let Goal::Script { cases } = &lab.goal else {
            panic!("Original kind changed")
        };
        proof::exact(cases, &s["cases"], "unchanged authored cases");
        proof::exact(
            &lab.solution,
            &vec![s["source"].as_str().unwrap()],
            "unchanged reference source",
        );
        let l = lab.teaching.as_ref().unwrap();
        assert_eq!(
            teaching_amendment::historical_example(&lab.id, l).unwrap(),
            s["example_source"].as_str().unwrap()
        );
        proof::exact(&l.fixture, &s["example_case"], "unchanged example input");
    }
    let vectors = vectors(&authored, &fixed, &boundaries);
    let mut bash = BTreeMap::new();
    for (i, row) in fixed["cases"].as_array().unwrap().iter().enumerate() {
        let id = row["id"].as_str().unwrap();
        assert_eq!(
            usize::from(CHANGED.contains(&id))
                + usize::from(EXCLUDED.contains(&id))
                + usize::from(CONTROLS.contains(&id)),
            1,
            "Fixed-case comparison policy is a disjoint complete partition"
        );
        let raw = baseline.read(&format!("fixed-{i:03}/OBSERVATION.json"));
        let value: Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(value["id"], row["id"]);
        assert_eq!(value["source"], row["source"]);
        proof::exact(&value["input"], &row["input"], "actual Bash input exact");
        assert!(
            value["actual_bash_version"]
                .as_str()
                .unwrap()
                .starts_with("GNU bash, version 5.3.9")
        );
        let mut actual = value["actual_bash"].clone();
        actual["error"] = Value::Null;
        assert!(
            bash.insert(
                row["id"].as_str().unwrap().to_string(),
                (actual, value["actual_public9"].clone())
            )
            .is_none()
        );
    }
    assert_eq!(bash.len(), 28);
    assert_eq!(CHANGED.len() + EXCLUDED.len() + CONTROLS.len(), 28);
    let deadline = Instant::now() + Duration::from_secs(150);
    let mut seen = BTreeSet::new();
    let mut current = BTreeMap::<Key, Outcome>::new();
    let mut retained = 0;
    let mut different = BTreeSet::new();
    let mut delta_hashes = BTreeMap::new();
    let mut counts = BTreeMap::<String, usize>::new();
    let mut stops = BTreeSet::new();
    let mut stop_ticks = BTreeMap::new();
    let mut records = BTreeSet::new();
    for row in &baseline.rows {
        let name = row["kind"].as_str().unwrap();
        *counts.entry(name.into()).or_default() += 1;
        match proof::kind(row).expect("Unknown/malformed baseline row") {
            proof::Kind::Named => {
                let id = row["id"].as_str().unwrap();
                let slice = row["slice"].as_u64().unwrap() as usize;
                let key = (name.to_string(), id.to_string());
                let vector = vectors.get(&key).expect("Unexpected named source");
                assert!(seen.insert((key.clone(), slice)));
                assert_eq!(row["source_sha256"], proof::sha(vector.source.as_bytes()));
                assert_eq!(row["actual_evaluator_revision"], 9);
                let (actual, ticks) = observe(&vector.source, &vector.fixture, slice, deadline);
                let actual_value = serde_json::to_value(&actual).unwrap();
                let intentional = name == "fixed-automation-public9" && CHANGED.contains(&id);
                if name == "fixed-automation-public9" {
                    let (expected, old) = &bash[id];
                    proof::exact(
                        &row["actual"],
                        old,
                        "historical fixed output vs actual observation",
                    );
                    if EXCLUDED.contains(&id) {
                        proof::exact(&actual_value, &row["actual"], id);
                    } else {
                        proof::exact(&actual_value, expected, id);
                    }
                    if intentional {
                        assert!(
                            actual_value != row["actual"],
                            "Expected changed fixed Outcome {id}"
                        );
                        different.insert(id.to_string());
                        delta_hashes.insert(id.to_string(),json!({"before9_sha256":proof::sha(&serde_json::to_vec(&row["actual"]).unwrap()),"actual_current_sha256":proof::sha(&serde_json::to_vec(&actual_value).unwrap()),"policy":"exact actual Bash5.3.9 Outcome"}));
                    }
                }
                if !intentional {
                    proof::exact(
                        &actual_value,
                        &row["actual"],
                        &format!("unchanged {name}/{id}/{slice}"),
                    );
                    assert_eq!(
                        ticks as u64,
                        row["ticks"].as_u64().unwrap(),
                        "Unchanged path tick count {name}/{id}/{slice}"
                    );
                }
                if let Some(first) = current.get(&key) {
                    proof::exact(first, &actual, id);
                } else {
                    retained += serde_json::to_vec(&actual).unwrap().len();
                    assert!(retained <= 32 * 1024 * 1024, "Retained actual Outcome cap");
                    current.insert(key, actual);
                }
            }
            proof::Kind::Record => {
                let id = row["id"].as_str().unwrap();
                assert!(records.insert(id.to_string()));
                let s = authored["scripts"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|s| s["id"] == id)
                    .unwrap();
                assert_eq!(row["scenario_version"], s["version"]);
                proof::exact(&row["cases"], &s["cases"], "original Record input cases");
                assert_eq!(row["record"]["engine_revision"], 9);
                assert_eq!(row["record"]["source"], s["source"]);
                let results = (0..s["cases"].as_array().unwrap().len())
                    .map(|i| &current[&("authored-case".into(), format!("{id}/{i}"))])
                    .collect::<Vec<_>>();
                proof::exact(
                    &results,
                    &row["record"]["results"],
                    "actual9 Record retained exact and default current results equal",
                );
            }
            proof::Kind::Stop => {
                let slice = row["slice"].as_u64().unwrap() as usize;
                assert!(SLICES.contains(&slice) && stops.insert(slice));
                assert_eq!(row["source"], STOP);
                assert_eq!(row["record_created"], false);
                let (actual, ticks) = stop(slice, deadline);
                assert!(stop_ticks.insert(slice, ticks).is_none());
                proof::exact(&actual, &row["actual"], "Stop Outcome exact");
                assert_eq!(ticks as u64, row["ticks_before_stop"].as_u64().unwrap());
            }
            proof::Kind::Identity => {
                assert!(std::ptr::eq(row, &baseline.rows[0]));
                verify_identity(&baseline, row, &authored);
            }
            proof::Kind::Profile => {
                proof::exact(
                    row,
                    &profile_metadata(),
                    "historical9 profile metadata is NOT a current11 profile claim",
                );
                let raw = baseline.read("vm-profile/writer9-original.json");
                assert_eq!(
                    proof::sha(&raw),
                    "0056319207aa4e196d8521bc64e6809daf5dd40049b1aa88aced65462dfbb492"
                );
            }
            proof::Kind::Finished => {
                assert!(std::ptr::eq(row, baseline.rows.last().unwrap()));
                proof::exact(
                    row,
                    &json!({"kind":"finished","actual_revision":9,"catalog_contexts":133,"authored_scripts":43,"authored_cases":282,"worked_examples":43,"fixed_automation_inputs":28,"vm_boundary_inputs":8,"slices":SLICES,"claims":"Actual public9 observations only; no emptyIFS/pipefail10 capability, learner execution, native acceptance or fabricated credit"}),
                    "historical9 finish metadata",
                );
            }
        }
    }
    assert_eq!(seen.len(), 1520);
    assert_eq!(current.len(), 380);
    assert_eq!(records.len(), 43);
    assert_eq!(stops, BTreeSet::from(SLICES));
    assert_eq!(different, CHANGED.into_iter().map(str::to_string).collect());
    assert_eq!(
        counts,
        BTreeMap::from([
            ("identity".into(), 1),
            ("authored-case".into(), 1128),
            ("authored-record".into(), 43),
            ("worked-example".into(), 172),
            ("unfinished-assignment".into(), 60),
            ("unfinished-example".into(), 8),
            ("imported-example".into(), 8),
            ("fixed-automation-public9".into(), 112),
            ("vm-boundary-public9".into(), 32),
            ("stopped-ungraded".into(), 4),
            ("profile-evidence".into(), 1),
            ("finished".into(), 1)
        ])
    );
    baseline.unchanged();
    let summary = json!({"kind":"actual11-comparison-test-summary","actual_evaluator_revision":shell::EVALUATOR_REVISION,"accepted_before9_manifest_sha256":proof::MANIFEST_SHA,"baseline_rows":1570,"named_outcome_rows":1520,"input_vectors":380,"unchanged_vectors":357,"intentionally_changed_fixed_vectors":23,"supported_fixed_cases":26,"excluded_fixed_cases":2,"stop_ticks":SLICES.map(|slice|stop_ticks[&slice]),"delta_hashes":delta_hashes,"compiled_shell_source_sha256":proof::sha(include_bytes!("../src/shell.rs")),"compiled_execution_source_sha256":proof::sha(include_bytes!("../src/shell/execution.rs")),"compiled_capture_source_sha256":proof::sha(include_bytes!("../src/shell/capture.rs")),"claims":"In-process opt-in comparison only; historical9 rows not relabeled; no new saved Records/profile/native/Bash capture or COMPLETE is produced"});
    let text = serde_json::to_string(&summary).unwrap();
    assert!(text.len() <= 16 * 1024);
    eprintln!("{text}");
}
