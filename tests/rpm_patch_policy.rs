use dlms_learning_lab::{
    rpm_packages as rpm, rpm_packages_workspace as work,
    rpm_patch_policy::{Advisory, Origin, Policy},
};
#[path = "support/rpm_metadata_fixture.rs"]
mod f;
#[path = "support/rpm_configuration_fixture.rs"]
mod shared;

fn definition() -> work::Definition {
    let state = shared::fixture(
        rpm::FileKind::ConfigNoreplace,
        Some("route=local\n"),
        "route=new\n",
    );
    let origins = state
        .catalog()
        .archives
        .keys()
        .map(|id| {
            (
                id.clone(),
                Origin {
                    product: "Training Linux".into(),
                    release: "8".into(),
                },
            )
        })
        .collect();
    serde_json::from_value(serde_json::json!({
        "initial": state,
        "archive_paths": {"/media/site-v1.rpm":"site-v1", "/media/site-v2.rpm":"site-v2",
            "/media/renderer.rpm":"renderer-v1"},
        "requirements": [{"kind":"AdvisoryFixed", "advisory":"TRAINING-SITE-1"},
            {"kind":"PreserveInitialContents", "path":"/etc/site.conf"},
            {"kind":"DependenciesHealthy"}],
        "patch_policy": Policy { advisories: vec![Advisory { id: "TRAINING-SITE-1".into(),
            package:f::identity("site"), product:"Training Linux".into(), release:"8".into(),
            fixed_in:rpm::Evr::parse("2.0-1").unwrap() }], origins }
    }))
    .unwrap()
}
#[test]
fn patch_outcomes_fail_initially_pass_after_update_and_keep_local_file() {
    let d = definition();
    assert_eq!(
        d.checks(&d.initial).iter().map(|r| r.0).collect::<Vec<_>>(),
        [false, true, true]
    );
    let mut s = d.initial.clone();
    work::apply(
        &d,
        &mut s,
        &rpm::Action::UpgradeSet {
            archives: vec!["site-v2".into()],
        },
    )
    .unwrap();
    assert!(d.checks(&s).iter().all(|r| r.0));
    assert_eq!(
        s.runtime().files["/etc/site.conf"].contents,
        "route=local\n"
    );
    assert_eq!(
        s.runtime().files["/etc/site.conf.rpmnew"].contents,
        "route=new\n"
    );
    assert!(!d.checks(&d.initial)[0].0, "No cached success");
}
#[test]
fn wrong_product_or_release_does_not_become_a_fix_from_a_high_version() {
    for other in [
        Origin {
            product: "Other Training Linux".into(),
            release: "8".into(),
        },
        Origin {
            product: "Training Linux".into(),
            release: "9".into(),
        },
    ] {
        let mut d = definition();
        // Keep one compatible supplied repair so validation can still prove reachability.
        d.patch_policy
            .as_mut()
            .unwrap()
            .origins
            .insert("site-v1".into(), other);
        let result = d.checks(&d.initial);
        assert!(!result[0].0);
        assert!(result[0].1.contains("another release"));
    }
}
#[test]
fn backported_release_order_uses_full_evr_not_upstream_number() {
    let d = definition();
    let mut policy = d.patch_policy.unwrap();
    policy.advisories[0].fixed_in = rpm::Evr::parse("1.0-2").unwrap();
    assert!(!policy.check(&d.initial, "TRAINING-SITE-1").0);
    assert!(
        policy
            .check(&d.initial, "TRAINING-SITE-1")
            .1
            .contains("backport")
    );
}
#[test]
fn legacy_definition_omits_policy_and_does_not_gain_config_authority() {
    let mut d = definition();
    d.patch_policy = None;
    d.requirements = vec![work::Requirement::Installed {
        package: f::identity("site"),
        version: None,
    }];
    assert!(d.validate().is_ok());
    assert!(
        serde_json::to_value(&d)
            .unwrap()
            .get("patch_policy")
            .is_none()
    );
    let mut s = d.initial.clone();
    let before = s.clone();
    assert!(
        work::apply(
            &d,
            &mut s,
            &rpm::Action::UpgradeSet {
                archives: vec!["site-v2".into()]
            }
        )
        .is_err()
    );
    assert_eq!(s, before);
}
#[test]
fn forged_or_missing_policy_and_foreign_state_are_refused() {
    let d = definition();
    let mut missing = d.clone();
    missing.patch_policy = None;
    assert!(missing.validate().is_err());
    let mut unknown = d.clone();
    unknown.patch_policy.as_mut().unwrap().advisories[0].id = "CVE-INVENTED".into();
    assert!(unknown.validate().is_err());
    let mut s = d.initial.clone();
    let mut foreign = d.clone();
    let c = foreign.initial.catalog().clone();
    let mut r = foreign.initial.runtime().clone();
    r.files.get_mut("/etc/site.conf").unwrap().contents = "foreign\n".into();
    // Changed runtime is learner state, but a changed immutable origin catalog is not.
    let mut c2 = c;
    c2.fixture = "foreign".into();
    foreign.initial = rpm::State::new(c2, r).unwrap();
    let before = s.clone();
    assert!(
        work::apply(
            &foreign,
            &mut s,
            &rpm::Action::UpgradeSet {
                archives: vec!["site-v2".into()]
            }
        )
        .is_err()
    );
    assert_eq!(s, before);
}
#[test]
fn malformed_policy_bounds_and_duplicate_origins_fail_before_value_body() {
    let d = definition();
    let mut v = serde_json::to_value(d).unwrap();
    v["patch_policy"]["advisories"][0]["product"] = serde_json::json!("x".repeat(65));
    assert!(serde_json::from_value::<work::Definition>(v).is_err());
    let raw = r#"{"advisories":[],"origins":{"a":{"product":"A","release":"1"},"a":null}}"#;
    let error = serde_json::from_str::<Policy>(raw).unwrap_err().to_string();
    assert!(error.contains("before body"));
}
