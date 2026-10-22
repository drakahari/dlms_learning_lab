use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    shell::{self, Case},
    theme,
};
use std::collections::BTreeMap;
fn fixture() -> Case {
    Case {
        name: "fixture".into(),
        args: vec!["two words".into(), "".into()],
        stdin: "  keep  words \\n\n".into(),
        env: BTreeMap::new(),
        files: BTreeMap::new(),
        stdout: String::new(),
        stderr: String::new(),
        status: 0,
        after: BTreeMap::new(),
    }
}
#[test]
fn all_authored_scripts_grade_behavior_and_break_after_edit() {
    let labs = pack::bundled().unwrap();
    assert_eq!(labs.len(), 64);
    let scripts: Vec<_> = labs.iter().filter(|s| s.track == "Bash/Shell").collect();
    assert_eq!(scripts.len(), 29);
    for s in scripts {
        let Goal::Script { cases } = &s.goal else {
            panic!()
        };
        let State::Script { mut source, .. } = s.initial.clone() else {
            panic!()
        };
        let mut record = shell::Record {
            engine_revision: dlms_learning_lab::shell::EVALUATOR_REVISION,
            source: source.clone(),
            results: cases.iter().map(|c| shell::run(&source, c)).collect(),
        };
        assert!(
            !shell::checks(&source, &Some(record), cases)
                .iter()
                .all(|c| c.0)
        );
        source = s.solution[0].clone();
        record = shell::Record {
            engine_revision: dlms_learning_lab::shell::EVALUATOR_REVISION,
            source: source.clone(),
            results: cases.iter().map(|c| shell::run(&source, c)).collect(),
        };
        assert!(
            shell::checks(&source, &Some(record.clone()), cases)
                .iter()
                .all(|c| c.0),
            "{}: {:?}",
            s.id,
            record.results
        );
        let state = State::Script {
            source: source.clone(),
            run: Some(record.clone()),
        };
        assert!(engine::solved(&state, &s.goal));
        source.push_str("\nprintf 'extra\\n'\n");
        assert!(
            !shell::checks(&source, &Some(record), cases)
                .iter()
                .any(|c| c.0)
        );
    }
}
#[test]
fn equivalent_scripts_and_partial_cases() {
    let labs = pack::bundled().unwrap();
    let s = labs.iter().find(|s| s.id == "shell-validation").unwrap();
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    let alternate = "test -n \"$1\" || { unsupported; }";
    assert!(shell::run(alternate, &cases[0]).error.is_some());
    let alternate = "if test -n \"$1\"; then\n echo \"Destination: $1\"\nelse\n echo 'Destination required' >&2\n exit 2\nfi";
    for c in cases {
        assert!(c.accepts(&shell::run(alternate, c)), "{}", c.name)
    }
    let partial = "printf 'Destination: %s\\n' \"$1\"";
    let results: Vec<_> = cases
        .iter()
        .map(|c| c.accepts(&shell::run(partial, c)))
        .collect();
    assert!(results.contains(&true) && results.contains(&false));
}
#[test]
fn quotations_expansion_status_and_redirects() {
    let c = fixture();
    let r = shell::run(
        "x='a  b'\nprintf '<%s>\\n' \"$x\" $x \"$1\" \"$2\"\nfalse\nprintf 'status=%s count=%s\\n' \"$?\" \"$#\"",
        &c,
    );
    assert_eq!(r.error, None);
    assert_eq!(
        r.stdout,
        "<a  b>\n<a>\n<b>\n<two words>\n<>\nstatus=1 count=2\n"
    );
    let r = shell::run(
        "printf 'first\\n' > a\nprintf 'second\\n' >> a\ncat a | grep -F second\nprintf 'bad\\n' >&2\n",
        &c,
    );
    assert_eq!(r.stdout, "second\n");
    assert_eq!(r.stderr, "bad\n");
    assert_eq!(r.files["/work/a"], "first\nsecond\n");
    assert_eq!(r.status, 0);
}
#[test]
fn loops_control_and_limits() {
    let c = fixture();
    let r = shell::run(
        "for x in \"$@\"; do printf '<%s>\\n' \"$x\"; done\nif false; then echo no; fi\nprintf '%s\\n' \"$?\"\nwhile false; do echo no; done\nprintf '%s\\n' \"$?\"",
        &c,
    );
    assert_eq!(r.error, None);
    assert_eq!(r.stdout, "<two words>\n<>\n0\n0\n");
    let r = shell::run("while true; do :; done", &c);
    assert!(r.error.unwrap().contains("Execution limit"));
    let r = shell::run(
        "while true; do printf 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\\n'; done",
        &c,
    );
    assert!(r.error.unwrap().contains("Output limit"));
    let huge_pipeline = std::iter::repeat_n(":", 17).collect::<Vec<_>>().join(" | ");
    assert!(
        shell::run(&huge_pipeline, &c)
            .error
            .unwrap()
            .contains("16 stages")
    );
    let huge_assignment = (0..129)
        .map(|n| format!("v{n}=x"))
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        shell::run(&huge_assignment, &c)
            .error
            .unwrap()
            .contains("limit")
    );
    let mut vm = shell::Vm::new("while true; do :; done", &c).unwrap();
    vm.tick(1);
    assert!(!vm.done());
    vm.stop();
    assert!(vm.done());
    assert!(vm.outcome.error.unwrap().contains("Stopped"));
}
#[test]
fn unsupported_and_isolation() {
    for source in [
        "if true; then fi",
        "while true; do; done",
        "for x in a; do; done",
        "echo a;; echo b",
        "echo a && ; echo b",
        "id",
        "bash -c 'echo bad'",
        "cat /etc/passwd",
        "cat ../secret",
        "echo $(id)",
        "echo `id`",
        "echo ~",
        "echo /etc/*",
        "x=(a b)",
        "eval echo hi",
        "echo yes &",
        "cat <<EOF",
        "printf '%d' 3",
        "exec sh",
    ] {
        assert!(shell::run(source, &fixture()).error.is_some(), "{source}");
    }
    let mut c = fixture();
    c.env
        .insert("TRAP".into(), "$(touch /tmp/not-allowed)".into());
    let r = shell::run("printf '%s\\n' \"$TRAP\"", &c);
    assert_eq!(r.stdout, "$(touch /tmp/not-allowed)\n");
    assert_eq!(r.error, None);
}
#[test]
fn resume_and_selected_reset_keep_old_attempts() {
    let labs = pack::bundled().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &labs {
        p.attempts.insert(s.id.clone(), Attempt::new(s, 1));
    }
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("progress-v1.json");
    progress::save(&path, &p).unwrap();
    let mut p = progress::load(&path, &labs).unwrap();
    if let State::Script { source, .. } = &mut p.attempts.get_mut("shell-handover").unwrap().state {
        *source = "# unfinished\nprintf '".into();
    }
    progress::save(&path, &p).unwrap();
    let mut resumed = progress::load(&path, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&resumed).unwrap()
    );
    let old = serde_json::to_value(&resumed.attempts["ccna-ipv4"]).unwrap();
    let s = labs.iter().find(|s| s.id == "shell-handover").unwrap();
    resumed.attempts.insert(s.id.clone(), Attempt::new(s, 2));
    assert_eq!(
        old,
        serde_json::to_value(&resumed.attempts["ccna-ipv4"]).unwrap()
    );
}
#[test]
fn every_palette_has_readable_native_roles_and_preferences_roundtrip() {
    assert_eq!(theme::inventory().len(), 28);
    let mut ids = std::collections::BTreeSet::new();
    let ctx = eframe::egui::Context::default();
    for t in theme::inventory() {
        assert!(ids.insert(&t.id));
        let r = t.roles;
        for bg in [r.background, r.panel, r.input, r.control, r.hover] {
            assert!(theme::contrast(r.text, bg) >= 6.99, "{} text", t.id);
            for c in [r.muted, r.accent, r.error, r.success] {
                assert!(theme::contrast(c, bg) >= 4.49, "{} role", t.id)
            }
            assert!(theme::contrast(r.border, bg) >= 2.99);
        }
        assert!(theme::contrast(r.on_accent, r.accent) >= 4.49);
        theme::apply(&ctx, &t.id);
        let v = ctx.style().visuals.clone();
        assert_eq!(v.disabled_alpha, 1.);
        assert!(
            theme::contrast(
                v.widgets.inactive.fg_stroke.color,
                v.widgets.inactive.bg_fill
            ) >= 6.99
        );
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("preferences-v1.json");
        theme::Preferences {
            version: 1,
            theme: t.id.clone(),
            text_scale: 1.5,
        }
        .save(&path)
        .unwrap();
        let loaded = theme::Preferences::load(&path).unwrap();
        assert_eq!(loaded.theme, t.id);
        assert_eq!(loaded.text_scale, 1.5);
    }
}

#[test]
fn limits_keep_records_resumable_and_invalid_packs_are_rejected() {
    let c = fixture();
    for source in [
        "while true; do printf 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\\n'; done",
        "x=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\nwhile true; do x=$x$x; done",
    ] {
        let result = shell::run(source, &c);
        assert!(result.error.is_some());
        assert!(
            shell::Record {
                engine_revision: dlms_learning_lab::shell::EVALUATOR_REVISION,
                source: source.into(),
                results: vec![result]
            }
            .valid()
        );
    }
    let mut pack_value: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/shell-unit-v1.json")).unwrap();
    pack_value["scenarios"][0]["prerequisites"] = serde_json::json!(["shell-capstone"]);
    assert!(
        pack::load(&pack_value.to_string())
            .unwrap_err()
            .contains("cyclic")
    );
    let mut pack_value: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/shell-unit-v1.json")).unwrap();
    pack_value["scenarios"][0]["goal"]["cases"] = serde_json::json!([]);
    assert!(pack::load(&pack_value.to_string()).is_err());
    let mut pack_value: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/shell-unit-v1.json")).unwrap();
    pack_value["scenarios"][0]["solution"] = serde_json::json!(["echo canned"]);
    assert!(pack::load(&pack_value.to_string()).is_err());
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("prefs.json");
    std::fs::write(
        &p,
        b"{\"version\":99,\"theme\":\"unknown\",\"text_scale\":1.0}",
    )
    .unwrap();
    let original = std::fs::read(&p).unwrap();
    assert!(theme::Preferences::load(&p).is_err());
    assert_eq!(std::fs::read(&p).unwrap(), original);
}
