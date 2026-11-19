use dlms_learning_lab::{
    boot_entry as boot, kernel_build as core,
    kernel_stage::Store,
    kernel_workspace::{Action, Kind, Source, Task, Work},
};

fn config(modular: bool, suffix: &str) -> String {
    format!(
        "CONFIG_MODULES={}\nCONFIG_BLK_DEV_INITRD=y\nCONFIG_BLK_DEV_NVME={}\nCONFIG_EXT4_FS={}\nCONFIG_LOCALVERSION=\"{suffix}\"\n",
        if modular { "y" } else { "n" },
        if modular { "m" } else { "y" },
        if modular { "m" } else { "y" }
    )
}
fn entry(release: &str) -> String {
    format!(
        "title Candidate\nlinux /vmlinuz-{release}\ninitrd /initramfs-{release}.img\noptions root=UUID=11111111-2222-3333-4444-555555555555 ro\n"
    )
}
fn root() -> boot::Root {
    boot::Root {
        uuid: "11111111-2222-3333-4444-555555555555".into(),
        label: "Supplied ext4 root".into(),
        required_drivers: ["nvme_core", "nvme", "jbd2", "mbcache", "ext4"]
            .map(str::to_owned)
            .to_vec(),
        init_present: true,
    }
}
fn image() -> Task {
    Task{id:"model-image-task".into(),kind:Kind::Image,instructions:"Build the supplied target image with external initramfs support and working NVMe/ext4 root drivers.".into(),sources:vec![Source{id:"new-source".into(),release:"6.6.50".into(),title:"Supplied target source".into()}],initial_source:0,wanted_source:0,initial_config:config(true,"-review").replace("CONFIG_BLK_DEV_NVME=m","CONFIG_BLK_DEV_NVME=n"),stage_root:"/review".into(),initial_entry_filename:"candidate.conf".into(),initial_entry:String::new(),initial_store:Store::default(),root:None,require_initrd_support:true,reference:vec![Action::Config{text:config(true,"-review")},Action::SaveConfig,Action::Run{input:"make all".into()},Action::WorkingInitramfs,Action::Check]}
}
fn boot() -> Task {
    let mut t = image();
    t.id = "model-staged-task".into();
    t.kind = Kind::Boot;
    t.root = Some(root());
    t.sources.insert(
        0,
        Source {
            id: "old-source".into(),
            release: "6.6.49".into(),
            title: "Working fallback source".into(),
        },
    );
    t.wanted_source = 1;
    t.initial_source = 0;
    let mut a = core::Artifacts::default();
    a.build(
        "old-source",
        "6.6.49",
        &core::Config::parse(&config(true, "-working")).unwrap(),
        core::Target::All,
    )
    .unwrap();
    let path = t
        .initial_store
        .image("/review", a.image.as_ref().unwrap())
        .unwrap();
    t.initial_store
        .modules("/review", a.modules.as_ref().unwrap())
        .unwrap();
    t.initial_store
        .assemble_initramfs("/review", &path)
        .unwrap();
    t.initial_store
        .entry("/review", "fallback.conf", &entry("6.6.49-working"))
        .unwrap();
    t.initial_entry = "title Still editing\nlinux /missing\n".into();
    t.reference = vec![
        Action::Source { index: 1 },
        Action::Config {
            text: config(true, "-review"),
        },
        Action::SaveConfig,
        Action::Run {
            input: "make all".into(),
        },
        Action::StageImage,
        Action::Run {
            input: "make INSTALL_MOD_PATH=/review modules_install".into(),
        },
        Action::Initramfs {
            image_path: "/review/boot/vmlinuz-6.6.50-review".into(),
        },
        Action::EntryDraft {
            filename: "candidate.conf".into(),
            text: entry("6.6.50-review"),
        },
        Action::SaveEntry,
        Action::Check,
    ];
    t
}
fn solve(t: &Task) -> Work {
    t.validate().unwrap();
    let mut w = Work::new(t);
    for a in &t.reference {
        t.apply(&mut w, a).unwrap();
    }
    assert!(t.solved(&w), "{:?}", t.checks(&w));
    w
}

#[test]
fn actual_tasks_reach_working_outcomes_and_accept_supported_alternatives() {
    let t = image();
    solve(&t);
    let mut w = Work::new(&t);
    for a in [
        Action::Config {
            text: config(false, "-my-builtin"),
        },
        Action::SaveConfig,
        Action::Run {
            input: "make bzImage".into(),
        },
        Action::Check,
    ] {
        t.apply(&mut w, &a).unwrap();
    }
    assert!(t.solved(&w), "{:?}", t.checks(&w));
    let t = boot();
    solve(&t);
    let mut w = Work::new(&t);
    for a in [
        Action::Source { index: 1 },
        Action::Config {
            text: config(false, "-alternate"),
        },
        Action::SaveConfig,
        Action::Run {
            input: "make".into(),
        },
        Action::StageImage,
        Action::Initramfs {
            image_path: "/review/boot/vmlinuz-6.6.50-alternate".into(),
        },
        Action::EntryDraft {
            filename: "different-name.conf".into(),
            text: entry("6.6.50-alternate"),
        },
        Action::SaveEntry,
        Action::Check,
    ] {
        t.apply(&mut w, &a).unwrap();
    }
    assert!(t.solved(&w), "{:?}", t.checks(&w));
}

#[test]
fn partial_work_and_incorrect_repairs_have_causal_feedback_and_no_cached_success() {
    let t = image();
    let mut w = Work::new(&t);
    t.apply(
        &mut w,
        &Action::Run {
            input: "make all".into(),
        },
    )
    .unwrap();
    t.apply(&mut w, &Action::Check).unwrap();
    assert!(!t.solved(&w));
    assert!(
        t.checks(&w)
            .iter()
            .any(|c| !c.passed && c.detail.contains("NVMe"))
    );
    let mut w = solve(&t);
    t.apply(
        &mut w,
        &Action::Config {
            text: config(true, "-changed"),
        },
    )
    .unwrap();
    assert!(!w.checked);
    t.apply(&mut w, &Action::Check).unwrap();
    assert!(!t.solved(&w));
    assert!(
        t.checks(&w)
            .iter()
            .any(|c| !c.passed && c.detail.contains("Save the current"))
    );
    t.apply(&mut w, &Action::SaveConfig).unwrap();
    t.apply(&mut w, &Action::Check).unwrap();
    assert!(!t.solved(&w));
    assert!(
        t.checks(&w)
            .iter()
            .any(|c| !c.passed && c.detail.contains("earlier image"))
    );
}

#[test]
fn booting_the_old_fallback_or_changing_a_title_is_not_a_new_kernel_handover() {
    let t = boot();
    let mut w = Work::new(&t);
    t.apply(
        &mut w,
        &Action::EntryDraft {
            filename: "candidate.conf".into(),
            text: entry("6.6.49-working"),
        },
    )
    .unwrap();
    t.apply(&mut w, &Action::SaveEntry).unwrap();
    t.apply(&mut w, &Action::Check).unwrap();
    let c = t.checks(&w);
    assert!(
        c.iter()
            .any(|c| c.label == "Candidate boot trial" && c.passed)
    );
    assert!(
        c.iter()
            .any(|c| c.label == "Staged target image" && !c.passed)
    );
    assert!(!t.solved(&w));
    let mut w = solve(&t);
    t.apply(
        &mut w,
        &Action::EntryDraft {
            filename: "candidate.conf".into(),
            text: entry("6.6.50-review").replace("11111111", "99999999"),
        },
    )
    .unwrap();
    t.apply(&mut w, &Action::SaveEntry).unwrap();
    t.apply(&mut w, &Action::Check).unwrap();
    assert!(!t.solved(&w));
    assert!(
        t.checks(&w)
            .iter()
            .any(|c| c.label == "Working fallback preserved" && c.passed)
    );
}

#[test]
fn incomplete_drafts_errors_and_partial_state_round_trip_and_reset_independently() {
    let t = boot();
    let mut a = Work::new(&t);
    let mut b = Work::new(&t);
    let unchanged = b.clone();
    t.apply(
        &mut a,
        &Action::Config {
            text: "CONFIG_MODULES=maybe\n".into(),
        },
    )
    .unwrap();
    assert!(t.apply(&mut a, &Action::SaveConfig).is_err());
    t.apply(
        &mut a,
        &Action::CommandDraft {
            text: "make modules_inst".into(),
        },
    )
    .unwrap();
    let before = (a.outputs.clone(), a.store.clone());
    assert!(
        t.apply(
            &mut a,
            &Action::Run {
                input: "make all; touch /host".into()
            }
        )
        .is_err()
    );
    assert_eq!((a.outputs.clone(), a.store.clone()), before);
    assert!(!a.log.last().unwrap().ok);
    let raw = serde_json::to_vec(&a).unwrap();
    let restored: Work = serde_json::from_slice(&raw).unwrap();
    t.validate_work(&restored).unwrap();
    assert_eq!(restored, a);
    assert_eq!(b, unchanged);
    assert!(!t.solved(&a));
    t.apply(
        &mut b,
        &Action::CommandDraft {
            text: "make help".into(),
        },
    )
    .unwrap();
    let before = b.clone();
    t.apply(&mut a, &Action::Reset).unwrap();
    assert_eq!(a, Work::new(&t));
    assert_eq!(b, before);
}

#[test]
fn unknown_sources_bad_definitions_and_oversized_actions_are_rejected() {
    let t = image();
    let mut w = solve(&t);
    w.outputs.image.as_mut().unwrap().source_id = "foreign-source".into();
    let before = w.clone();
    assert!(t.apply(&mut w, &Action::Check).is_err());
    assert_eq!(w, before);
    assert!(!t.solved(&w));
    let mut t = image();
    t.reference = vec![Action::Check];
    assert!(t.validate().is_err());
    let mut t = boot();
    t.sources[1].id = t.sources[0].id.clone();
    assert!(t.validate().is_err());
    let mut w = Work::new(&image());
    assert!(
        image()
            .apply(&mut w, &Action::Source { index: usize::MAX })
            .is_err()
    );
    let before = w.clone();
    assert!(
        image()
            .apply(
                &mut w,
                &Action::Config {
                    text: "x".repeat(8193)
                }
            )
            .is_err()
    );
    assert_eq!(w, before);
    let mut raw = serde_json::to_value(w).unwrap();
    raw["command_draft"] = serde_json::Value::String("x".repeat(2049));
    assert!(serde_json::from_value::<Work>(raw).is_err());
}

#[test]
fn repeated_help_is_bounded_and_never_substitutes_for_the_assignment() {
    let t = image();
    let mut w = Work::new(&t);
    for _ in 0..140 {
        t.apply(
            &mut w,
            &Action::Run {
                input: "make help".into(),
            },
        )
        .unwrap();
    }
    assert_eq!(w.log.len(), 128);
    assert!(w.outputs.image.is_none());
    assert!(w.store.images.is_empty());
    assert!(!t.solved(&w));
    t.validate_work(&w).unwrap();
}

fn definition() -> dlms_learning_lab::kernel_workspace::Definition {
    let assignment = image();
    let mut example = image();
    example.id = "model-image-example".into();
    example.sources[0].id = "worked-source".into();
    example.sources[0].release = "6.1.99".into();
    example.sources[0].title = "Separate worked source".into();
    example.initial_config =
        config(true, "-practice").replace("CONFIG_EXT4_FS=m", "CONFIG_EXT4_FS=n");
    example.reference = vec![
        Action::Config {
            text: config(true, "-practice"),
        },
        Action::SaveConfig,
        Action::Run {
            input: "make".into(),
        },
        Action::WorkingInitramfs,
        Action::Check,
    ];
    dlms_learning_lab::kernel_workspace::Definition {
        version: 1,
        source_id: assignment.id.clone(),
        assignment,
        example,
    }
}

#[test]
fn source_bound_example_completion_never_changes_assignment_drafts_or_credit() {
    use dlms_learning_lab::kernel_workspace::Session;
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    let assignment = s.assignment.clone();
    for a in &d.example.reference {
        s.apply(&d, true, a).unwrap();
    }
    assert!(d.example.solved(&s.example.work));
    assert_eq!(s.assignment, assignment);
    assert!(!s.current_solved(&d));
    s.apply(
        &d,
        false,
        &Action::CommandDraft {
            text: "make bzIma".into(),
        },
    )
    .unwrap();
    let before = s.assignment.clone();
    s.apply(&d, true, &Action::Reset).unwrap();
    assert_eq!(s.assignment, before);
    assert_eq!(s.example.work, Work::new(&d.example));
    let raw = serde_json::to_vec(&s).unwrap();
    let loaded: Session = serde_json::from_slice(&raw).unwrap();
    loaded.validate_shape().unwrap();
    loaded.bind(&d).unwrap();
    assert_eq!(loaded, s);
}

#[test]
fn source_bound_resume_recomputes_success_and_refuses_changed_or_crossed_tasks() {
    use dlms_learning_lab::kernel_workspace::Session;
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    for a in &d.assignment.reference {
        s.apply(&d, false, a).unwrap();
    }
    assert!(s.current_solved(&d));
    let mut loaded: Session = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    loaded.validate_shape().unwrap();
    assert!(loaded.current_solved(&d));
    loaded
        .assignment
        .work
        .outputs
        .image
        .as_mut()
        .unwrap()
        .config
        .nvme = core::Feature::Disabled;
    assert!(!loaded.current_solved(&d));
    let mut crossed = s.clone();
    crossed.assignment = s.example.clone();
    let before = crossed.clone();
    assert!(crossed.apply(&d, false, &Action::Check).is_err());
    assert_eq!(crossed, before);
    assert!(!crossed.current_solved(&d));
    let mut changed = d.clone();
    changed.assignment.sources[0].release = "6.6.51".into();
    let before = s.clone();
    assert!(s.apply(&changed, false, &Action::Check).is_err());
    assert_eq!(s, before);
    assert!(!s.current_solved(&changed));
    let mut bad = s;
    bad.version = 2;
    assert!(bad.validate_shape().is_err());
    bad.version = 1;
    bad.assignment.source_task = "{not a supported task}".into();
    assert!(bad.validate_shape().is_err());
}
