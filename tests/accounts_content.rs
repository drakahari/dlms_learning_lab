//! Authored account outcomes, alternatives, preservation and separate practice.
//! All mutation commands use the immutable goal-aware boundary, never host tools.
#[path = "../examples/support/automation_digest.rs"]
mod digest;
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
#[path = "support/one_off_registration.rs"]
mod one_off_registration;
use dlms_learning_lab::{
    accounts::{self, Requirement},
    accounts_teaching::{self, Learning},
    backup, catalog, course,
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching::View,
    shell, workspace,
};
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    sync::atomic::{AtomicBool, AtomicUsize},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Plans {
    version: u32,
    plans: Vec<Plan>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    id: String,
    alternatives: Vec<Vec<String>>,
    negatives: Vec<Wrong>,
    break_command: String,
    repair_command: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wrong {
    name: String,
    commands: Vec<String>,
}
fn labs() -> Vec<Scenario> {
    pack::load(include_str!("../scenarios/accounts-lifecycle-v1.json")).unwrap()
}
fn plans() -> Vec<Plan> {
    let p: Plans =
        serde_json::from_str(include_str!("fixtures/accounts-content-review.json")).unwrap();
    assert_eq!(p.version, 1);
    p.plans
}
fn catalog_with_labs() -> Vec<Scenario> {
    let normal = pack::bundled().unwrap();
    assert_eq!(one_off_registration::original_catalog(&normal).len(), 176);
    let mut all = normal
        .into_iter()
        .filter(|s| s.unit != "local-account-lifecycle")
        .collect::<Vec<_>>();
    all.extend(labs());
    all
}
fn simulation(state: &State) -> &accounts::State {
    let State::Accounts { simulation } = state else {
        panic!("account state expected")
    };
    simulation
}
fn goal(s: &Scenario) -> (&accounts::State, &[Requirement]) {
    let Goal::Accounts {
        initial,
        requirements,
    } = &s.goal
    else {
        panic!("account goal expected")
    };
    (initial, requirements)
}
fn run(s: &Scenario, state: &mut State, commands: &[String]) {
    for c in commands {
        engine::execute_for_goal(state, &s.goal, c)
            .unwrap_or_else(|e| panic!("{} / {c}: {e}", s.id));
    }
}
fn reference(s: &Scenario) -> State {
    let mut state = s.initial.clone();
    run(s, &mut state, &s.solution);
    state
}
fn attempt(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    accounts_teaching::prepare(&mut a, s.accounts_teaching.as_ref().unwrap(), true);
    a
}
fn learning(a: &mut Attempt) -> &mut Learning {
    a.workspace
        .as_mut()
        .unwrap()
        .accounts_learning
        .as_mut()
        .unwrap()
}

#[test]
fn four_distinct_lifecycle_tasks_are_reachable_in_the_normal_unit_and_prerequisite_path() {
    let authored = labs();
    assert_eq!(authored.len(), 4);
    let all = pack::bundled().unwrap();
    let units = course::bundled(&all).unwrap();
    let unit = units
        .iter()
        .find(|u| u.id == "local-account-lifecycle")
        .unwrap();
    assert_eq!(unit.track, "LPIC-1");
    assert_eq!(unit.prerequisites, ["linux-access-policy"]);
    assert_eq!(
        unit.labs,
        authored.iter().map(|s| s.id.clone()).collect::<Vec<_>>()
    );
    for (i, s) in authored.iter().enumerate() {
        assert!(all.iter().any(|v| v.id == s.id));
        assert_eq!(s.unit, unit.id);
        assert_eq!(s.track, "LPIC-1");
        assert_eq!(
            s.prerequisites,
            if i == 0 {
                vec![]
            } else {
                vec![authored[i - 1].id.clone()]
            }
        );
        assert_eq!(goal(s).0, simulation(&s.initial));
        assert!(accounts::valid_requirements(goal(s).0, goal(s).1));
        assert!(!engine::solved(&s.initial, &s.goal));
        assert!(engine::solved(&reference(s), &s.goal));
        assert_eq!(
            goal(s)
                .1
                .iter()
                .filter(|r| matches!(r, Requirement::Configuration { .. }))
                .count(),
            1
        );
        let lesson = s.accounts_teaching.as_ref().unwrap();
        lesson.validate().unwrap();
        assert!(lesson.separate_from(simulation(&s.initial)));
        assert!(
            lesson
                .example
                .iter()
                .zip(&lesson.example_commands)
                .all(|(c, a)| c == &a.command && !a.explanation.is_empty())
        );
        assert!(
            s.sources
                .iter()
                .any(|u| u.contains("lpi.org/our-certifications/exam-101-102-objectives"))
        );
        assert!(
            s.simplifications.contains("metadata only")
                && s.simplifications.contains("authentication")
        );
    }
    // Distinct operations, not an address/identity variant presented as a new job.
    assert!(
        authored[0]
            .solution
            .iter()
            .any(|c| c.starts_with("useradd ") && c.contains(" -k "))
    );
    assert!(
        authored[1]
            .solution
            .iter()
            .any(|c| c.starts_with("usermod -g "))
    );
    assert!(
        authored[2]
            .solution
            .iter()
            .any(|c| c.starts_with("userdel -r "))
    );
    assert!(
        authored[3]
            .solution
            .iter()
            .any(|c| c.starts_with("groupmod -n "))
    );
    assert!(
        authored[3]
            .solution
            .iter()
            .any(|c| c.starts_with("usermod -e "))
    );
}

#[test]
fn equivalent_flags_order_numeric_ids_and_readonly_investigation_grade_final_state() {
    let authored = labs();
    let review = plans();
    assert_eq!(review.len(), authored.len());
    for (s, p) in authored.iter().zip(review) {
        assert_eq!(s.id, p.id);
        let wanted = reference(s);
        for commands in p.alternatives {
            assert_ne!(commands, s.solution);
            let mut actual = s.initial.clone();
            // Investigation is optional, never a recorded prerequisite for credit.
            engine::execute_for_goal(&mut actual, &s.goal, "getent passwd").unwrap();
            engine::execute_for_goal(&mut actual, &s.goal, "ls -ln /srv/shared").unwrap();
            run(s, &mut actual, &commands);
            assert_eq!(actual, wanted, "{} alternate", s.id);
            assert!(engine::solved(&actual, &s.goal));
        }
        let mut actual = wanted.clone();
        let user = simulation(&actual)
            .configuration()
            .users
            .iter()
            .find(|(_, u)| u.uid >= 1000)
            .unwrap();
        let name = user.0.clone();
        let uid = user.1.uid;
        let group = simulation(&actual)
            .configuration()
            .groups
            .iter()
            .find(|(_, g)| g.gid >= 1000)
            .unwrap();
        let group_name = group.0.clone();
        let gid = group.1.gid;
        for (left, right) in [
            (
                format!("getent passwd {name}"),
                format!("getent passwd {uid}"),
            ),
            (
                format!("getent group {group_name}"),
                format!("getent group {gid}"),
            ),
            (format!("id -Gn {name}"), format!("id -nG {name}")),
            ("getent passwd".into(), "cat /etc/passwd".into()),
            ("getent group".into(), "cat /etc/group".into()),
            ("getent shadow".into(), "cat /etc/shadow".into()),
        ] {
            assert_eq!(
                engine::execute_for_goal(&mut actual, &s.goal, &left).unwrap(),
                engine::execute_for_goal(&mut actual, &s.goal, &right).unwrap()
            );
            assert_eq!(actual, wanted, "{} query changed state", s.id);
        }
    }
}

#[test]
fn wrong_partial_lifecycle_and_collateral_changes_fail_while_partial_progress_is_visible() {
    let authored = labs();
    let review = plans();
    for (s, p) in authored.iter().zip(review) {
        for wrong in p.negatives {
            let mut state = s.initial.clone();
            run(s, &mut state, &wrong.commands);
            assert!(simulation(&state).valid_against(simulation(&s.initial)));
            assert!(
                !engine::solved(&state, &s.goal),
                "{} / {}",
                s.id,
                wrong.name
            );
            assert!(engine::checks(&state, &s.goal).iter().any(|c| !c.0));
        }
        let mut partial = s.initial.clone();
        engine::execute_for_goal(&mut partial, &s.goal, &s.solution[0]).unwrap();
        let checks = engine::checks(&partial, &s.goal);
        assert!(
            checks.iter().any(|c| c.0) && checks.iter().any(|c| !c.0),
            "{} needs useful partial results",
            s.id
        );
        // Omit each authored repair. Dependent commands may correctly fail, but
        // neither a command failure nor a later query may earn completion.
        for omitted in 0..s.solution.len() {
            let mut state = s.initial.clone();
            for (i, command) in s.solution.iter().enumerate() {
                if i != omitted {
                    let before = state.clone();
                    if engine::execute_for_goal(&mut state, &s.goal, command).is_err() {
                        assert_eq!(state, before, "{} failed command mutated state", s.id);
                    }
                }
            }
            assert!(
                !engine::solved(&state, &s.goal),
                "{} missing repair {omitted}",
                s.id
            );
        }
    }
}

#[test]
fn successful_current_state_breaks_and_repairs_without_relying_on_checked_history() {
    for (s, p) in labs().iter().zip(plans()) {
        let mut a = attempt(s);
        a.state = reference(s);
        a.workspace.as_mut().unwrap().checked = true;
        assert!(progress::current_solved(&a, s));
        assert_eq!(a.step, 0);
        engine::execute_for_goal(&mut a.state, &s.goal, &p.break_command).unwrap();
        assert!(!progress::current_solved(&a, s));
        assert!(a.workspace.as_ref().unwrap().checked);
        engine::execute_for_goal(&mut a.state, &s.goal, &p.repair_command).unwrap();
        assert!(progress::current_solved(&a, s));
    }
}

fn break_requirement(s: &Scenario, solved: &State, requirement: &Requirement) -> State {
    // Controlled typed fixture mutation exercises the grader even for irreversible
    // learner operations. It is not a learner command/import bypass.
    let mut value = serde_json::to_value(solved).unwrap();
    let c = &mut value["simulation"]["configuration"];
    let initial = simulation(&s.initial).configuration();
    match requirement {
        Requirement::User { name, .. } => {
            c["users"][name]["expires"] =
                serde_json::json!(if c["users"][name]["expires"].is_null() {
                    Some(30_000)
                } else {
                    None
                });
        }
        Requirement::Group { name, .. } => {
            let record = c["groups"].as_object_mut().unwrap().remove(name).unwrap();
            c["groups"][format!("{name}_changed")] = record;
        }
        Requirement::UserAbsent { name, .. } => {
            let mut u = initial.users[name].clone();
            let groups = c["groups"]
                .as_object()
                .unwrap()
                .values()
                .map(|g| g["gid"].as_u64().unwrap() as u32)
                .collect::<BTreeSet<_>>();
            if !groups.contains(&u.primary_gid) {
                u.primary_gid = *groups.iter().find(|g| **g >= 1000).unwrap();
            }
            u.supplementary.retain(|g| groups.contains(g));
            c["users"][name] = serde_json::to_value(u).unwrap();
        }
        Requirement::GroupAbsent { name, .. } => {
            c["groups"][name] = serde_json::to_value(&initial.groups[name]).unwrap()
        }
        Requirement::Entry { path, .. } => {
            let mode = c["entries"][path]["mode"].as_u64().unwrap();
            c["entries"][path]["mode"] = serde_json::json!(mode ^ 0o40);
        }
        Requirement::AbsentPath { path } => {
            let mut pending = path.as_str();
            let mut parents = vec![];
            while !c["entries"].as_object().unwrap().contains_key(pending) {
                parents.push(pending.to_owned());
                pending = pending.rsplit_once('/').unwrap().0;
                if pending.is_empty() {
                    pending = "/";
                }
            }
            for p in parents.into_iter().rev() {
                c["entries"][&p] = serde_json::to_value(&initial.entries[&p]).unwrap();
            }
        }
        Requirement::Configuration { .. } => {
            c["entries"]["/srv/shared/outside.txt"]["data"]["contents"] =
                "Unrequested change\n".into();
        }
        Requirement::ExpiryRange { name, .. } => {
            c["users"][name]["expires"] = serde_json::Value::Null
        }
    }
    serde_json::from_value(value).unwrap()
}

#[test]
fn every_individual_typed_outcome_can_invalidate_a_previously_successful_attempt() {
    for s in labs() {
        let good = reference(&s);
        let (_, requirements) = goal(&s);
        for (i, r) in requirements.iter().enumerate() {
            let broken = break_requirement(&s, &good, r);
            pack::validate_state(&broken).unwrap();
            assert!(
                simulation(&broken).valid_against(simulation(&s.initial)),
                "{} goal{i} broke context instead of outcome",
                s.id
            );
            let checks = engine::checks(&broken, &s.goal);
            assert_eq!(checks.len(), requirements.len());
            assert!(!checks[i].0, "{} goal{i} silently remained correct", s.id);
            let mut a = attempt(&s);
            a.state = broken;
            a.workspace.as_mut().unwrap().checked = true;
            assert!(!progress::current_solved(&a, &s));
        }
    }
}

#[test]
fn protected_data_unsupported_commands_and_unsafe_uid_reuse_fail_atomically() {
    for s in labs() {
        let mut state = s.initial.clone();
        for command in [
            "usermod -G audit root",
            "groupdel root",
            "chown pax:staff /etc/skel/.profile",
            "passwd pax",
            "usermod -L pax",
            "id pax | cat",
            "useradd incomplete",
            "userdel -r ../../root",
        ] {
            let before = state.clone();
            assert!(
                engine::execute_for_goal(&mut state, &s.goal, command).is_err(),
                "{} / {command}",
                s.id
            );
            assert_eq!(state, before);
        }
    }
    let authored = labs();
    let s = &authored[2];
    let mut state = reference(s);
    for command in [
        "useradd -N -u 1401 -g staff -d /home/replacement -s /bin/bash -M replacement",
        "useradd -N -u 1402 -g staff -d /home/reed -s /bin/bash -M reed",
    ] {
        let before = state.clone();
        assert!(engine::execute_for_goal(&mut state, &s.goal, command).is_err());
        assert_eq!(state, before);
        assert!(engine::solved(&state, &s.goal));
    }
}

#[test]
fn examples_teach_execute_try_and_repair_without_mutating_or_completing_assignments() {
    for s in labs() {
        let lesson = s.accounts_teaching.as_ref().unwrap();
        let mut a = attempt(&s);
        let assignment = a.state.clone();
        let assignment_command = "unfinished assignment command".to_owned();
        a.workspace.as_mut().unwrap().command = assignment_command.clone();
        for (index, command) in lesson.example.iter().enumerate() {
            let l = learning(&mut a);
            l.example_command = index;
            l.command = "previous ungraded input".into();
            l.checked = true;
            let mut before = serde_json::to_value(&*l).unwrap();
            l.fill_example(lesson).unwrap();
            before["command"] = command.clone().into();
            assert_eq!(
                serde_json::to_value(&*l).unwrap(),
                before,
                "Fill executed or altered example data"
            );
            let output = l.execute(lesson).unwrap();
            assert_eq!(output.status, 0);
            assert!(!l.checked);
            assert_eq!(a.state, assignment);
            assert_eq!(a.workspace.as_ref().unwrap().command, assignment_command);
            assert!(!progress::current_solved(&a, &s));
        }
        assert!(learning(&mut a).checks(lesson).iter().all(|c| c.0));
        for c in &lesson.try_actions {
            learning(&mut a).command = c.clone();
            assert_eq!(learning(&mut a).execute(lesson).unwrap().status, 0);
        }
        let current = &learning(&mut a).simulation;
        assert!(
            accounts::checks(&lesson.fixture, current, &lesson.try_outcomes)
                .iter()
                .all(|c| c.0)
        );
        assert!(learning(&mut a).checks(lesson).iter().any(|c| !c.0));
        for c in &lesson.repair {
            learning(&mut a).command = c.clone();
            assert_eq!(learning(&mut a).execute(lesson).unwrap().status, 0);
        }
        assert!(learning(&mut a).checks(lesson).iter().all(|c| c.0));
        assert_eq!(a.state, assignment);
        assert!(!progress::current_solved(&a, &s));
        // Challenge still grades the same substantive assignment. Practice flags
        // are never a credential; actual visibility needs separate native tests.
        a.challenge = true;
        learning(&mut a).checked = true;
        assert!(!progress::current_solved(&a, &s));
        learning(&mut a).command = "usermod -G 'unfinished".into();
        let before = learning(&mut a).simulation.clone();
        assert!(learning(&mut a).execute(lesson).is_err());
        assert_eq!(learning(&mut a).simulation, before);
        learning(&mut a).command = "getent passwd".into();
        assert_eq!(learning(&mut a).execute(lesson).unwrap().status, 0);
    }
}

#[test]
fn imported_policy_identity_skeleton_and_foreign_example_attacks_retain_profile_bytes() {
    let all = catalog_with_labs();
    for s in labs() {
        let mut p = Progress {
            schema_version: 1,
            ..Default::default()
        };
        p.attempts.insert(s.id.clone(), attempt(&s));
        let good = serde_json::to_vec_pretty(&p).unwrap();
        progress::decode(&good, &all).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("progress.json");
        std::fs::write(&target, &good).unwrap();
        let export = dir.path().join("portable.json");
        backup::export(&export, &p, &all).unwrap();
        let portable: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&export).unwrap()).unwrap();
        for attack in 0..5 {
            let mut value: serde_json::Value = serde_json::from_slice(&good).unwrap();
            let a = &mut value["attempts"][&s.id];
            match attack {
                0 => a["state"]["simulation"]["policy"]["fixture"] = "forged-assignment".into(),
                1 => a["state"]["simulation"]["configuration"]["users"]["pax"]["uid"] = 9999.into(),
                2 => {
                    a["state"]["simulation"]["configuration"]["entries"]["/etc/skel/.profile"]["data"]
                        ["contents"] = "Forged template\n".into()
                }
                3 => {
                    a["workspace"]["accounts_learning"]["simulation"]["policy"]["fixture"] =
                        "foreign-example".into()
                }
                _ => {
                    a["workspace"]["accounts_learning"]["simulation"] =
                        a["state"]["simulation"].clone()
                }
            }
            assert!(
                progress::decode(&serde_json::to_vec(&value).unwrap(), &all).is_err(),
                "{} attack{attack}",
                s.id
            );
            let mut envelope = portable.clone();
            envelope["progress"] = value;
            let forged = dir.path().join(format!("forged-{attack}.json"));
            let forged_bytes = serde_json::to_vec(&envelope).unwrap();
            std::fs::write(&forged, &forged_bytes).unwrap();
            assert!(
                backup::inspect(&forged, &all, &AtomicBool::new(false), &AtomicUsize::new(0))
                    .is_err()
            );
            assert_eq!(std::fs::read(&forged).unwrap(), forged_bytes);
            assert_eq!(std::fs::read(&target).unwrap(), good);
        }
    }
}

#[test]
fn prior_ninety_and_four_acl_attempts_new_drafts_and_shell_history_survive_export_restore_reset() {
    let authored = labs();
    let ids = authored
        .iter()
        .map(|s| s.id.as_str())
        .collect::<BTreeSet<_>>();
    let all = catalog_with_labs();
    let prior = all
        .iter()
        .filter(|s| !ids.contains(s.id.as_str()))
        .collect::<Vec<_>>();
    assert!(
        prior.len() >= 94,
        "Original90 plus the four accepted ACL activities must remain"
    );
    for id in [
        "ccna-acl-source-policy",
        "ccna-acl-service-order",
        "ccna-acl-direction-repair",
        "ccna-acl-policy-handover",
    ] {
        assert!(prior.iter().any(|s| s.id == id));
    }
    let mut p = Progress {
        schema_version: 1,
        clock: 172,
        ..Default::default()
    };
    for s in &prior {
        let mut a = Attempt::new(s, 1);
        if let Goal::Script { cases } = &s.goal {
            run(s, &mut a.state, &s.solution);
            let State::Script { source, run } = &mut a.state else {
                unreachable!()
            };
            *run = Some(shell::Record {
                engine_revision: shell::EVALUATOR_REVISION,
                source: source.clone(),
                results: cases.iter().map(|c| shell::run(source, c)).collect(),
            });
        }
        workspace::prepare(&mut a);
        if matches!(a.state, State::Cron { .. }) {
            dlms_learning_lab::cron_workspace::budget::prepare(&mut a, s).unwrap();
        }
        if let Some(lesson) = &s.file_backup_teaching {
            // Preserve this real new-family positive in the cross-family matrix.
            dlms_learning_lab::file_backup_teaching::prepare(&mut a, lesson, true).unwrap();
        }
        if matches!(a.state, State::OneOff { .. }) {
            use dlms_learning_lab::one_off_workspace as one;
            one::budget::prepare(&mut a, s).unwrap();
            let path = if let Goal::OneOff { definition } = &s.goal {
                definition.editable[0].clone()
            } else {
                unreachable!()
            };
            one::budget::apply_attempt(
                &mut a,
                s,
                &one::Action::Edit {
                    path,
                    text: format!("# Retained OneOff assignment draft for {}\n", s.id),
                },
                false,
            )
            .unwrap();
            let path = s.one_off_teaching.as_ref().unwrap().definition.editable[0].clone();
            one::budget::apply_attempt(
                &mut a,
                s,
                &one::Action::Edit {
                    path,
                    text: "# Retained separate OneOff example draft\n".into(),
                },
                true,
            )
            .unwrap();
            assert!(a.workspace.as_ref().unwrap().command.is_empty());
            assert!(one::context(&a, s).is_ok());
            assert!(!progress::current_solved(&a, s));
        } else if matches!(a.state, State::PhysicalLink { .. }) {
            use dlms_learning_lab::physical_link_workspace as physical;
            physical::prepare_attempt(&mut a, s, true).unwrap();
            let mut draft = a
                .workspace
                .as_ref()
                .unwrap()
                .physical_link_draft
                .as_ref()
                .unwrap()
                .clone();
            let port = draft.configuration.ports.values_mut().next().unwrap();
            port.admin_up = !port.admin_up;
            physical::set_draft(&mut a, s, physical::Scope::Assignment, &draft).unwrap();
            assert!(!progress::current_solved(&a, s));
            assert!(
                a.workspace.as_ref().unwrap().command.is_empty(),
                "Native physical config never borrows a terminal draft"
            );
        } else {
            a.workspace.as_mut().unwrap().command = format!("Retained draft for {}", s.id);
        }
        a.transcript
            .push("Retained synthetic investigation transcript".into());
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id.clone());
    }
    let historic: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/shell-v6-authored-records.json")).unwrap();
    let sample = &historic[0];
    let old: shell::Record = serde_json::from_value(sample["record"].clone()).unwrap();
    assert_eq!(old.engine_revision, 6);
    let legacy_id = sample["id"].as_str().unwrap();
    p.attempts
        .get_mut(legacy_id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .script_history = Some(old);
    let prior_bytes = serde_json::to_value(&p.attempts).unwrap();
    for s in &authored {
        let mut a = attempt(s);
        a.state = reference(s);
        a.workspace.as_mut().unwrap().command = "Unsubmitted assignment draft ü '".into();
        let l = learning(&mut a);
        l.view = View::Example;
        l.command = "Unsubmitted example draft ü '".into();
        l.transcript.push("Independent example transcript".into());
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id.clone());
    }
    let dir = tempfile::tempdir().unwrap();
    let profile = dir.path().join("progress.json");
    progress::save(&profile, &p).unwrap();
    let exact = std::fs::read(&profile).unwrap();
    let loaded = progress::load(&profile, &all).unwrap();
    assert_eq!(std::fs::read(&profile).unwrap(), exact);
    assert_eq!(
        serde_json::to_value(&loaded.attempts).unwrap(),
        serde_json::to_value(&p.attempts).unwrap()
    );
    let export = dir.path().join("portable.json");
    backup::export(&export, &loaded, &all).unwrap();
    let portable_bytes = std::fs::read(&export).unwrap();
    let candidate =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    assert_eq!(std::fs::read(&export).unwrap(), portable_bytes);
    let destination = dir.path().join("restored.json");
    progress::save(
        &destination,
        &Progress {
            schema_version: 1,
            ..Default::default()
        },
    )
    .unwrap();
    let before_restore = std::fs::read(&destination).unwrap();
    let restored = backup::restore(&destination, &candidate, &all).unwrap();
    assert_eq!(
        std::fs::read(restored.safety_copy.unwrap()).unwrap(),
        before_restore
    );
    let mut again = progress::load(&destination, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&again.attempts).unwrap(),
        serde_json::to_value(&p.attempts).unwrap()
    );
    assert_eq!(again.favorites, p.favorites);
    for s in &authored {
        assert!(progress::current_solved(&again.attempts[&s.id], s));
    }
    let before_reset = serde_json::to_value(&again.attempts).unwrap();
    let selected = &authored[1];
    again
        .attempts
        .insert(selected.id.clone(), attempt(selected));
    progress::save(&destination, &again).unwrap();
    let again = progress::load(&destination, &all).unwrap();
    for (id, a) in &again.attempts {
        if id != &selected.id {
            assert_eq!(serde_json::to_value(a).unwrap(), before_reset[id]);
        }
    }
    assert!(!progress::current_solved(
        &again.attempts[&selected.id],
        selected
    ));
    for (id, want) in prior_bytes.as_object().unwrap() {
        assert_eq!(serde_json::to_value(&again.attempts[id]).unwrap(), *want);
    }
}

#[test]
fn library_filters_and_advisory_unit_navigation_keep_the_complete_assignments_accessible() {
    let authored = labs();
    let all = pack::bundled().unwrap();
    let units = course::bundled(&all).unwrap();
    let unit = units
        .iter()
        .find(|u| u.id == "local-account-lifecycle")
        .unwrap();
    let mut f = catalog::Filter {
        track: "LPIC-1".into(),
        unit: unit.id.clone(),
        ..Default::default()
    };
    assert_eq!(
        all.iter()
            .filter(|s| catalog::matches(s, &f, false, false, false))
            .count(),
        4
    );
    f.search = "retire".into();
    assert!(catalog::matches(&authored[2], &f, false, false, false));
    f.search.clear();
    f.favorites = true;
    f.recent = true;
    f.status = "In progress".into();
    assert!(catalog::matches(&authored[0], &f, true, false, true));
    assert!(!catalog::matches(&authored[0], &f, true, true, true));
    assert!(!catalog::matches(&authored[0], &f, false, false, true));
    let p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let next = course::next(unit, &authored[0].id, &all, &p).unwrap();
    assert_eq!(all[next].id, authored[1].id);
    // No forced setup/mastery gate exists: the final independent task can start.
    let a = attempt(&authored[3]);
    assert_eq!(a.state, authored[3].initial);
    assert!(!progress::current_solved(&a, &authored[3]));
}
