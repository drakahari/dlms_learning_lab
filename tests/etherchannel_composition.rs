//! Composition-only controls; no fixture outcome is a saved credential.
use dlms_learning_lab::{
    engine::{self, State},
    etherchannel as core, etherchannel_teaching as teaching, etherchannel_workspace as work,
    file_backup_teaching, file_backup_workspace, pack,
    progress::{self, Attempt},
};
fn pair() -> (pack::Scenario, pack::Scenario) {
    let e = pack::bundled()
        .unwrap()
        .into_iter()
        .find(|s| s.id == "ccna-lacp-initiation")
        .unwrap();
    let mut backup: pack::Pack =
        serde_json::from_str(include_str!("../scenarios/file-backup-v1.json")).unwrap();
    let b = backup.scenarios.remove(0);
    (e, b)
}
fn positive(e: &pack::Scenario, b: &pack::Scenario) -> (Attempt, Attempt) {
    let mut a = Attempt::new(e, 1);
    teaching::prepare(&mut a, e, true).unwrap();
    for action in &e.solution {
        let core::Action::Configure { configuration } =
            core::Action::from_json(action.strip_prefix(work::PREFIX).unwrap().as_bytes()).unwrap();
        let State::EtherChannel { simulation } = &a.state else {
            unreachable!()
        };
        let draft = work::Draft::from_configuration(&configuration, simulation.topology()).unwrap();
        work::set_assignment_draft(&mut a, e, &draft).unwrap();
        work::apply_assignment(&mut a, e).unwrap();
    }
    let mut c = Attempt::new(b, 2);
    file_backup_teaching::prepare(&mut c, b.file_backup_teaching.as_ref().unwrap(), true).unwrap();
    for command in &b.solution {
        engine::execute_for_goal(&mut c.state, &b.goal, command).unwrap();
    }
    assert!(progress::current_solved(&a, e));
    assert!(progress::current_solved(&c, b));
    (a, c)
}
#[test]
fn etherchannel_and_file_backup_reciprocal_valid_metadata_refuse_without_copy_or_lost_work() {
    let (e, b) = pair();
    let (a, c) = positive(&e, &b);
    let mut mixed = a.clone();
    mixed.workspace.as_mut().unwrap().file_backup_learning =
        c.workspace.as_ref().unwrap().file_backup_learning.clone();
    let before = serde_json::to_value(&mixed).unwrap();
    assert!(mixed.workspace.as_ref().unwrap().valid());
    assert!(!progress::current_solved(&mixed, &e));
    assert!(work::validate_context(&mixed, &e).is_err());
    assert!(teaching::prepare(&mut mixed, &e, false).is_err());
    assert!(work::fill_practice(&mut mixed, &e).is_err());
    assert_eq!(serde_json::to_value(&mixed).unwrap(), before);
    for draft_only in [true, false] {
        let mut mixed = c.clone();
        if draft_only {
            mixed.workspace.as_mut().unwrap().etherchannel_draft =
                a.workspace.as_ref().unwrap().etherchannel_draft.clone();
        } else {
            mixed.workspace.as_mut().unwrap().etherchannel_learning =
                a.workspace.as_ref().unwrap().etherchannel_learning.clone();
        }
        let before = serde_json::to_value(&mixed).unwrap();
        assert!(mixed.workspace.as_ref().unwrap().valid());
        assert!(file_backup_workspace::context(&mixed, &b).is_err());
        assert!(!progress::current_solved(&mixed, &b));
        assert!(
            file_backup_teaching::prepare(
                &mut mixed,
                b.file_backup_teaching.as_ref().unwrap(),
                false
            )
            .is_err()
        );
        assert_eq!(serde_json::to_value(&mixed).unwrap(), before);
    }
    let mut wrong_e = e.clone();
    wrong_e.file_backup_teaching = b.file_backup_teaching.clone();
    assert!(work::validate_context(&a, &wrong_e).is_err());
    assert!(!progress::current_solved(&a, &wrong_e));
    let mut wrong_b = b.clone();
    wrong_b.etherchannel_teaching = e.etherchannel_teaching.clone();
    assert!(file_backup_workspace::context(&c, &wrong_b).is_err());
    assert!(!progress::current_solved(&c, &wrong_b));
    assert!(progress::current_solved(&a, &e));
    assert!(progress::current_solved(&c, &b));
}
