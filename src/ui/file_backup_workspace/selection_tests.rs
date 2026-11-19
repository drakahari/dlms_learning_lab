//! Five actual authored backup tasks against the current bounded shared index.
use super::*;
fn prepared(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    model::prepare(&mut a);
    dlms_learning_lab::file_backup_teaching::prepare(
        &mut a,
        s.file_backup_teaching.as_ref().unwrap(),
        true,
    )
    .unwrap();
    for c in &s.solution {
        work::perform(&mut a, s, work::Scope::Assignment, c).unwrap();
    }
    assert!(dlms_learning_lab::progress::current_solved(&a, s));
    a
}
fn verify(m: &mut CheckMemo, a: &Attempt, s: &Scenario) {
    m.prepare(a, s, false).unwrap();
    for _ in 0..200 {
        if m.job.is_none() {
            break;
        }
        m.advance(a, s);
    }
    assert!(m.job.is_none());
    assert_eq!(
        m.certificates.status(a, s),
        Some(completion_index::Status::Complete)
    );
}
#[test]
fn backup_selection_five_authored_backup_proofs_survive_four_selected_snapshots_and_idle_without_json_or_new_work()
 {
    let all = dlms_learning_lab::pack::ccna_pilot().unwrap();
    let sources = all
        .iter()
        .filter(|s| s.file_backup_teaching.is_some())
        .collect::<Vec<_>>();
    assert_eq!(sources.len(), 5);
    assert_eq!(sources[4].id, "lpic2-backup-directory-selection");
    let mut memo = CheckMemo::default();
    let attempts = sources.iter().map(|s| prepared(s)).collect::<Vec<_>>();
    for (s, a) in sources.iter().zip(&attempts) {
        verify(&mut memo, a, s);
    }
    assert!(memo.entries.len() <= 4);
    let before = memo.certificates.observed();
    assert_eq!(before.0, 5);
    let counters = memo.counters;
    let json = dlms_learning_lab::native_observations::json_admission_calls();
    for _ in 0..80 {
        for (s, a) in sources.iter().zip(&attempts) {
            assert_eq!(
                memo.certificates.status(a, s),
                Some(completion_index::Status::Complete)
            );
        }
        memo.prepare(attempts.last().unwrap(), sources.last().unwrap(), false)
            .unwrap();
    }
    let after = memo.certificates.observed();
    assert_eq!(
        (
            after.0,
            after.1,
            after.2.contexts,
            after.2.key_copies,
            after.2.certificates
        ),
        (
            before.0,
            before.1,
            before.2.contexts,
            before.2.key_copies,
            before.2.certificates
        )
    );
    assert_eq!(memo.counters.admissions, counters.admissions);
    assert_eq!(memo.counters.copies, counters.copies);
    assert_eq!(memo.counters.ticks, counters.ticks);
    assert_eq!(
        dlms_learning_lab::native_observations::json_admission_calls(),
        json
    );
    assert!(memo.entries.iter().map(|e| e.charge).sum::<usize>() <= TOTAL_BYTES);
}
#[test]
fn backup_selection_selected_damage_foreign_source_and_reset_cannot_reuse_fifth_success_or_destroy_siblings()
 {
    let all = dlms_learning_lab::pack::ccna_pilot().unwrap();
    let sources = all
        .iter()
        .filter(|s| s.file_backup_teaching.is_some())
        .collect::<Vec<_>>();
    assert_eq!(sources.len(), 5);
    let mut memo = CheckMemo::default();
    let mut attempts = sources.iter().map(|s| prepared(s)).collect::<Vec<_>>();
    for (s, a) in sources.iter().zip(&attempts) {
        verify(&mut memo, a, s);
    }
    let selected = sources[4];
    let original = attempts[4].clone();
    work::perform(
        &mut attempts[4],
        selected,
        work::Scope::Assignment,
        "chmod 777 /recovery/etc/orchard/payload.txt",
    )
    .unwrap();
    assert_eq!(
        memo.certificates.status(&attempts[4], selected),
        Some(completion_index::Status::Pending)
    );
    memo.prepare(&attempts[4], selected, false).unwrap();
    for _ in 0..200 {
        if memo.job.is_none() {
            break;
        }
        memo.advance(&attempts[4], selected);
    }
    assert!(memo.job.is_none());
    assert_eq!(
        memo.certificates.status(&attempts[4], selected),
        Some(completion_index::Status::Incomplete)
    );
    let before = memo.counters;
    let mut foreign = selected.clone();
    foreign.goal = all[0].goal.clone();
    assert!(memo.prepare(&original, &foreign, false).is_err());
    assert_eq!(memo.counters.copies, before.copies);
    assert_eq!(memo.counters.admissions, before.admissions);
    assert_eq!(
        memo.certificates.status(&original, &foreign),
        Some(completion_index::Status::Pending)
    );
    for (s, a) in sources[..4].iter().zip(&attempts[..4]) {
        assert_eq!(
            memo.certificates.status(a, s),
            Some(completion_index::Status::Complete)
        );
    }
    verify(&mut memo, &original, selected);
    memo.certificates.forget(&selected.id);
    assert_eq!(
        memo.certificates.status(&original, selected),
        Some(completion_index::Status::Pending)
    );
    for (s, a) in sources[..4].iter().zip(&attempts[..4]) {
        assert_eq!(
            memo.certificates.status(a, s),
            Some(completion_index::Status::Complete)
        );
    }
}
