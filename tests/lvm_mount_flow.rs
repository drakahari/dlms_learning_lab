//! Typed teaching/scoring prerequisite; never invokes a host command or mount.
#[path = "support/lvm_mount_fixture.rs"]
mod fixture;
use dlms_learning_lab::lvm_workspace::mount_flow::{self as f, Live};
use fixture::*;
fn pass(l: &Live, c: &f::Context, s: &dlms_learning_lab::lvm::State) -> bool {
    l.checks(c, &world(), s).iter().all(|r| r.0)
}
fn apply(
    l: &mut Live,
    c: &f::Context,
    s: &mut dlms_learning_lab::lvm::State,
    op: &f::Operation,
) -> String {
    l.apply(c, &world(), s, op).unwrap()
}
#[test]
fn wrong_initial_saved_and_live_data_need_both_repairs() {
    let mut s = world();
    let c = context(&s);
    let mut l = Live::new(&c, &s).unwrap();
    assert!(!pass(&l, &c, &s));
    apply(&mut l, &c, &mut s, &save("/dev/team/draft"));
    let rows = l.checks(&c, &world(), &s);
    assert!(rows[2].0);
    assert!(!rows[3].0);
    assert!(!rows[4].0);
    let before = (l.clone(), s.clone());
    assert!(
        l.apply(&c, &world(), &mut s, &cmd("mount /srv/report"))
            .is_err()
    );
    assert_eq!((l.clone(), s.clone()), before);
    apply(&mut l, &c, &mut s, &cmd("umount /srv/report"));
    assert!(!pass(&l, &c, &s));
    apply(&mut l, &c, &mut s, &cmd("mount /srv/report"));
    assert!(pass(&l, &c, &s));
    assert!(apply(&mut l, &c, &mut s, &cmd("cat /srv/report/keep.txt")).starts_with("team/draft:"));
}
#[test]
fn draft_is_inert_saved_work_and_live_work_each_invalidate_current_success() {
    let mut s = world();
    let c = context(&s);
    let mut l = Live::new(&c, &s).unwrap();
    for op in reference() {
        apply(&mut l, &c, &mut s, &op);
    }
    assert!(pass(&l, &c, &s));
    let saved = l.mounted.clone();
    l.set_draft(&c, &world(), &s, "unfinished\n".into())
        .unwrap();
    assert_eq!(l.mounted, saved);
    assert!(!pass(&l, &c, &s));
    l.save_draft(&c, &world(), &mut s).unwrap();
    assert!(!pass(&l, &c, &s));
    assert_eq!(l.mounted.storage.live(), saved.storage.live());
    apply(&mut l, &c, &mut s, &save("/dev/team/draft"));
    assert!(pass(&l, &c, &s));
    apply(&mut l, &c, &mut s, &cmd("umount /srv/report"));
    assert!(!pass(&l, &c, &s));
}
#[test]
fn behavior_grading_accepts_uuid_comments_whitespace_and_equivalent_valid_order() {
    let s0 = world();
    let c = context(&s0);
    for source in ["/dev/team/draft".into(), format!("UUID={UUID_A}")] {
        let mut s = s0.clone();
        let mut l = Live::new(&c, &s).unwrap();
        apply(&mut l, &c, &mut s, &cmd("umount /srv/report"));
        apply(
            &mut l,
            &c,
            &mut s,
            &f::Operation::SavePolicy {
                text: format!("# retained data\n{source}\t/srv/report\text4\trw,auto\t0\t2\n"),
            },
        );
        apply(&mut l, &c, &mut s, &cmd("mount /srv/report"));
        assert!(pass(&l, &c, &s));
    }
}
#[test]
fn read_only_and_noauto_are_real_requirements_not_assumed_success() {
    let mut s = world();
    let mut c = context(&s);
    c.targets[0].read_only = true;
    c.targets[0].automatic = false;
    let mut l = Live::new(&c, &s).unwrap();
    for op in reference() {
        apply(&mut l, &c, &mut s, &op);
    }
    assert!(!pass(&l, &c, &s));
    apply(
        &mut l,
        &c,
        &mut s,
        &f::Operation::SavePolicy {
            text: "/dev/team/draft /srv/report ext4 ro,noauto 0 2\n".into(),
        },
    );
    assert!(!pass(&l, &c, &s));
    apply(&mut l, &c, &mut s, &cmd("umount /srv/report"));
    apply(&mut l, &c, &mut s, &cmd("mount /srv/report"));
    assert!(pass(&l, &c, &s));
}
#[test]
fn same_text_wrong_original_is_not_a_false_positive() {
    let mut s0 = world();
    let contents = s0.groups["team"].volumes["draft"]
        .filesystem
        .as_ref()
        .unwrap()
        .files["/keep.txt"]
        .contents
        .clone();
    s0.groups
        .get_mut("team")
        .unwrap()
        .volumes
        .get_mut("reference")
        .unwrap()
        .filesystem
        .as_mut()
        .unwrap()
        .files
        .get_mut("/keep.txt")
        .unwrap()
        .contents = contents;
    let c = context(&s0);
    let l = Live::new(&c, &s0).unwrap();
    let rows = l.checks(&c, &s0, &s0);
    assert!(rows[4].0);
    assert!(!rows[2].0);
    assert!(!rows[3].0);
    assert!(!rows.iter().all(|r| r.0));
}
#[test]
fn mounted_original_survives_name_handover_but_saved_policy_must_be_repaired() {
    let mut s = world();
    let t = rename_task();
    t.validate(&s).unwrap();
    let c = &t.context;
    let mut l = Live::new(c, &s).unwrap();
    assert!(!pass(&l, c, &s));
    for op in &t.reference[..2] {
        apply(&mut l, c, &mut s, op);
    }
    assert!(!pass(&l, c, &s));
    assert!(
        l.mounted
            .read_path(&c.inventory, &s, "/srv/report/keep.txt")
            .unwrap()
            .starts_with("team/draft:")
    );
    apply(&mut l, c, &mut s, &save("/dev/production/records"));
    assert!(pass(&l, c, &s));
    apply(
        &mut l,
        c,
        &mut s,
        &cmd("lvrename production records temporary"),
    );
    assert!(!pass(&l, c, &s));
}
#[test]
fn teaching_reachability_requires_real_work_wrong_outcome_and_repair() {
    let s = world();
    task().validate(&s).unwrap();
    practice().validate(&s).unwrap();
    let mut t = task();
    t.reference = vec![cmd("findmnt")];
    assert!(t.validate(&s).is_err());
    t = task();
    t.reference = vec![save("/dev/team/draft")];
    assert!(t.validate(&s).is_err());
    let mut p = practice();
    p.experiment = cmd("findmnt");
    assert!(p.validate(&s).is_err());
    p = practice();
    p.repair = vec![cmd("findmnt")];
    assert!(p.validate(&s).is_err());
    p = practice();
    p.worked[0].explanation.clear();
    assert!(p.validate(&s).is_err());
    t = task();
    t.context.initial_policy = policy("/dev/team/draft");
    t.context.initial_live[0].original_volume = 4;
    assert!(t.validate(&s).is_err());
}
#[test]
fn sources_are_bounded_and_all_supplied_mountpoints_have_actual_file_requirements() {
    let s = world();
    let mut c = context(&s);
    c.targets[0].file = "/missing".into();
    assert!(c.shape(&s).is_err());
    c = context(&s);
    c.targets[0].file = "/../keep.txt".into();
    assert!(c.shape(&s).is_err());
    c = context(&s);
    c.targets[0].original_volume = 3;
    assert!(c.shape(&s).is_err());
    c = context(&s);
    c.inventory
        .mountpoints
        .push(dlms_learning_lab::lvm::mounts::Mountpoint(
            "/srv/extra".into(),
        ));
    assert!(c.shape(&s).is_err());
    c = context(&s);
    c.initial_live.push(c.initial_live[0].clone());
    assert!(c.shape(&s).is_err());
    c = context(&s);
    c.targets.push(c.targets[0].clone());
    assert!(c.shape(&s).is_err());
}
#[test]
fn serialized_drafts_current_mounts_and_bad_saved_policy_resume_exactly() {
    let mut s = world();
    let c = context(&s);
    let mut l = Live::new(&c, &s).unwrap();
    l.set_draft(&c, &world(), &s, "unsaved broken entry".into())
        .unwrap();
    let bytes = serde_json::to_vec(&(s.clone(), l.clone())).unwrap();
    let (now, restored): (dlms_learning_lab::lvm::State, Live) =
        serde_json::from_slice(&bytes).unwrap();
    assert_eq!((now.clone(), restored.clone()), (s.clone(), l.clone()));
    restored.validate(&c, &world(), &now).unwrap();
    l.save_draft(&c, &world(), &mut s).unwrap();
    assert!(!l.mounted.storage.validate_configuration().is_empty());
    let r: Live = serde_json::from_slice(&serde_json::to_vec(&l).unwrap()).unwrap();
    r.validate(&c, &world(), &s).unwrap();
    assert_eq!(r, l);
    assert_eq!(Live::new(&c, &world()).unwrap().draft, c.initial_policy);
}
#[test]
fn unsupported_or_host_like_commands_are_atomic_and_history_cannot_award_success() {
    let mut s = world();
    let c = context(&s);
    let mut l = Live::new(&c, &s).unwrap();
    for text in [
        "mount -a",
        "mount /dev/team/draft /srv/report",
        "mount -o remount /srv/report",
        "umount -l /srv/report",
        "cat /etc/passwd",
        "cat /srv/report/../audit/keep.txt",
        "lvs; true",
        "sh -c true",
        "lvcreate -l 1 -n extra team",
        "lvremove -y team/draft",
        "vgchange -ay",
        "findmnt /home",
        "blkid /dev/sda",
        "cat /srv/report/keep.txt > /tmp/out",
    ] {
        let before = (s.clone(), l.clone());
        assert!(l.apply(&c, &world(), &mut s, &cmd(text)).is_err(), "{text}");
        assert_eq!((s.clone(), l.clone()), before, "{text}");
    }
    for text in [
        "blkid",
        "findmnt",
        "findmnt /srv/report",
        "cat /srv/report/keep.txt",
        "pvs",
        "vgs",
        "lvs",
    ] {
        assert!(!apply(&mut l, &c, &mut s, &cmd(text)).is_empty());
        assert!(!pass(&l, &c, &s));
    }
}
#[test]
fn replaced_original_or_modified_payload_is_not_admitted_as_same_task() {
    let s = world();
    let c = context(&s);
    let l = Live::new(&c, &s).unwrap();
    let mut bad = s.clone();
    bad.groups
        .get_mut("team")
        .unwrap()
        .volumes
        .get_mut("draft")
        .unwrap()
        .filesystem
        .as_mut()
        .unwrap()
        .files
        .get_mut("/keep.txt")
        .unwrap()
        .contents = "forged".into();
    assert!(l.validate(&c, &s, &bad).is_err());
    bad = s.clone();
    bad.execute("lvcreate -l 1 -n extra team").unwrap();
    assert!(l.validate(&c, &s, &bad).is_err());
    bad = s.clone();
    bad.identity
        .as_mut()
        .unwrap()
        .volumes
        .insert("team/draft".into(), 6);
    bad.identity.as_mut().unwrap().next = 7;
    assert!(l.validate(&c, &s, &bad).is_err());
}
#[test]
fn unrelated_raw_volume_activation_invalidates_otherwise_good_mount_work() {
    let mut s = world();
    let c = context(&s);
    let mut l = Live::new(&c, &s).unwrap();
    for op in reference() {
        apply(&mut l, &c, &mut s, &op);
    }
    assert!(pass(&l, &c, &s));
    apply(&mut l, &c, &mut s, &cmd("vgchange -ay reserve"));
    assert!(!pass(&l, &c, &s));
    apply(&mut l, &c, &mut s, &cmd("vgchange -an reserve"));
    assert!(pass(&l, &c, &s));
}
