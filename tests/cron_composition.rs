//! Composed-family admission and exact-wire size checks. Fixed test fixtures only;
//! no host execution, historical writer regeneration or progress installation.
use dlms_learning_lab::{
    cron_workspace as cron,
    engine::{Goal, State},
    etherchannel_teaching as ether_teach, etherchannel_workspace as ether,
    file_backup_teaching as backup_teach, file_backup_workspace as backup,
    pack::{self, Scenario},
    progress::{self, Attempt},
    workspace,
};
fn sources() -> (Scenario, Scenario, Scenario) {
    let mut all = pack::bundled().unwrap();
    assert_eq!(all.len(), 176);
    let cron = all.remove(
        all.iter()
            .position(|s| s.id == "lpic1-cron-weekday-report")
            .unwrap(),
    );
    let ether = all.remove(
        all.iter()
            .position(|s| s.id == "ccna-lacp-initiation")
            .unwrap(),
    );
    let mut extras: pack::Pack =
        serde_json::from_str(include_str!("../scenarios/file-backup-v1.json")).unwrap();
    let backup = extras.scenarios.remove(0);
    let Goal::FileBackup { definition } = &backup.goal else {
        panic!()
    };
    definition.validate().unwrap();
    backup
        .file_backup_teaching
        .as_ref()
        .unwrap()
        .validate()
        .unwrap();
    (cron, ether, backup)
}
fn cron_attempt(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 3);
    cron::budget::prepare(&mut a, s).unwrap();
    a
}
#[test]
fn all_three_families_reject_each_others_valid_metadata_and_source_without_mutation() {
    let (c, e, b) = sources();
    let mut ca = cron_attempt(&c);
    let mut ea = Attempt::new(&e, 3);
    ether_teach::prepare(&mut ea, &e, true).unwrap();
    let mut ba = Attempt::new(&b, 3);
    backup_teach::prepare(&mut ba, b.file_backup_teaching.as_ref().unwrap(), true).unwrap();
    let cs = ca.workspace.as_ref().unwrap().cron_session.clone();
    let cl = ca.workspace.as_ref().unwrap().cron_learning.clone();
    let ed = ea.workspace.as_ref().unwrap().etherchannel_draft.clone();
    let el = ea.workspace.as_ref().unwrap().etherchannel_learning.clone();
    let bl = ba.workspace.as_ref().unwrap().file_backup_learning.clone();
    assert!(
        cs.as_ref().unwrap().valid()
            && cl.as_ref().unwrap().valid()
            && ed.as_ref().unwrap().valid()
            && el.as_ref().unwrap().valid()
            && bl.as_ref().unwrap().valid()
    );
    for payload in 0..4 {
        let w = ca.workspace.as_mut().unwrap();
        match payload {
            0 => w.etherchannel_draft = ed.clone(),
            1 => w.etherchannel_learning = el.clone(),
            2 => {
                w.etherchannel_draft = ed.clone();
                w.etherchannel_learning = el.clone();
            }
            3 => w.file_backup_learning = bl.clone(),
            _ => unreachable!(),
        }
        assert!(w.valid());
        let before = serde_json::to_vec(&ca).unwrap();
        assert!(cron::context(&ca, &c).is_err());
        assert!(!progress::current_solved(&ca, &c));
        assert!(
            cron::budget::edit_attempt(
                &mut ca,
                &c,
                cron::budget::SessionEdit {
                    actor: Some("root"),
                    ..Default::default()
                },
                false
            )
            .is_err()
        );
        assert_eq!(serde_json::to_vec(&ca).unwrap(), before);
        ca = cron_attempt(&c);
    }
    for payload in 0..3 {
        let w = ea.workspace.as_mut().unwrap();
        match payload {
            0 => w.cron_session = cs.clone(),
            1 => w.cron_learning = cl.clone(),
            2 => {
                w.cron_session = cs.clone();
                w.cron_learning = cl.clone();
            }
            _ => unreachable!(),
        }
        assert!(w.valid());
        let before = serde_json::to_vec(&ea).unwrap();
        assert!(ether::validate_context(&ea, &e).is_err());
        assert!(!progress::current_solved(&ea, &e));
        assert_eq!(serde_json::to_vec(&ea).unwrap(), before);
        ea.workspace.as_mut().unwrap().cron_session = None;
        ea.workspace.as_mut().unwrap().cron_learning = None;
        let w = ba.workspace.as_mut().unwrap();
        match payload {
            0 => w.cron_session = cs.clone(),
            1 => w.cron_learning = cl.clone(),
            2 => {
                w.cron_session = cs.clone();
                w.cron_learning = cl.clone();
            }
            _ => unreachable!(),
        }
        assert!(w.valid());
        let before = serde_json::to_vec(&ba).unwrap();
        assert!(backup::context(&ba, &b).is_err());
        assert!(!progress::current_solved(&ba, &b));
        assert!(backup::perform(&mut ba, &b, backup::Scope::Assignment, "pwd").is_err());
        assert_eq!(serde_json::to_vec(&ba).unwrap(), before);
        ba.workspace.as_mut().unwrap().cron_session = None;
        ba.workspace.as_mut().unwrap().cron_learning = None;
    }
    for (mut source, payload) in [
        (c.clone(), 0),
        (c.clone(), 1),
        (e.clone(), 2),
        (b.clone(), 3),
    ] {
        match payload {
            0 => source.etherchannel_teaching = e.etherchannel_teaching.clone(),
            1 => source.file_backup_teaching = b.file_backup_teaching.clone(),
            2 | 3 => source.cron_teaching = c.cron_teaching.clone(),
            _ => unreachable!(),
        }
        match payload {
            0 | 1 => assert!(cron::context(&ca, &source).is_err()),
            2 => assert!(ether::validate_context(&ea, &source).is_err()),
            3 => assert!(backup::context(&ba, &source).is_err()),
            _ => unreachable!(),
        }
    }
    assert!(
        cron::context(&ca, &c).is_ok()
            && ether::validate_context(&ea, &e).is_ok()
            && backup::context(&ba, &b).is_ok(),
        "Foreign refusal must not damage independent positive contexts"
    );
}
#[test]
fn whole_attempt_projection_counts_every_new_field_and_json_escaping_before_refusing_foreign_edits()
{
    let (c, e, b) = sources();
    let mut a = cron_attempt(&c);
    let ed = ether::Draft::from_state(if let State::EtherChannel { simulation } = &e.initial {
        simulation
    } else {
        panic!()
    })
    .unwrap();
    let el = ether_teach::Learning::new(e.etherchannel_teaching.as_ref().unwrap(), true).unwrap();
    let bl = backup_teach::Learning::new(b.file_backup_teaching.as_ref().unwrap()).unwrap();
    for bits in 0u8..8 {
        let w = a.workspace.as_mut().unwrap();
        w.command = "retained ü \"quoted\"\n".into();
        w.etherchannel_draft = (bits & 1 != 0).then(|| ed.clone());
        w.etherchannel_learning = (bits & 2 != 0).then(|| el.clone());
        w.file_backup_learning = (bits & 4 != 0).then(|| bl.clone());
        let session = w.cron_session.as_ref().unwrap();
        let learning = w.cron_learning.as_ref().unwrap();
        let ssize = cron::size(session, cron::MAX_ATTEMPT).unwrap();
        let lsize = cron::size(learning, cron::MAX_ATTEMPT).unwrap();
        let State::Cron { simulation } = &a.state else {
            panic!()
        };
        let predicted =
            cron::budget::projected_attempt_size(&a, simulation, Some(ssize), Some(lsize));
        assert_eq!(
            predicted,
            Some(serde_json::to_vec(&a).unwrap().len()),
            "All present/absent new-family fields and escaped raw bytes must be charged: bits{bits}"
        );
        let before = serde_json::to_vec(&a).unwrap();
        if bits != 0 {
            assert!(cron::budget::edit_attempt(&mut a, &c, Default::default(), false).is_err());
            assert_eq!(serde_json::to_vec(&a).unwrap(), before);
        }
    }
}
#[test]
fn new_kind_without_optional_metadata_cannot_be_prepared_as_an_old_learning_family() {
    let (c, _, _) = sources();
    let all = pack::bundled().unwrap();
    for source in &all {
        let mut a = Attempt::new(&c, 3);
        let before = serde_json::to_vec(&a).unwrap();
        if let Some(l) = &source.accounts_teaching {
            dlms_learning_lab::accounts_teaching::prepare(&mut a, l, true);
        }
        if let Some(l) = &source.teaching {
            dlms_learning_lab::script_teaching::prepare(&mut a, l, true);
        }
        if let Some(l) = &source.service_teaching {
            dlms_learning_lab::service_teaching::prepare(&mut a, l, true);
        }
        workspace::prepare(&mut a);
        assert_eq!(
            serde_json::to_vec(&a).unwrap(),
            before,
            "No metadata exists yet: {}",
            source.id
        );
    }
    let source = &c;
    let mut a = Attempt::new(source, 3);
    cron::budget::prepare(&mut a, source).unwrap();
    assert!(
        a.workspace.as_ref().unwrap().cron_session.is_some()
            && a.workspace.as_ref().unwrap().cron_learning.is_some()
    );
    assert!(!progress::current_solved(&a, source));
    assert_eq!(
        a.workspace
            .as_ref()
            .unwrap()
            .cron_learning
            .as_ref()
            .unwrap()
            .view,
        dlms_learning_lab::script_teaching::View::Introduction
    );
}
