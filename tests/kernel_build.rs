// Finite source/configuration fixtures; no real kernel, compiler, boot or host operations.
use dlms_learning_lab::kernel_build::{self as k, Build, Config, Feature};
use std::collections::BTreeSet;

fn config() -> Config {
    Config {
        modules: true,
        initrd: true,
        nvme: Feature::Module,
        ext4: Feature::Module,
        localversion: "-training".into(),
    }
}

#[test]
fn configuration_fragment_parses_values_and_explicit_disabled_settings() {
    let text = "CONFIG_MODULES=y\nCONFIG_BLK_DEV_INITRD=y\nCONFIG_BLK_DEV_NVME=m\nCONFIG_EXT4_FS=m\nCONFIG_LOCALVERSION=\"-training\"\n";
    assert_eq!(Config::parse(text).unwrap(), config());
    let disabled = text
        .replace("CONFIG_MODULES=y", "# CONFIG_MODULES is not set")
        .replace("CONFIG_BLK_DEV_NVME=m", "CONFIG_BLK_DEV_NVME=y")
        .replace("CONFIG_EXT4_FS=m", "CONFIG_EXT4_FS=y");
    let parsed = Config::parse(&disabled).unwrap();
    assert!(!parsed.modules);
    assert_eq!(parsed.nvme, Feature::BuiltIn);
    for bad in [
        text.to_owned() + "CONFIG_MODULES=y\n",
        text.to_owned() + "CONFIG_UNKNOWN=y\n",
        text.replace("=m", "=yes"),
        text.replace("CONFIG_MODULES=y", "CONFIG_MODULES=n"),
        text.replace("\"-training\"", "\"../../host\""),
        text.replace("CONFIG_LOCALVERSION=\"-training\"\n", ""),
    ] {
        assert!(
            Config::parse(&bad).is_err(),
            "accepted malformed/unsupported fragment: {bad}"
        );
    }
}

#[test]
fn root_support_accepts_builtin_and_matching_modular_alternatives() {
    let mut built = Build::from_config("source-a", "6.6.50", config()).unwrap();
    assert!(!k::root_ready(&built, None, &BTreeSet::new()).passed);
    built.image = true;
    built.modules = true;
    let inventory = built.config.loadable_inventory();
    assert!(!k::root_ready(&built, None, &inventory).passed);
    assert!(!k::root_ready(&built, Some("6.6.49-training"), &inventory).passed);
    assert!(k::root_ready(&built, Some(&built.release), &inventory).passed);
    for missing in &inventory {
        let mut partial = inventory.clone();
        partial.remove(missing);
        assert!(
            !k::root_ready(&built, Some(&built.release), &partial).passed,
            "missing dependency {missing}"
        );
    }
    let mut alternative = config();
    alternative.modules = false;
    alternative.initrd = false;
    alternative.nvme = Feature::BuiltIn;
    alternative.ext4 = Feature::BuiltIn;
    let mut built = Build::from_config("source-a", "6.6.50", alternative).unwrap();
    built.image = true;
    assert!(k::root_ready(&built, None, &BTreeSet::new()).passed);
}

#[test]
fn source_or_configuration_changes_do_not_reuse_a_current_build_claim() {
    let c = config();
    let built = Build::from_config("source-a", "6.6.50", c.clone()).unwrap();
    assert!(built.current_for("source-a", "6.6.50", &c));
    assert!(!built.current_for("source-b", "6.6.50", &c));
    assert!(!built.current_for("source-a", "6.6.51", &c));
    let mut changed = c;
    changed.localversion = "-other".into();
    assert!(!built.current_for("source-a", "6.6.50", &changed));
}

#[test]
fn bounded_validation_and_partial_boot_feedback_do_not_claim_success() {
    assert!(Config::parse(&"x".repeat(k::CONFIG_CAP + 1)).is_err());
    assert!(Config::parse("\0").is_err());
    assert!(Build::from_config("../host", "6.6.50", config()).is_err());
    assert!(Build::from_config("source-a", "../host", config()).is_err());
    let mut build = Build::from_config("source-a", "6.6.50", config()).unwrap();
    build.image = true;
    build.modules = true;
    let inventory = build.config.loadable_inventory();
    build.config.nvme = Feature::Disabled;
    let r = k::root_ready(&build, Some(&build.release), &inventory);
    assert!(!r.passed);
    assert!(r.reason.contains("NVMe"));
    build.config.nvme = Feature::Module;
    build.config.ext4 = Feature::Disabled;
    let r = k::root_ready(&build, Some(&build.release), &inventory);
    assert!(!r.passed);
    assert!(r.reason.contains("ext4"));
    build.config.ext4 = Feature::Module;
    build.config.initrd = false;
    let r = k::root_ready(&build, Some(&build.release), &inventory);
    assert!(!r.passed);
    assert!(r.reason.contains("initramfs"));
    build.config.modules = false;
    assert!(!build.current_for("source-a", "6.6.50", &build.config));
    assert!(!k::root_ready(&build, Some(&build.release), &inventory).passed);
}

#[test]
fn partial_rebuilds_keep_image_modules_and_initramfs_separate() {
    let mut outputs = k::Artifacts::default();
    let a = config();
    outputs
        .build("source-a", "6.6.50", &a, k::Target::All)
        .unwrap();
    assert!(
        !outputs.readiness().passed,
        "A modular image has no initramfs yet"
    );
    outputs.assemble_initramfs().unwrap();
    assert!(outputs.readiness().passed);
    let fallback = outputs.clone();
    let old_modules = outputs.modules.clone();
    let old_initramfs = outputs.initramfs.clone();
    let mut b = a;
    b.localversion = "-updated".into();
    outputs
        .build("source-a", "6.6.50", &b, k::Target::Image)
        .unwrap();
    assert_eq!(outputs.modules, old_modules);
    assert_eq!(outputs.initramfs, old_initramfs);
    assert!(
        !outputs.readiness().passed,
        "Building only the new image must not rebuild the other outputs"
    );
    let before = outputs.clone();
    assert!(
        outputs.assemble_initramfs().is_err(),
        "An image-bound assembly refuses stale module inputs"
    );
    assert_eq!(outputs, before);
    assert!(!outputs.readiness().passed);
    outputs
        .build("source-a", "6.6.50", &b, k::Target::Modules)
        .unwrap();
    assert!(
        !outputs.readiness().passed,
        "The old initramfs must not silently update"
    );
    outputs.assemble_initramfs().unwrap();
    assert!(outputs.readiness().passed);
    assert!(
        fallback.readiness().passed,
        "Earlier independent fallback outputs remain usable"
    );
}

#[test]
fn an_assembled_initramfs_retains_its_inputs_when_working_modules_change() {
    let mut outputs = k::Artifacts::default();
    let c = config();
    outputs
        .build("source-a", "6.6.50", &c, k::Target::All)
        .unwrap();
    outputs.assemble_initramfs().unwrap();
    let old_initramfs = outputs.initramfs.clone();
    outputs
        .build(
            "different-source-same-version",
            "6.6.50",
            &c,
            k::Target::Modules,
        )
        .unwrap();
    assert_eq!(outputs.initramfs, old_initramfs);
    assert!(outputs.readiness().passed);
    let before = outputs.clone();
    assert!(outputs.assemble_initramfs().is_err());
    assert_eq!(outputs, before);
    let mut other = k::Artifacts::default();
    other
        .build(
            "different-source-same-version",
            "6.6.50",
            &c,
            k::Target::All,
        )
        .unwrap();
    other.assemble_initramfs().unwrap();
    outputs.initramfs = other.initramfs;
    let r = outputs.readiness();
    assert!(!r.passed);
    assert!(r.reason.contains("Matching release names alone"));
}

#[test]
fn explicit_modules_target_fails_atomically_when_modules_are_disabled() {
    let mut outputs = k::Artifacts::default();
    let before = outputs.clone();
    assert!(
        outputs.assemble_initramfs().is_err(),
        "No image exists to bind this assembly to"
    );
    assert_eq!(outputs, before);
    let mut c = config();
    c.modules = false;
    c.initrd = false;
    c.nvme = Feature::BuiltIn;
    c.ext4 = Feature::BuiltIn;
    assert!(
        outputs
            .build("source-a", "6.6.50", &c, k::Target::Modules)
            .is_err()
    );
    assert_eq!(outputs, before);
    outputs
        .build("source-a", "6.6.50", &c, k::Target::All)
        .unwrap();
    assert!(outputs.image.is_some());
    assert!(outputs.modules.is_none());
    assert!(outputs.readiness().passed);
    outputs.assemble_initramfs().unwrap();
    assert!(outputs.initramfs.as_ref().unwrap().included.is_empty());
    assert!(outputs.initramfs.as_ref().unwrap().module_build.is_none());
    assert!(
        !outputs.readiness().passed,
        "This image cannot load a selected external initramfs"
    );
    outputs.initramfs = None;
    assert!(
        outputs.readiness().passed,
        "A built-in-only image may still use the bare-device path"
    );
    c.initrd = true;
    outputs
        .build("source-a", "6.6.50", &c, k::Target::All)
        .unwrap();
    outputs.assemble_initramfs().unwrap();
    assert!(
        outputs.readiness().passed,
        "Early userspace does not require loadable kernel drivers"
    );
}

#[test]
fn saved_artifact_shapes_are_bounded_and_cannot_launder_missing_builds() {
    let mut outputs = k::Artifacts::default();
    outputs
        .build("source-a", "6.6.50", &config(), k::Target::All)
        .unwrap();
    outputs.assemble_initramfs().unwrap();
    let serialized = serde_json::to_value(&outputs).unwrap();
    let restored: k::Artifacts = serde_json::from_value(serialized.clone()).unwrap();
    assert_eq!(restored, outputs);
    assert!(restored.readiness().passed);
    for field in ["source_id", "release"] {
        let mut raw = serialized.clone();
        raw["image"][field] = serde_json::Value::String("x".repeat(73));
        assert!(serde_json::from_value::<k::Artifacts>(raw).is_err());
    }
    let mut raw = serialized.clone();
    raw["image"]["config"]["localversion"] =
        serde_json::Value::String("-".to_owned() + &"x".repeat(24));
    assert!(serde_json::from_value::<k::Artifacts>(raw).is_err());
    for names in [
        vec!["ext4", "ext4"],
        vec!["../host"],
        vec!["nvme", "nvme_core", "ext4", "jbd2", "mbcache", "extra"],
    ] {
        let mut raw = serialized.clone();
        raw["initramfs"]["included"] = serde_json::to_value(names).unwrap();
        assert!(serde_json::from_value::<k::Artifacts>(raw).is_err());
    }
    let mut bad = outputs;
    bad.image.as_mut().unwrap().image = false;
    assert!(bad.validate_shape().is_err());
    assert!(!bad.readiness().passed);
    let before = bad.clone();
    assert!(
        bad.build("source-a", "6.6.50", &config(), k::Target::All)
            .is_err()
    );
    assert_eq!(bad, before);
}

#[test]
fn release_labels_match_the_existing_boot_inventory_and_kernel_limit() {
    let mut c = config();
    c.localversion = "-".to_owned() + &"x".repeat(15);
    let base = "6".repeat(48);
    let exact = Build::from_config("source-a", &base, c.clone()).unwrap();
    assert_eq!(exact.release.len(), 64);
    c.localversion.push('x');
    assert!(Build::from_config("source-a", &base, c).is_err());
}
