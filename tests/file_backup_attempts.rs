//! Real serialized attempts with isolated practice and no imported credit invention.
#[path = "support/file_backup_authored.rs"]
mod fixture;
use dlms_learning_lab::{
    engine::State,
    file_backup_teaching as teach,
    file_backup_workspace::{self as work, Scope},
    progress::{self, Attempt, Progress},
    script_teaching::View,
    workspace,
};
use fixture::*;
fn exact<T: serde::Serialize + ?Sized>(v: &T) -> serde_json::Value {
    serde_json::to_value(v).unwrap()
}
fn fresh(s: &dlms_learning_lab::pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 7);
    workspace::prepare(&mut a);
    teach::prepare(&mut a, s.file_backup_teaching.as_ref().unwrap(), true).unwrap();
    a
}
fn finish(a: &mut Attempt, s: &dlms_learning_lab::pack::Scenario) {
    for c in &s.solution {
        work::perform(a, s, Scope::Assignment, c).unwrap();
    }
}
#[test]
fn practice_success_cannot_complete_or_overwrite_any_assignment_draft() {
    for row in review()["labs"].as_array().unwrap() {
        let s = lab(row["id"].as_str().unwrap());
        let mut a = fresh(s);
        a.workspace.as_mut().unwrap().command = "tar -xf /backups/unfinished.tar".into();
        let before = exact(&a.state);
        let lesson = s.file_backup_teaching.as_ref().unwrap();
        let learning = a
            .workspace
            .as_mut()
            .unwrap()
            .file_backup_learning
            .as_mut()
            .unwrap();
        learning.view = View::Example;
        for c in &lesson.example_commands {
            learning.command = c.command.clone();
            learning.execute(lesson).unwrap();
        }
        learning.checked = true;
        assert!(
            lesson
                .definition
                .checks(&learning.state)
                .iter()
                .all(|r| r.0)
        );
        assert_eq!(exact(&a.state), before);
        assert_eq!(
            a.workspace.as_ref().unwrap().command,
            "tar -xf /backups/unfinished.tar"
        );
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn both_drafts_outputs_and_goal_state_survive_safe_save_and_fresh_file_load() {
    for row in review()["labs"].as_array().unwrap() {
        let s = lab(row["id"].as_str().unwrap());
        let mut a = fresh(s);
        finish(&mut a, s);
        assert_eq!(a.step, 0);
        assert!(progress::current_solved(&a, s));
        let lesson = s.file_backup_teaching.as_ref().unwrap();
        let learning = a
            .workspace
            .as_mut()
            .unwrap()
            .file_backup_learning
            .as_mut()
            .unwrap();
        learning.view = View::Example;
        for c in lesson.example_commands.iter().take(2) {
            learning.command = c.command.clone();
            learning.execute(lesson).unwrap();
        }
        learning.command = "stat -c '%a %U %G' unfinished-example.txt".into();
        a.workspace.as_mut().unwrap().command = "tar -tf /backups/unfinished-assignment.tar".into();
        let mut p = Progress {
            schema_version: 1,
            clock: 20,
            ..Default::default()
        };
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
        let wanted = exact(&p);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress.json");
        progress::save(&path, &p).unwrap();
        drop(p);
        let loaded = progress::load(&path, catalog()).unwrap();
        assert_eq!(exact(&loaded), wanted);
        assert!(progress::current_solved(&loaded.attempts[&s.id], s));
        let loaded_a = &loaded.attempts[&s.id];
        assert_eq!(
            loaded_a
                .workspace
                .as_ref()
                .unwrap()
                .file_backup_learning
                .as_ref()
                .unwrap()
                .context,
            lesson.definition
        );
    }
}
#[test]
fn selected_reset_retains_other_labs_favorites_and_example_drafts() {
    let rows = review();
    let ids = rows["labs"].as_array().unwrap();
    let mut p = Progress {
        schema_version: 1,
        clock: 30,
        ..Default::default()
    };
    for row in ids {
        let s = lab(row["id"].as_str().unwrap());
        let mut a = fresh(s);
        finish(&mut a, s);
        a.workspace
            .as_mut()
            .unwrap()
            .file_backup_learning
            .as_mut()
            .unwrap()
            .command = format!("saved separate draft for {}", s.id);
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id.clone());
    }
    let selected = ids[1]["id"].as_str().unwrap();
    let before = exact(&p);
    let s = lab(selected);
    p.attempts.insert(selected.into(), fresh(s));
    assert!(!progress::current_solved(&p.attempts[selected], s));
    let after = exact(&p);
    for row in ids {
        let id = row["id"].as_str().unwrap();
        if id != selected {
            assert_eq!(after["attempts"][id], before["attempts"][id]);
        }
    }
    assert_eq!(after["favorites"], before["favorites"]);
    assert_eq!(after["clock"], before["clock"]);
    assert!(
        p.attempts[selected]
            .workspace
            .as_ref()
            .unwrap()
            .file_backup_learning
            .as_ref()
            .unwrap()
            .command
            .is_empty()
    );
}
#[test]
fn wrong_versions_context_and_foreign_metadata_fail_without_replacing_work() {
    let s = lab("lpic2-backup-selective-recovery");
    let a = fresh(s);
    let before = exact(&a);
    let mut wrong = s.clone();
    wrong.version += 1;
    assert!(!progress::current_solved(&a, &wrong));
    let mut different = a.clone();
    different.workspace.as_mut().unwrap().service_port = Some("unrelated service data".into());
    let saved = exact(&different);
    assert!(
        teach::prepare(
            &mut different,
            s.file_backup_teaching.as_ref().unwrap(),
            false
        )
        .is_err()
    );
    assert_eq!(exact(&different), saved);
    assert!(!progress::current_solved(&different, s));
    let nfs = catalog()
        .iter()
        .find(|s| matches!(s.initial, State::Nfs { .. }))
        .unwrap();
    let mut other = Attempt::new(nfs, 7);
    workspace::prepare(&mut other);
    other.workspace.as_mut().unwrap().file_backup_learning =
        a.workspace.as_ref().unwrap().file_backup_learning.clone();
    assert!(!progress::current_solved(&other, nfs));
    let p = Progress {
        schema_version: 1,
        clock: 20,
        attempts: std::collections::BTreeMap::from([(nfs.id.clone(), other)]),
        ..Default::default()
    };
    assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), catalog()).is_err());
    assert_eq!(exact(&a), before);
}
#[test]
fn practice_setup_capacity_refusal_is_atomic_before_installing_any_field() {
    let s = lab("lpic2-backup-selective-recovery");
    let mut a = Attempt::new(s, 7);
    workspace::prepare(&mut a);
    a.transcript = vec![String::new(); 120];
    let cap = dlms_learning_lab::file_backup_workspace::MAX_ATTEMPT;
    let base = dlms_learning_lab::file_backup_workspace::size(&a, cap).unwrap();
    let fill = cap - base - 64;
    for (i, text) in a.transcript.iter_mut().enumerate() {
        *text = "x".repeat(fill / 120 + usize::from(i < fill % 120));
    }
    assert!(dlms_learning_lab::file_backup_workspace::within(&a, cap));
    let before = exact(&a);
    assert!(teach::prepare(&mut a, s.file_backup_teaching.as_ref().unwrap(), true).is_err());
    assert_eq!(exact(&a), before);
    assert!(a.workspace.as_ref().unwrap().file_backup_learning.is_none());
}
#[test]
fn existing154_attempts_remain_exact_and_no_new_context_or_mastery_is_invented() {
    let all = catalog();
    let mut p = Progress {
        schema_version: 1,
        clock: 40,
        ..Default::default()
    };
    for s in &all[..154] {
        let mut a = Attempt::new(s, 7);
        if let State::Script { source, .. } = &mut a.state {
            *source = "printf '%s\\n' 'unfinished old draft'".into();
        }
        if let State::Subnet { answers, .. } = &mut a.state {
            answers[0] = "192.168.10.64".into();
        }
        workspace::prepare(&mut a);
        p.attempts.insert(s.id.clone(), a);
    }
    p.favorites.insert("ccna-ipv4".into());
    let old = exact(&p);
    let loaded = progress::decode(&serde_json::to_vec(&p).unwrap(), all).unwrap();
    assert_eq!(exact(&loaded), old);
    for row in review()["labs"].as_array().unwrap() {
        assert!(!loaded.attempts.contains_key(row["id"].as_str().unwrap()));
    }
    assert_eq!(loaded.attempts.len(), 154);
}
#[test]
fn real_practice_transactions_keep_assignment_state_history_and_drafts_independent() {
    for row in review()["labs"].as_array().unwrap() {
        let s = lab(row["id"].as_str().unwrap());
        let mut a = fresh(s);
        a.workspace.as_mut().unwrap().command = "unfinished assignment command".into();
        let state = exact(&a.state);
        let lesson = s.file_backup_teaching.as_ref().unwrap();
        for c in &lesson.example_commands {
            work::perform(&mut a, s, Scope::Practice, &c.command).unwrap();
        }
        let l = a
            .workspace
            .as_ref()
            .unwrap()
            .file_backup_learning
            .as_ref()
            .unwrap();
        assert!(lesson.definition.checks(&l.state).iter().all(|r| r.0));
        assert_eq!(exact(&a.state), state);
        assert!(a.transcript.is_empty());
        assert_eq!(
            a.workspace.as_ref().unwrap().command,
            "unfinished assignment command"
        );
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn rejected_terminal_actions_keep_the_complete_attempt_and_success_invalidation_is_current() {
    let s = lab("lpic2-backup-selective-recovery");
    let mut a = fresh(s);
    finish(&mut a, s);
    assert!(progress::current_solved(&a, s));
    for command in [
        "tar -xf /backups/project-full.tar -C /personal",
        "cat '/unfinished quote",
        "sh -c host-command",
        "tar --to-command=sh -xf /backups/project-full.tar",
    ] {
        let before = exact(&a);
        assert!(work::perform(&mut a, s, Scope::Assignment, command).is_err());
        assert_eq!(exact(&a), before);
        assert!(progress::current_solved(&a, s));
    }
    work::perform(
        &mut a,
        s,
        Scope::Assignment,
        "chmod 777 /restore/project/handover.txt",
    )
    .unwrap();
    assert!(!progress::current_solved(&a, s));
    assert!(
        progress::current_file_backup_checks(&a, s)
            .iter()
            .any(|r| !r.0 && r.1.contains("permissions"))
    );
    finish(&mut a, s);
    assert!(progress::current_solved(&a, s));
}
#[test]
fn prospective_practice_growth_is_rejected_without_losing_any_scope_or_history() {
    let s = lab("lpic2-backup-selective-recovery");
    let mut a = fresh(s);
    a.transcript = vec![String::new(); 120];
    let cap = work::MAX_ATTEMPT;
    let base = work::size(&a, cap).unwrap();
    let fill = cap - base - 8;
    for (i, text) in a.transcript.iter_mut().enumerate() {
        *text = "x".repeat(fill / 120 + usize::from(i < fill % 120));
    }
    assert!(work::within(&a, cap));
    let before = exact(&a);
    assert!(
        work::perform(
            &mut a,
            s,
            Scope::Practice,
            "tar -tf /example-backups/policy.tar"
        )
        .is_err()
    );
    assert_eq!(exact(&a), before);
}
