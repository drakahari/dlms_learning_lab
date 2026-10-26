//! Fixed developer-only actual-public9 capture. Never a learner source dispatcher.
//! No stdin/CLI source, no host subprocess, no reference shell or app runtime hook.
use dlms_learning_lab::{
    backup,
    engine::{Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching,
    shell::{self, Case, Outcome, Record, Vm},
    workspace,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, AtomicUsize},
    time::{Duration, Instant},
};
#[path = "support/automation_attestation.rs"]
mod attestation;
#[path = "support/automation_digest.rs"]
mod digest;
const ORIGINAL8: &[u8] =
    include_bytes!("../tests/fixtures/shell-v8-synthetic-writer-progress.json");
const ORIGINAL7: &[u8] =
    include_bytes!("../tests/fixtures/shell-v7-synthetic-writer-progress.json");
const OLD7: &str = include_str!("../tests/fixtures/shell-v7-authored-records.jsonl");
const INPUTS: &str = include_str!("../tests/fixtures/automation-oracle-inputs.json");
const VM_BOUNDARIES: &str = include_str!("../tests/fixtures/automation-vm-boundaries.json");
const AUTHORED: &str = include_str!("../tests/fixtures/automation-before10-authored-inputs.json");
const ROW_CAP: usize = 4 * 1024 * 1024;
const STREAM_CAP: usize = 32 * 1024 * 1024;
const PROFILE_CAP: usize = 16_000_000;
const SLICES: [usize; 4] = [1, 16, 32, 64];

struct Limited<W> {
    inner: W,
    count: usize,
    cap: usize,
}
impl<W: Write> Write for Limited<W> {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        if b.len() > self.cap.saturating_sub(self.count) {
            return Err(io::Error::other("Fixed capture byte ceiling"));
        }
        let n = self.inner.write(b)?;
        self.count += n;
        Ok(n)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
fn row(out: &mut impl Write, value: &impl Serialize) {
    let mut bytes = Limited {
        inner: Vec::new(),
        count: 0,
        cap: ROW_CAP,
    };
    serde_json::to_writer(&mut bytes, value).unwrap();
    out.write_all(&bytes.inner).unwrap();
    out.write_all(b"\n").unwrap();
}
#[derive(Deserialize)]
struct FixedInputs {
    cases: Vec<Fixed>,
}
#[derive(Deserialize)]
struct Fixed {
    id: String,
    source: String,
    input: Input,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    args: Vec<String>,
    stdin: String,
    env: BTreeMap<String, String>,
    files: BTreeMap<String, String>,
}
impl Input {
    fn case(&self, id: &str) -> Case {
        // These fields are API scaffolding, not predicted successful9 observations.
        Case {
            name: id.into(),
            args: self.args.clone(),
            stdin: self.stdin.clone(),
            env: self.env.clone(),
            files: self.files.clone(),
            stdout: String::new(),
            stderr: String::new(),
            status: 0,
            after: self.files.clone(),
        }
    }
}
#[derive(Deserialize)]
struct Authored {
    catalog: BTreeMap<String, u32>,
    scripts: Vec<Script>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Script {
    id: String,
    version: u32,
    unit: String,
    source: String,
    cases: Vec<Case>,
    example_source: String,
    example_case: Case,
}
#[derive(Deserialize)]
struct Old {
    id: String,
    record: Record,
}
const STOP_SOURCE: &str = "held=$(printf entered > case-child.txt; while true; do printf ''; done)";
fn observe_stop(slice: usize, deadline: Instant) -> (Outcome, usize) {
    let stop_case = Case {
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
    let mut vm = Vm::new(STOP_SOURCE, &stop_case).unwrap();
    let mut ticks = 0;
    while vm
        .outcome
        .files
        .get("/work/case-child.txt")
        .is_none_or(|v| v != "entered")
    {
        before(deadline);
        assert!(!vm.done());
        vm.tick(slice);
        ticks += 1;
        assert!(ticks < 256);
    }
    assert!(ticks > 0);
    vm.stop();
    let actual = vm.outcome.clone();
    assert!(
        actual
            .error
            .as_deref()
            .is_some_and(|e| e.starts_with("Stopped"))
    );
    for _ in 0..8 {
        vm.tick(slice);
        assert_eq!(vm.outcome, actual);
        assert!(vm.done());
    }
    (actual, ticks)
}
fn before(deadline: Instant) {
    assert!(Instant::now() < deadline, "Fixed VM capture deadline");
}
fn observe(source: &str, case: &Case, slice: usize, deadline: Instant) -> (Outcome, usize) {
    assert!(case.valid());
    before(deadline);
    match Vm::new(source, case) {
        Err(_) => (shell::run(source, case), 0),
        Ok(mut vm) => {
            let mut ticks = 0;
            while !vm.done() {
                before(deadline);
                vm.tick(slice);
                ticks += 1;
                assert!(ticks <= 1_000_000);
            }
            (vm.outcome, ticks)
        }
    }
}
fn observed(
    out: &mut impl Write,
    kind: &str,
    id: &str,
    source: &str,
    case: &Case,
    deadline: Instant,
) -> Outcome {
    let mut first = None;
    for slice in SLICES {
        let (actual, ticks) = observe(source, case, slice, deadline);
        if let Some(original) = &first {
            assert_eq!(original, &actual, "tick slicing changed {id}");
        } else {
            first = Some(actual.clone());
        }
        row(
            out,
            &json!({"kind":kind,"id":id,"slice":slice,"ticks":ticks,"source_sha256":digest::hash(source.as_bytes()),"actual_evaluator_revision":9,"actual":actual}),
        );
    }
    first.unwrap()
}
fn bounded(path: &Path, cap: usize) -> Vec<u8> {
    let before = fs::symlink_metadata(path).unwrap();
    assert!(before.file_type().is_file() && before.len() <= cap as u64);
    let f = File::open(path).unwrap();
    let opened = f.metadata().unwrap();
    assert!(opened.is_file() && opened.len() == before.len());
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        assert_eq!((opened.dev(), opened.ino()), (before.dev(), before.ino()));
    }
    let mut b = Vec::new();
    f.take(cap as u64 + 1).read_to_end(&mut b).unwrap();
    assert!(b.len() <= cap && b.len() as u64 == opened.len());
    let after = fs::symlink_metadata(path).unwrap();
    assert!(after.file_type().is_file() && after.len() == opened.len());
    assert_eq!(after.modified().unwrap(), opened.modified().unwrap());
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        assert_eq!((after.dev(), after.ino()), (opened.dev(), opened.ino()));
    }
    b
}
fn new_bytes(path: &Path, bytes: &[u8]) {
    assert!(bytes.len() <= PROFILE_CAP);
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut f = options.open(path).unwrap();
    f.write_all(bytes).unwrap();
    f.sync_all().unwrap();
}
fn directory(parent: &Path, name: &str) -> PathBuf {
    let p = parent.join(name);
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(&p).unwrap();
    p
}
fn snapshots(p: &Progress) -> BTreeMap<String, Vec<Record>> {
    p.attempts
        .iter()
        .filter_map(|(id, a)| {
            a.workspace.as_ref().map(|w| {
                (
                    id.clone(),
                    w.script_history
                        .iter()
                        .chain(&w.script_history_more)
                        .cloned()
                        .collect(),
                )
            })
        })
        .collect()
}
fn histories(p: &Progress) {
    let history = snapshots(p);
    let raw8: Value = serde_json::from_slice(ORIGINAL8).unwrap();
    let mut eight = 0;
    for (id, a) in raw8["attempts"].as_object().unwrap() {
        if a["state"]["kind"] == "Script" {
            let record: Record = serde_json::from_value(a["state"]["run"].clone()).unwrap();
            assert_eq!(record.engine_revision, 8);
            assert_eq!(history[id].iter().filter(|r| **r == record).count(), 1);
            eight += 1;
        }
    }
    assert_eq!(eight, 40);

    assert_eq!(
        history
            .values()
            .flatten()
            .filter(|r| r.engine_revision == 6)
            .count(),
        33
    );
    assert_eq!(
        history
            .values()
            .flatten()
            .filter(|r| r.engine_revision == 7)
            .count(),
        37
    );
    for line in OLD7.lines() {
        let old: Old = serde_json::from_str(line).unwrap();
        assert_eq!(old.record.engine_revision, 7);
        assert_eq!(
            history[&old.id]
                .iter()
                .filter(|r| *r == &old.record)
                .count(),
            1
        );
    }
}
fn profile(
    out: &mut impl Write,
    dir: &Path,
    labs: &[pack::Scenario],
    records: &BTreeMap<String, Record>,
    deadline: Instant,
) {
    before(deadline);
    let path = dir.join("progress.json");
    new_bytes(&path, ORIGINAL8);
    new_bytes(&path.with_extension("before-shell-v8.json"), ORIGINAL7);
    let mut p = progress::load(&path, labs).unwrap();
    assert_eq!(
        bounded(&path, PROFILE_CAP),
        ORIGINAL8,
        "reader changed actual8 target"
    );
    let old: Value = serde_json::from_slice(ORIGINAL8).unwrap();
    for a in p.attempts.values_mut() {
        if matches!(a.state, State::Script { .. }) {
            workspace::prepare(a);
        }
    }
    histories(&p);
    let original_history = snapshots(&p);
    // Existing non-script contexts and examples remain unchanged. Later contexts start ungraded.
    for (index, s) in labs.iter().enumerate() {
        let new = !p.attempts.contains_key(&s.id);
        let a = p
            .attempts
            .entry(s.id.clone())
            .or_insert_with(|| Attempt::new(s, index as u64 + 1000));
        if new || matches!(a.state, State::Script { .. }) {
            workspace::prepare(a);
        }
        if let Some(l) = &s.teaching {
            script_teaching::prepare(a, l, false);
        }
        if let Some(r) = records.get(&s.id) {
            a.state = State::Script {
                source: r.source.clone(),
                run: Some(r.clone()),
            };
        }
    }
    // Actual9 Records stay genuine; controlled unfinished text cannot reuse their credit.
    for id in ["shell-handover", "shell-capture-handover"] {
        let a = p.attempts.get_mut(id).unwrap();
        let State::Script { source, .. } = &mut a.state else {
            panic!()
        };
        source.push_str("\nif");
        assert!(!progress::current_solved(
            a,
            labs.iter().find(|s| s.id == id).unwrap()
        ));
        let l = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        l.example_source.push_str("\nif");
    }
    assert_eq!(p.attempts.len(), labs.len());
    histories(&p);
    assert_eq!(snapshots(&p), original_history);
    for (id, a) in old["attempts"].as_object().unwrap() {
        if a["state"]["kind"] != "Script" {
            assert_eq!(
                serde_json::to_value(&p.attempts[id]).unwrap(),
                *a,
                "old non-script changed {id}"
            );
        }
        if a["workspace"]["script_learning"].is_object() {
            let mut wanted = a["workspace"]["script_learning"].clone();
            if ["shell-handover", "shell-capture-handover"].contains(&id.as_str()) {
                wanted["example_source"] =
                    format!("{}\nif", wanted["example_source"].as_str().unwrap()).into();
            }
            assert_eq!(
                serde_json::to_value(&p.attempts[id]).unwrap()["workspace"]["script_learning"],
                wanted,
                "imported example changed beyond explicit draft {id}"
            );
        }
    }
    let portable = serde_json::to_vec_pretty(&p).unwrap();
    assert!(portable.len() <= PROFILE_CAP);
    assert!(
        portable.len().checked_mul(6).unwrap()
            + ORIGINAL8.len() * 10
            + ORIGINAL7.len() * 4
            + 1_000_000
            <= 64 * 1024 * 1024
    );
    progress::decode(&portable, labs).unwrap();
    new_bytes(&dir.join("synthetic-current9-portable.json"), &portable);
    let caller = serde_json::to_value(&p.migration_receipts).unwrap();
    assert!(
        progress::save(&path, &p)
            .unwrap()
            .durability_warning
            .is_none()
    );
    assert_eq!(
        serde_json::to_value(&p.migration_receipts).unwrap(),
        caller,
        "save stamped caller"
    );
    assert_eq!(
        bounded(&path.with_extension("before-shell-v9.json"), PROFILE_CAP),
        ORIGINAL8
    );
    assert_eq!(
        bounded(&path.with_extension("before-shell-v8.json"), PROFILE_CAP),
        ORIGINAL7
    );
    let writer = bounded(&path, PROFILE_CAP);
    new_bytes(&dir.join("writer9-original.json"), &writer);
    let loaded = progress::load(&path, labs).unwrap();
    histories(&loaded);
    assert_eq!(snapshots(&loaded), original_history);
    assert_eq!(
        loaded.migration_receipts.as_ref().unwrap().shell,
        vec![7, 8, 9]
    );
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    progress::save(&path, &loaded).unwrap();
    assert_eq!(bounded(&path, PROFILE_CAP), writer);
    let export = dir.join("portable-backup.json");
    assert!(backup::export(&export, &loaded, labs).unwrap().is_none());
    assert!(
        backup::inspect(&export, labs, &AtomicBool::new(true), &AtomicUsize::new(0))
            .err()
            .unwrap()
            .contains("cancelled")
    );
    assert_eq!(bounded(&path, PROFILE_CAP), writer);
    let candidate =
        backup::inspect(&export, labs, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    assert_eq!(bounded(&path, PROFILE_CAP), writer);
    assert_eq!(snapshots(&candidate.progress), original_history);
    let restored = backup::restore(&path, &candidate, labs).unwrap();
    assert!(restored.durability_warning.is_none());
    assert_eq!(
        bounded(restored.safety_copy.as_ref().unwrap(), PROFILE_CAP),
        writer
    );
    assert_eq!(
        snapshots(&progress::load(&path, labs).unwrap()),
        original_history
    );
    for id in ["shell-handover", "shell-capture-handover"] {
        let a = &candidate.progress.attempts[id];
        let State::Script { source, .. } = &a.state else {
            panic!()
        };
        assert!(source.ends_with("\nif"));
        assert!(
            a.workspace
                .as_ref()
                .unwrap()
                .script_learning
                .as_ref()
                .unwrap()
                .example_source
                .ends_with("\nif")
        );
        assert!(!progress::current_solved(
            a,
            labs.iter().find(|s| s.id == id).unwrap()
        ));
    }
    let empty = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let reset = directory(dir, "reset-first");
    let reset_path = reset.join("progress.json");
    new_bytes(&reset_path, ORIGINAL8);
    progress::save(&reset_path, &empty).unwrap();
    assert_eq!(
        bounded(
            &reset_path.with_extension("before-shell-v9.json"),
            PROFILE_CAP
        ),
        ORIGINAL8
    );
    let occupied = directory(dir, "occupied-slot");
    let target = occupied.join("progress.json");
    new_bytes(&target, ORIGINAL8);
    let mut conflict = ORIGINAL8.to_vec();
    conflict.push(b'\n');
    let slot = target.with_extension("before-shell-v9.json");
    new_bytes(&slot, &conflict);
    assert!(progress::save(&target, &empty).is_err());
    assert_eq!(bounded(&target, PROFILE_CAP), ORIGINAL8);
    assert_eq!(bounded(&slot, PROFILE_CAP), conflict);
    row(
        out,
        &json!({"kind":"profile-evidence","synthetic_only":true,"catalog_contexts":labs.len(),"current9_actual_records":43,"genuine6_history_records":33,"genuine7_history_records":37,"genuine8_history_records":40,"ungraded_assignment_drafts":["shell-handover","shell-capture-handover"],"ungraded_example_drafts":["shell-handover","shell-capture-handover"],"writer_receipts":[7,8,9],"before9_exact_original8":true,"before8_exact_original7":true,"repeat_write_exact":true,"preview_cancel_preserve_target":true,"restore_exact_current9_safety_copy":true,"reset_first_exact_original8":true,"occupied_whitespace_slot_rejected":true,"future10_not_implemented":true}),
    );
}
fn guard() -> PathBuf {
    assert_eq!(
        shell::EVALUATOR_REVISION,
        9,
        "Only original public9 before future10"
    );
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    assert!(
        args.is_empty() || args == [std::ffi::OsString::from("--identity")],
        "Only fixed identity mode, no source/path arguments"
    );
    assert_eq!(
        std::env::var("DLMS_FIXED_AUTOMATION_CHILD").unwrap(),
        "before10-public9-r1"
    );
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .canonicalize()
        .unwrap();
    assert!(root.join(".git").is_dir(), "Actual capture is MAIN only");
    assert_eq!(std::env::current_dir().unwrap(), root);
    assert_eq!(
        std::env::var("CARGO_TARGET_DIR").unwrap(),
        root.join("target").to_str().unwrap()
    );
    let exe = std::env::current_exe().unwrap().canonicalize().unwrap();
    assert_eq!(
        exe,
        root.join("target/debug/examples/automation_before10_capture")
    );
    let marker: Value = serde_json::from_slice(&bounded(
        &root.join(".build-artifacts/active-command.json"),
        16384,
    ))
    .unwrap();
    assert_eq!(marker["state"], "running");
    let proc = String::from_utf8(bounded_proc()).unwrap();
    let fields: Vec<_> = proc
        .rsplit_once(')')
        .unwrap()
        .1
        .split_whitespace()
        .collect();
    let parent: u64 = fields[1].parse().unwrap();
    let group: u64 = fields[2].parse().unwrap();
    assert_eq!(marker["child_pid"].as_u64().unwrap(), parent);
    assert_eq!(group, parent);
    assert_eq!(std::env::var("CARGO_BUILD_JOBS").unwrap(), "2");
    assert_eq!(std::env::var("RUST_TEST_THREADS").unwrap(), "2");
    let dir = PathBuf::from(std::env::var("DLMS_FIXED_AUTOMATION_OUTPUT").unwrap());
    assert!(dir.starts_with(root.join(".build-artifacts/reviews/bash-safe-automation/captures")));
    assert_eq!(dir.canonicalize().unwrap(), dir);
    assert!(dir.is_dir());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(dir.metadata().unwrap().permissions().mode() & 0o777, 0o700);
    }
    assert_eq!(dir.file_name().unwrap(), "vm-profile");
    assert!(fs::read_dir(&dir).unwrap().next().is_none());
    dir
}
fn bounded_proc() -> Vec<u8> {
    // procfs size is0; finite same-file read, no command-line/private process observation.
    let mut b = Vec::new();
    File::open("/proc/self/stat")
        .unwrap()
        .take(4097)
        .read_to_end(&mut b)
        .unwrap();
    assert!(b.len() <= 4096);
    b
}
fn main() {
    let dir = guard();
    let deadline = Instant::now() + Duration::from_secs(150);
    let mut out = Limited {
        inner: io::stdout().lock(),
        count: 0,
        cap: STREAM_CAP,
    };
    let compiled: Vec<_> = attestation::FILES
        .iter()
        .map(|(name, bytes)| json!({"file":name,"bytes":bytes.len(),"sha256":digest::hash(bytes)}))
        .collect();
    let pin: Authored = serde_json::from_str(AUTHORED).unwrap();
    let labs = pack::bundled().unwrap();
    assert_eq!(labs.len(), pin.catalog.len());
    assert_eq!(pin.scripts.len(), 43);
    let linked_catalog = labs
        .iter()
        .map(|s| (s.id.clone(), s.version))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(linked_catalog, pin.catalog);
    let linked_scripts: Vec<_> = labs
        .iter()
        .filter_map(|s| {
            let Goal::Script { cases } = &s.goal else {
                return None;
            };
            let example = s.teaching.as_ref().unwrap();
            assert_eq!(s.solution.len(), 1);
            Some(Script {
                id: s.id.clone(),
                version: s.version,
                unit: s.unit.clone(),
                source: s.solution[0].clone(),
                cases: cases.clone(),
                example_source: example.example.clone(),
                example_case: example.fixture.clone(),
            })
        })
        .collect();
    assert_eq!(
        serde_json::to_value(&linked_scripts).unwrap(),
        serde_json::to_value(&pin.scripts).unwrap()
    );
    row(
        &mut out,
        &json!({"kind":"identity","revision":9,"compiled_manifest_dir":env!("CARGO_MANIFEST_DIR"),"compiled_inputs":compiled,"linked_catalog":linked_catalog,"linked_scripts":linked_scripts,"probe_source_sha256":digest::hash(include_bytes!("automation_before10_capture.rs")),"digest_source_sha256":digest::hash(include_bytes!("support/automation_digest.rs")),"attestation_source_sha256":digest::hash(include_bytes!("support/automation_attestation.rs"))}),
    );
    if std::env::args_os().nth(1).is_some() {
        out.flush().unwrap();
        return;
    }
    let mut records = BTreeMap::new();
    let mut case_count = 0;
    for s in &pin.scripts {
        let lab = labs.iter().find(|l| l.id == s.id).unwrap();
        let Goal::Script { cases } = &lab.goal else {
            panic!()
        };
        assert_eq!(lab.version, s.version);
        assert_eq!(lab.unit, s.unit);
        assert_eq!(lab.solution.as_slice(), std::slice::from_ref(&s.source));
        assert_eq!(cases, &s.cases);
        let lesson = lab.teaching.as_ref().unwrap();
        assert_eq!(lesson.example, s.example_source);
        assert_eq!(lesson.fixture, s.example_case);
        let results = cases
            .iter()
            .enumerate()
            .map(|(i, c)| {
                case_count += 1;
                let o = observed(
                    &mut out,
                    "authored-case",
                    &format!("{}/{}", s.id, i),
                    &s.source,
                    c,
                    deadline,
                );
                assert!(c.accepts(&o));
                o
            })
            .collect();
        let record = Record {
            engine_revision: 9,
            source: s.source.clone(),
            results,
        };
        assert!(record.valid());
        row(
            &mut out,
            &json!({"kind":"authored-record","id":s.id,"scenario_version":s.version,"cases":cases,"record":record}),
        );
        records.insert(s.id.clone(), record);
        let example = observed(
            &mut out,
            "worked-example",
            &s.id,
            &s.example_source,
            &s.example_case,
            deadline,
        );
        assert!(s.example_case.accepts(&example));
    }
    // Independently observed malformed draft outcomes anchor explicit backup rechecks.
    for id in ["shell-handover", "shell-capture-handover"] {
        let s = pin.scripts.iter().find(|s| s.id == id).unwrap();
        let source = format!("{}\nif", s.source);
        for (i, case) in s.cases.iter().enumerate() {
            observed(
                &mut out,
                "unfinished-assignment",
                &format!("{id}/{i}"),
                &source,
                case,
                deadline,
            );
        }
        observed(
            &mut out,
            "unfinished-example",
            id,
            &format!("{}\nif", s.example_source),
            &s.example_case,
            deadline,
        );
    }
    let old8: Value = serde_json::from_slice(ORIGINAL8).unwrap();
    for (id, attempt) in old8["attempts"].as_object().unwrap() {
        if let Some(source) = attempt["workspace"]["script_learning"]["example_source"].as_str() {
            let s = pin.scripts.iter().find(|s| &s.id == id).unwrap();
            if source != s.example_source {
                observed(
                    &mut out,
                    "imported-example",
                    id,
                    source,
                    &s.example_case,
                    deadline,
                );
            }
        }
    }
    assert_eq!(case_count, 282);
    let inputs: FixedInputs = serde_json::from_str(INPUTS).unwrap();
    assert_eq!(inputs.cases.len(), 28);
    for c in inputs.cases {
        observed(
            &mut out,
            "fixed-automation-public9",
            &c.id,
            &c.source,
            &c.input.case(&c.id),
            deadline,
        );
    }
    let boundaries: FixedInputs = serde_json::from_str(VM_BOUNDARIES).unwrap();
    assert_eq!(boundaries.cases.len(), 8);
    for c in boundaries.cases {
        observed(
            &mut out,
            "vm-boundary-public9",
            &c.id,
            &c.source,
            &c.input.case(&c.id),
            deadline,
        );
    }
    for slice in SLICES {
        let (actual, ticks) = observe_stop(slice, deadline);
        row(
            &mut out,
            &json!({"kind":"stopped-ungraded","slice":slice,"ticks_before_stop":ticks,"source":STOP_SOURCE,"actual":actual,"record_created":false}),
        );
    }
    profile(&mut out, &dir, &labs, &records, deadline);
    row(
        &mut out,
        &json!({"kind":"finished","actual_revision":9,"catalog_contexts":pin.catalog.len(),"authored_scripts":43,"authored_cases":282,"worked_examples":43,"fixed_automation_inputs":28,"vm_boundary_inputs":8,"slices":SLICES,"claims":"Actual public9 observations only; no emptyIFS/pipefail10 capability, learner execution, native acceptance or fabricated credit"}),
    );
    out.flush().unwrap();
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_capture_stop_path_observes_absolute_virtual_file_and_cannot_resume() {
        for slice in SLICES {
            let (actual, ticks) = observe_stop(slice, Instant::now() + Duration::from_secs(5));
            assert!(ticks > 0);
            assert_eq!(actual.files.get("/work/case-child.txt").unwrap(), "entered");
            assert!(actual.error.as_deref().unwrap().starts_with("Stopped"));
        }
    }
    #[test]
    fn limited_writer_rejects_before_allocating_or_writing_excess() {
        let mut w = Limited {
            inner: Vec::new(),
            count: 0,
            cap: 2,
        };
        w.write_all(b"ab").unwrap();
        assert!(w.write_all(b"c").is_err());
        assert_eq!(w.inner, b"ab");
    }
    #[test]
    fn fixed_inputs_remain_unobserved_and_have_no_expected_results() {
        let value: Value = serde_json::from_str(INPUTS).unwrap();
        assert_eq!(value["cases"].as_array().unwrap().len(), 28);
        for c in value["cases"].as_array().unwrap() {
            assert!(c["actual_bash_5_3"].is_null());
            assert!(c["actual_public9_before10"].is_null());
            assert!(c["accepted_after10"].is_null());
            assert!(c.get("expected").is_none());
        }
    }
    #[test]
    fn capture_entry_rejects_missing_guard_without_running_vm() {
        assert!(std::panic::catch_unwind(guard).is_err());
    }
}
