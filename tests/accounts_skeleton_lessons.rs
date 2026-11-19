//! Actual authored state, alternate methods and negative outcomes.
use dlms_learning_lab::{
    accounts::{self, Kind},
    accounts_template::Step,
    accounts_template_workspace as work,
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
fn lab() -> pack::Scenario {
    static LAB: std::sync::OnceLock<pack::Scenario> = std::sync::OnceLock::new();
    LAB.get_or_init(|| {
        // Keep actual prerequisite validation while this new pack is unregistered.
        let old = pack::accepted286().unwrap();
        let mut texts: Vec<_> = old
            .chunks(8)
            .map(|chunk| {
                serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":chunk}))
                    .unwrap()
            })
            .collect();
        texts.push(include_str!("../scenarios/account-skeleton-lessons-v1.json").into());
        pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>())
            .unwrap()
            .into_iter()
            .find(|s| s.id == "lpic1-skeleton-handover")
            .unwrap()
    })
    .clone()
}

#[test]
fn registered_lesson_preserves_all286_objects89_units_and_prerequisite_order() {
    let old = pack::accepted286().unwrap();
    let now = pack::accepted287().unwrap();
    assert_eq!((old.len(), now.len()), (286, 287));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&now[..286]).unwrap()
    );
    let before = dlms_learning_lab::course::accepted286(&old).unwrap();
    let units = dlms_learning_lab::course::accepted287(&now).unwrap();
    assert_eq!((before.len(), units.len()), (89, 90));
    let retained: Vec<_> = units
        .iter()
        .filter(|u| u.id != "linux-account-defaults")
        .collect();
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "linux-account-defaults")
        .unwrap();
    for id in ["local-account-lifecycle", "linux-file-locations"] {
        assert!(units.iter().position(|u| u.id == id).unwrap() < at);
    }
    assert_eq!(
        serde_json::to_value(now.iter().find(|s| s.id == "lpic1-skeleton-handover").unwrap()).unwrap(),
        serde_json::to_value(lab()).unwrap()
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| now.iter().filter(|s| s.track == t).count()),
        [72, 96, 45, 74]
    );
}
fn fresh(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    work::prepare(&mut a, s, true).unwrap();
    a
}
// Test-fixture helper only; production constructors must NEVER fill the assignment.
fn steps(a: &mut Attempt, s: &pack::Scenario, items: &[Step]) {
    let source = work::source(s).unwrap().unwrap();
    let State::Accounts { simulation } = &mut a.state else {
        panic!()
    };
    let d = a
        .workspace
        .as_mut()
        .unwrap()
        .accounts_template_drafts
        .as_mut()
        .unwrap();
    for item in items {
        let key = item.path().to_string();
        d.selected = key.clone();
        let row = d.leaves.get_mut(&key).unwrap();
        match item {
            Step::Save { contents, mode, .. } => {
                row.included = true;
                row.contents = contents.clone();
                row.mode = format!("{mode:04o}");
            }
            Step::Remove { .. } => row.included = false,
        }
        d.save_selected(source.initial, simulation).unwrap();
    }
}
fn commands(a: &mut Attempt, s: &pack::Scenario, items: &[String]) {
    for c in items {
        engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
    }
}
fn native(s: &pack::Scenario) -> Vec<Step> {
    s.accounts_teaching
        .as_ref()
        .unwrap()
        .template_teaching
        .as_ref()
        .unwrap()
        .assignment
        .clone()
}
fn finished(s: &pack::Scenario) -> Attempt {
    let mut a = fresh(s);
    steps(&mut a, s, &native(s));
    commands(&mut a, s, &s.solution);
    assert!(progress::current_solved(&a, s));
    a
}
#[test]
fn source_gate_matches_exact_prerequisites_native_actions_and_unhidden_starter() {
    let s = lab();
    assert_eq!(s.id, "lpic1-skeleton-handover");
    assert_eq!(s.unit, "linux-account-defaults");
    assert_eq!(
        s.prerequisites,
        ["lpic1-account-onboarding", "lpic1-fhs-locate-active-files"]
    );
    let source = work::source(&s).unwrap().unwrap();
    assert_eq!(source.initial.policy().editable_skeleton_leaves.len(), 3);
    assert_eq!(source.teaching.assignment.len(), 3);
    let a = fresh(&s);
    assert!(!progress::current_solved(&a, &s));
    assert_ne!(
        a.workspace
            .as_ref()
            .unwrap()
            .accounts_template_drafts
            .as_ref()
            .unwrap()
            .leaves["/etc/skel/.profile"]
            .contents,
        "TEAM_ZONE=north\nexport TEAM_ZONE\n"
    );
    for c in &s.solution {
        assert!(accounts::parse(c).is_ok());
    }
}
#[test]
fn canonical_and_genuine_alternate_account_commands_reach_full_source_and_home_outcomes() {
    let s = lab();
    for cmds in [s.solution.clone(),vec!["groupadd --gid 2600 support".into(),"useradd -m -N -u 1600 -g support -G audit,audit -d /home/rhea -s /bin/bash -k /etc/skel rhea".into()],vec!["groupadd -g 2600 support".into(),"useradd -N -u 1600 -g support -d /home/rhea -s /bin/bash -m rhea".into(),"usermod -a -G audit rhea".into()]]{let mut a=fresh(&s);steps(&mut a,&s,&native(&s));commands(&mut a,&s,&cmds);assert!(progress::current_solved(&a,&s));assert!(engine::checks(&a.state,&s.goal).iter().all(|r|r.0));}
}
#[test]
fn wrong_template_modes_text_and_note_retention_fail_behaviorally() {
    let s = lab();
    for kind in 0..4 {
        let mut a = fresh(&s);
        let mut edits = native(&s);
        match kind {
            0 => {
                edits.pop();
            }
            1 => {
                if let Step::Save { mode, .. } = &mut edits[0] {
                    *mode = 0o644;
                }
            }
            2 => {
                if let Step::Save { contents, .. } = &mut edits[1] {
                    contents.pop();
                }
            }
            _ => {
                if let Step::Save { contents, .. } = &mut edits[0] {
                    *contents = "DEMO_ZONE=harbor\nexport DEMO_ZONE\n".into();
                }
            }
        }
        steps(&mut a, &s, &edits);
        commands(&mut a, &s, &s.solution);
        assert!(
            !progress::current_solved(&a, &s),
            "wrong authored approach {kind}"
        );
        assert!(engine::checks(&a.state, &s.goal).iter().any(|r| !r.0));
    }
}
#[test]
fn record_only_other_skeleton_and_unedited_source_fail_without_inventing_success() {
    let s = lab();
    let mut a = fresh(&s);
    commands(&mut a, &s, &s.solution);
    assert!(!progress::current_solved(&a, &s));
    for flag in ["-M", "-m -k /etc/skel-field"] {
        let mut a = fresh(&s);
        steps(&mut a, &s, &native(&s));
        commands(
            &mut a,
            &s,
            &[
                "groupadd -g 2600 support".into(),
                format!(
                    "useradd -N -u 1600 -g support -G audit -d /home/rhea -s /bin/bash {flag} rhea"
                ),
            ],
        );
        assert!(!progress::current_solved(&a, &s));
    }
}
#[test]
fn partial_native_work_and_primary_group_do_not_earn_account_or_home_credit() {
    let s = lab();
    let mut a = fresh(&s);
    steps(&mut a, &s, &native(&s));
    commands(&mut a, &s, &["groupadd -g 2600 support".into()]);
    let rows = engine::checks(&a.state, &s.goal);
    assert!(rows.iter().any(|r| r.0));
    assert!(rows.iter().any(|r| !r.0));
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn early_copy_is_not_retroactively_fixed_and_trial_recreation_is_legitimate_retry() {
    let s = lab();
    let mut a = fresh(&s);
    commands(&mut a, &s, &s.solution);
    let State::Accounts { simulation } = &a.state else {
        panic!()
    };
    let copied = simulation.configuration().entries["/home/rhea/.profile"].clone();
    steps(&mut a, &s, &native(&s));
    let State::Accounts { simulation } = &a.state else {
        panic!()
    };
    assert_eq!(
        simulation.configuration().entries["/home/rhea/.profile"],
        copied
    );
    assert!(!progress::current_solved(&a, &s));
    commands(
        &mut a,
        &s,
        &["userdel -r rhea".into(), s.solution[1].clone()],
    );
    assert!(progress::current_solved(&a, &s));
}
#[test]
fn pending_edits_and_late_wrong_save_invalidate_current_success_and_retry_remains_possible() {
    let s = lab();
    let mut a = finished(&s);
    let d = a
        .workspace
        .as_mut()
        .unwrap()
        .accounts_template_drafts
        .as_mut()
        .unwrap();
    d.selected = "/etc/skel/.profile".into();
    d.leaves.get_mut("/etc/skel/.profile").unwrap().contents = "Late unsaved change\n".into();
    assert!(!progress::current_solved(&a, &s));
    let bad = Step::Save {
        path: "/etc/skel/.profile".into(),
        contents: "Late wrong source\n".into(),
        mode: 0o640,
    };
    steps(&mut a, &s, &[bad]);
    assert!(!progress::current_solved(&a, &s));
    steps(&mut a, &s, &native(&s));
    assert!(progress::current_solved(&a, &s));
}
#[test]
fn whole_preservation_rejects_changed_unrelated_home_text() {
    let s = lab();
    let mut a = finished(&s);
    let State::Accounts { simulation } = &mut a.state else {
        panic!()
    };
    let mut c = simulation.configuration().clone();
    c.entries.get_mut("/home/pax/.profile").unwrap().data = Kind::Text {
        contents: "Accidentally replaced personal defaults\n".into(),
    };
    *simulation = accounts::State::new(simulation.policy().clone(), c).unwrap();
    assert!(!progress::current_solved(&a, &s));
    assert!(engine::checks(&a.state, &s.goal).iter().any(|r| !r.0));
}
#[test]
fn example_phases_diagnose_copy_once_without_touching_assignment_or_awarding_credit() {
    let s = lab();
    let a = fresh(&s);
    let untouched = serde_json::to_value(&a).unwrap();
    let l = s.accounts_teaching.as_ref().unwrap();
    l.validate().unwrap();
    let t = l.template_teaching.as_ref().unwrap();
    let mut example = l.fixture.clone();
    dlms_learning_lab::accounts_template::apply_phase(&l.fixture, &mut example, &t.example)
        .unwrap();
    for c in &l.example {
        dlms_learning_lab::accounts_teaching::execute_in_context(&l.fixture, &mut example, c)
            .unwrap();
    }
    assert!(
        accounts::checks(&l.fixture, &example, &l.outcomes)
            .iter()
            .all(|r| r.0)
    );
    dlms_learning_lab::accounts_template::apply_phase(&l.fixture, &mut example, &t.try_steps)
        .unwrap();
    for c in &l.try_actions {
        dlms_learning_lab::accounts_teaching::execute_in_context(&l.fixture, &mut example, c)
            .unwrap();
    }
    assert!(
        accounts::checks(&l.fixture, &example, &l.try_outcomes)
            .iter()
            .all(|r| r.0)
    );
    assert!(
        !accounts::checks(&l.fixture, &example, &l.outcomes)
            .iter()
            .all(|r| r.0)
    );
    dlms_learning_lab::accounts_template::apply_phase(&l.fixture, &mut example, &t.repair).unwrap();
    for c in &l.repair {
        dlms_learning_lab::accounts_teaching::execute_in_context(&l.fixture, &mut example, c)
            .unwrap();
    }
    assert!(
        accounts::checks(&l.fixture, &example, &l.outcomes)
            .iter()
            .all(|r| r.0)
    );
    assert_eq!(serde_json::to_value(&a).unwrap(), untouched);
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn pending_separate_work_resumes_and_reset_preserves_favorites_without_fabricating_answers() {
    let s = lab();
    let mut p = Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    p.favorites.insert(s.id.clone());
    let mut a = finished(&s);
    let d = a
        .workspace
        .as_mut()
        .unwrap()
        .accounts_template_drafts
        .as_mut()
        .unwrap();
    d.leaves.get_mut("/etc/skel/welcome.txt").unwrap().contents = "Unfinished\0draft\n".into();
    let example = a
        .workspace
        .as_mut()
        .unwrap()
        .accounts_learning
        .as_mut()
        .unwrap();
    example
        .template_drafts
        .as_mut()
        .unwrap()
        .leaves
        .get_mut("/etc/skel/.profile")
        .unwrap()
        .mode = "08".into();
    p.attempts.insert(s.id.clone(), a);
    let encoded = serde_json::to_vec(&p).unwrap();
    let mut restored = progress::decode(&encoded, std::slice::from_ref(&s)).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    assert!(!progress::current_solved(&restored.attempts[&s.id], &s));
    restored.attempts.insert(s.id.clone(), fresh(&s));
    assert_eq!(restored.favorites, p.favorites);
    assert!(!progress::current_solved(&restored.attempts[&s.id], &s));
}
