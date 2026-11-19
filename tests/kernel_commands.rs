use dlms_learning_lab::{kernel_build as core, kernel_commands as cmd, kernel_stage::Store};

fn config() -> core::Config {
    core::Config {
        modules: true,
        initrd: true,
        nvme: core::Feature::Module,
        ext4: core::Feature::Module,
        localversion: "-review".into(),
    }
}
fn run(
    a: &mut core::Artifacts,
    s: &mut Store,
    c: Option<&core::Config>,
    text: &str,
) -> Result<String, cmd::Error> {
    cmd::execute(
        cmd::Context {
            source_id: "source-a",
            base_release: "6.6.50",
            saved_config: c,
        },
        a,
        s,
        text,
    )
}

#[test]
fn supported_target_and_variable_forms_have_equivalent_state_outcomes() {
    let c = config();
    let mut a = core::Artifacts::default();
    let mut s = Store::default();
    let output = run(&mut a, &mut s, Some(&c), "make").unwrap();
    assert!(output.contains("6.6.50-review"));
    assert!(output.contains("5"));
    let mut alternate = core::Artifacts::default();
    run(
        &mut alternate,
        &mut Store::default(),
        Some(&c),
        " make   all ",
    )
    .unwrap();
    assert_eq!(alternate, a);
    let mut first = Store::default();
    run(
        &mut a,
        &mut first,
        Some(&c),
        "make INSTALL_MOD_PATH=/review modules_install",
    )
    .unwrap();
    run(
        &mut alternate,
        &mut s,
        Some(&c),
        "make modules_install INSTALL_MOD_PATH=/review",
    )
    .unwrap();
    assert_eq!(s, first);
    let installed = s.clone();
    let mut changed = c.clone();
    changed.localversion = "-new".into();
    run(&mut a, &mut s, Some(&changed), "make bzImage").unwrap();
    assert_eq!(s, installed);
    assert_ne!(
        a.image.as_ref().unwrap().release,
        a.modules.as_ref().unwrap().release
    );
}

#[test]
fn default_module_destination_is_virtual_root_not_the_graded_staging_root() {
    let c = config();
    let mut a = core::Artifacts::default();
    let mut s = Store::default();
    run(&mut a, &mut s, Some(&c), "make modules").unwrap();
    run(&mut a, &mut s, Some(&c), "make modules_install").unwrap();
    assert_eq!(s.modules[0].path, "/lib/modules/6.6.50-review");
    run(
        &mut a,
        &mut s,
        Some(&c),
        "make modules_install INSTALL_MOD_PATH=/review",
    )
    .unwrap();
    assert_eq!(s.modules.len(), 2);
    assert!(
        s.modules
            .iter()
            .any(|f| f.path == "/review/lib/modules/6.6.50-review")
    );
}

#[test]
fn unsupported_shell_makefile_and_escape_syntax_is_honest_and_has_no_effect() {
    let c = config();
    let mut a = core::Artifacts::default();
    let mut s = Store::default();
    run(&mut a, &mut s, Some(&c), "make all").unwrap();
    let before = (a.clone(), s.clone());
    for input in [
        "make all; rm -rf /",
        "make $(touch /host)",
        "make all | sh",
        "make -f /host/Makefile",
        "make -j1000",
        "make clean",
        "make bzImage modules",
        "make O=/host all",
        "make modules_install INSTALL_MOD_PATH=/review/../host",
        "make modules_install INSTALL_MOD_PATH=/review INSTALL_MOD_PATH=/other",
        "make \"all\"",
        "sh script.sh",
        "make all\nmake modules_install",
    ] {
        let e = run(&mut a, &mut s, Some(&c), input).unwrap_err();
        assert!(!e.detail.is_empty());
        assert_eq!((a.clone(), s.clone()), before, "mutated for {input}");
    }
    assert_eq!(cmd::parse("").unwrap_err().kind, cmd::ErrorKind::Syntax);
    assert_eq!(
        cmd::parse(&"x".repeat(cmd::MAX_INPUT + 1))
            .unwrap_err()
            .kind,
        cmd::ErrorKind::Limit
    );
}

#[test]
fn help_requires_no_configuration_and_does_not_build_or_award_anything() {
    let mut a = core::Artifacts::default();
    let mut s = Store::default();
    let before = (a.clone(), s.clone());
    let help = run(&mut a, &mut s, None, "make help").unwrap();
    assert!(help.contains("not a Linux shell") || help.contains("No host make"));
    assert_eq!((a.clone(), s.clone()), before);
    assert!(run(&mut a, &mut s, None, "make all").is_err());
    assert_eq!((a, s), before);
}

#[test]
fn module_support_and_fresh_build_boundaries_fail_before_installing_files() {
    let c = config();
    let mut a = core::Artifacts::default();
    let mut s = Store::default();
    assert!(run(&mut a, &mut s, Some(&c), "make modules_install").is_err());
    run(&mut a, &mut s, Some(&c), "make all").unwrap();
    let before = (a.clone(), s.clone());
    let mut changed = c.clone();
    changed.localversion = "-different".into();
    let e = run(
        &mut a,
        &mut s,
        Some(&changed),
        "make modules_install INSTALL_MOD_PATH=/review",
    )
    .unwrap_err();
    assert_eq!(e.kind, cmd::ErrorKind::Unsupported);
    assert_eq!((a.clone(), s.clone()), before);
    changed.modules = false;
    changed.nvme = core::Feature::BuiltIn;
    changed.ext4 = core::Feature::BuiltIn;
    assert!(run(&mut a, &mut s, Some(&changed), "make modules").is_err());
    assert!(run(&mut a, &mut s, Some(&changed), "make modules_install").is_err());
    assert_eq!((a, s), before);
}
