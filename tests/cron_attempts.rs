//! New optional metadata must not alter old-wire serialization or let an old
//! activity retain a solved result when a foreign scheduling payload is present.
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use dlms_learning_lab::{
    engine, pack,
    progress::{self, Attempt, Progress},
    workspace,
};
fn technical() -> pack::Scenario {
    pack::load(include_str!("fixtures/cron-workbench-v1.json"))
        .unwrap()
        .remove(0)
}
#[test]
fn old_catalog_success_rejects_cron_payload_without_mutation_or_invented_credit() {
    let cron = technical();
    let engine::Goal::Cron { definition } = &cron.goal else {
        unreachable!()
    };
    let session = dlms_learning_lab::cron_workspace::Session::new(definition).unwrap();
    let learning =
        dlms_learning_lab::cron_teaching::Learning::new(cron.cron_teaching.as_ref().unwrap())
            .unwrap();
    let catalog = pack::bundled().unwrap();
    // This is the immutable pre-Cron154 prefix, not a current catalog count.
    // Existing catalog baseline tests separately pin the exact older147 rows.
    assert_eq!(one_off_boundary::before_len(&catalog), 176);
    assert_eq!(
        catalog[147..154]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        [
            "shell-command-options",
            "shell-command-dispatch",
            "shell-command-plan",
            "lpic2-nfs-reference",
            "lpic2-nfs-publication",
            "lpic2-nfs-handover",
            "lpic2-nfs-capstone"
        ]
    );
    assert_eq!(
        catalog[154..160]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        [
            "ccna-lacp-initiation",
            "ccna-lacp-member-service",
            "ccna-lacp-resilient-handover",
            "shell-optional-policy",
            "shell-optional-report",
            "shell-optional-handover"
        ]
    );
    for s in &catalog[..160] {
        let mut a = Attempt::new(s, 31);
        for command in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, command)
                .unwrap_or_else(|e| panic!("{}: {e}", s.id));
        }
        // Script editing and running are separate operations. Execute every
        // authored deterministic case in the in-process VM, as the normal Run
        // path does; installing source alone must not imply completion.
        if let (engine::State::Script { source, run }, engine::Goal::Script { cases }) =
            (&mut a.state, &s.goal)
        {
            *run = Some(dlms_learning_lab::shell::Record {
                engine_revision: dlms_learning_lab::shell::EVALUATOR_REVISION,
                source: source.clone(),
                results: cases
                    .iter()
                    .map(|case| dlms_learning_lab::shell::run(source, case))
                    .collect(),
            });
        }
        assert!(progress::current_solved(&a, s), "{} baseline", s.id);
        let wire = serde_json::to_string(&a).unwrap();
        assert!(!wire.contains("cron_session") && !wire.contains("cron_learning"));
        for practice in [false, true] {
            let mut a = a.clone();
            let w = a
                .workspace
                .get_or_insert_with(workspace::WorkspaceData::new);
            if practice {
                w.cron_learning = Some(learning.clone())
            } else {
                w.cron_session = Some(session.clone())
            }
            assert!(w.valid());
            let before = serde_json::to_vec(&a).unwrap();
            assert!(
                !progress::current_solved(&a, s),
                "{} foreign practice={practice}",
                s.id
            );
            workspace::prepare(&mut a);
            assert_eq!(serde_json::to_vec(&a).unwrap(), before);
            let mut p = Progress {
                schema_version: 1,
                clock: 31,
                ..Progress::default()
            };
            p.attempts.insert(s.id.clone(), a);
            let raw = serde_json::to_vec(&p).unwrap();
            assert!(
                progress::decode(&raw, &catalog).is_err(),
                "{} decode foreign practice={practice}",
                s.id
            );
        }
    }
}
#[test]
fn new_drafts_restore_and_bad_output_remains_admitted_without_success() {
    let s = technical();
    let mut a = Attempt::new(&s, 5);
    dlms_learning_lab::cron_workspace::budget::prepare(&mut a, &s).unwrap();
    let mut p = Progress {
        schema_version: 1,
        clock: 5,
        ..Progress::default()
    };
    p.attempts.insert(s.id.clone(), a);
    let mut p =
        progress::decode(&serde_json::to_vec(&p).unwrap(), std::slice::from_ref(&s)).unwrap();
    let a = p.attempts.get_mut(&s.id).unwrap();
    assert!(!progress::current_solved(a, &s));
    let engine::State::Cron { simulation } = &mut a.state else {
        unreachable!()
    };
    simulation
        .files
        .insert("/work/source".into(), "damaged supplied input\n".into());
    let bytes = serde_json::to_vec(&p).unwrap();
    let restored = progress::decode(&bytes, std::slice::from_ref(&s)).unwrap();
    assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
    assert!(!progress::current_solved(&restored.attempts[&s.id], &s));
}
