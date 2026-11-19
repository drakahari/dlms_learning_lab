//! Source-bound proof only: no authored catalog lab and no real RPM operation.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    rpm_packages as core, rpm_packages_terminal as terminal, rpm_packages_workspace as work,
    workspace,
};
#[path = "support/rpm_metadata_fixture.rs"]
mod f;
fn definition() -> work::Definition {
    serde_json::from_value(serde_json::json!({"initial":f::state(),"archive_paths":{
 "/rpms/site-v1.rpm":"site-v1","/rpms/site-v2.rpm":"site-v2","/rpms/renderer-v1.rpm":"renderer-v1"},"allow_metadata_restore":true,"requirements":[
 {"kind":"Installed","package":f::identity("site"),"version":{"relation":"Eq","evr":"1.0-1"}},
 {"kind":"OwnedMetadataMatches","package":f::identity("site"),"path":"/etc/site.conf"},
 {"kind":"OwnedMetadataMatches","package":f::identity("site"),"path":"/usr/share/site.txt"},
 {"kind":"PreserveInitialContents","path":"/etc/site.conf"},
 {"kind":"PreserveInitialContents","path":"/usr/share/site.txt"},
 {"kind":"PreserveInitialPackage","package":f::identity("renderer")},
 {"kind":"PreserveInitialFile","path":"/notes.txt"},
 {"kind":"DependenciesHealthy"}]})).unwrap()
}
fn restored(d: &work::Definition, s: &mut core::State) {
    let out = terminal::execute(d, s, "rpm --restore site");
    assert!(
        matches!(out, terminal::Outcome::Utility { status: 0, .. }),
        "{out:?}"
    );
}
#[test]
fn metadata_command_needs_authored_opt_in_and_preserves_default_serialization_and_refusals() {
    let d = definition();
    assert!(terminal::parse("rpm --restore site").is_err());
    assert!(terminal::validate_guidance("rpm --restore site").is_err());
    assert!(terminal::validate_guidance_for(&d, "rpm --restore site").is_ok());
    let mut old = d.clone();
    old.allow_metadata_restore = false;
    old.requirements = vec![work::Requirement::Installed {
        package: f::identity("site"),
        version: None,
    }];
    old.validate().unwrap();
    let raw = serde_json::to_value(&old).unwrap();
    assert!(raw.get("allow_metadata_restore").is_none());
    let read: work::Definition = serde_json::from_value(raw.clone()).unwrap();
    assert!(!read.allow_metadata_restore);
    assert_eq!(serde_json::to_value(read).unwrap(), raw);
    let mut s = old.initial.clone();
    let exact = s.clone();
    assert!(matches!(
        terminal::execute(&old, &mut s, "rpm --restore site"),
        terminal::Outcome::SimulatorUnsupported { .. }
    ));
    assert_eq!(s, exact);
    let mut invalid = serde_json::to_value(&d).unwrap();
    invalid["allow_metadata_restore"] = false.into();
    assert!(serde_json::from_value::<work::Definition>(invalid).is_err());
    for c in [
        "rpm --restore site renderer",
        "rpm --restore --all",
        "rpm -V site",
        "rpm --restore site; touch /tmp/no",
        "rpm --restore $(printf site)",
        "rpm --restore /etc/site.conf",
    ] {
        let before = s.clone();
        assert!(
            matches!(
                terminal::execute(&d, &mut s, c),
                terminal::Outcome::SimulatorUnsupported { .. }
            ),
            "{c}"
        );
        assert_eq!(s, before);
    }
}
#[test]
fn metadata_state_results_are_specific_recomputed_and_distinct_from_content_repair() {
    let d = definition();
    let mut s = d.initial.clone();
    let rows = d.checks(&s);
    assert!(rows[0].0);
    assert!(!rows[1].0);
    for text in ["mode 0640", "owner alex", "group ops", "modified time 99"] {
        assert!(rows[1].1.contains(text), "{rows:?}");
    }
    let wrong = terminal::execute(&d, &mut s, "rpm --reinstall /rpms/site-v1.rpm");
    assert!(matches!(
        wrong,
        terminal::Outcome::SimulatorUnsupported { .. }
    ));
    assert!(!d.checks(&s)[1].0);
    restored(&d, &mut s);
    assert!(d.checks(&s).iter().all(|r| r.0));
    assert_eq!(
        s.runtime().files["/etc/site.conf"].contents,
        "route=local\n"
    );
    assert!(
        s.file_issues(&f::identity("site"))
            .unwrap()
            .iter()
            .any(|r| r.differences == [core::Difference::Contents])
    );
    let out = terminal::execute(&d, &mut s, "rpm --reinstall /rpms/renderer-v1.rpm");
    assert!(matches!(out, terminal::Outcome::Utility { status: 0, .. }));
    assert!(
        !d.checks(&s)[5].0,
        "Repairing unrequested content must invalidate the stated preservation goal"
    );
    let mut mixed = d.clone();
    mixed
        .requirements
        .retain(|r| !matches!(r, work::Requirement::PreserveInitialPackage { .. }));
    mixed
        .requirements
        .push(work::Requirement::OwnedPayloadMatches {
            package: f::identity("renderer"),
            path: "/usr/share/renderer.txt".into(),
        });
    mixed.validate().unwrap();
    for commands in [
        [
            "rpm --restore site",
            "rpm --reinstall /rpms/renderer-v1.rpm",
        ],
        [
            "rpm --reinstall /rpms/renderer-v1.rpm",
            "rpm --restore site",
        ],
    ] {
        let mut state = mixed.initial.clone();
        for command in commands {
            assert!(matches!(
                terminal::execute(&mixed, &mut state, command),
                terminal::Outcome::Utility { status: 0, .. }
            ));
        }
        assert!(mixed.checks(&state).iter().all(|r| r.0));
    }
    let mut only_meta = mixed.initial.clone();
    restored(&mixed, &mut only_meta);
    assert!(
        !mixed.checks(&only_meta).last().unwrap().0,
        "Metadata-only success cannot repair damaged content"
    );
}
#[test]
fn metadata_missing_file_foreign_context_and_malformed_goal_refuse_without_mutation() {
    let d = definition();
    let mut runtime = d.initial.runtime().clone();
    runtime.files.remove("/usr/share/site.txt");
    let mut missing = core::State::new(d.initial.catalog().clone(), runtime).unwrap();
    let exact = missing.clone();
    let out = terminal::execute(&d, &mut missing, "rpm --restore site");
    assert!(
        matches!(out,terminal::Outcome::SimulatorUnsupported{reason}if reason.contains("Missing payload"))
    );
    assert_eq!(missing, exact);
    assert!(d.checks(&missing)[2].1.contains("missing"));
    let mut c = d.initial.catalog().clone();
    c.fixture = "foreign-metadata-world".into();
    let mut foreign = core::State::new(c, d.initial.runtime().clone()).unwrap();
    let exact = foreign.clone();
    assert!(matches!(
        terminal::execute(&d, &mut foreign, "rpm --restore site"),
        terminal::Outcome::SimulatorUnsupported { .. }
    ));
    assert_eq!(foreign, exact);
    let mut bad = serde_json::to_value(&d).unwrap();
    bad["requirements"][1]["path"] = "/outside".into();
    assert!(serde_json::from_value::<work::Definition>(bad).is_err());
    let mut raw = serde_json::to_value(&d.initial).unwrap();
    raw["allow_metadata_restore"] = true.into();
    assert!(serde_json::from_value::<core::State>(raw).is_err());
    let session = terminal::Session {
        command: "rpm --restore site".into(),
        ..Default::default()
    };
    let mut raw = serde_json::to_value(session).unwrap();
    raw["allow_metadata_restore"] = true.into();
    assert!(serde_json::from_value::<terminal::Session>(raw).is_err());
}
#[test]
fn metadata_progress_round_trip_keeps_real_state_draft_favorite_and_selected_reset() {
    let d = definition();
    let raw = serde_json::json!({"schema_version":1,"scenarios":[{"id":"metadata-proof","version":1,"track":"LPIC-1","topic":"Unregistered developer proof","title":"Metadata proof fixture","summary":"Typed simulation proof, not catalog curriculum.","objective":"Restore site metadata while retaining its local text and all other supplied data.","difficulty":"Intermediate","minutes":1,"steps":["Understand the task"],"guide":[],"initial":{"kind":"RpmPackages","simulation":d.initial},"goal":{"kind":"RpmPackages","definition":d},"solution":["rpm --restore site"],"hints":["Inspect the stated current metadata."],"explanation":"Developer proof, no authored catalog credit.","sources":["https://rpm.org/docs/6.0.x/man/rpm.8"],"simplifications":"Bounded regular-file metadata only; no host operation."}]});
    let s = pack::load(&raw.to_string()).unwrap().remove(0);
    let mut a = Attempt::new(&s, 1);
    workspace::prepare(&mut a);
    assert!(!progress::current_solved(&a, &s));
    engine::execute_for_goal(&mut a.state, &s.goal, "rpm --restore site").unwrap();
    assert!(progress::current_solved(&a, &s));
    a.workspace
        .as_mut()
        .unwrap()
        .rpm_session
        .as_mut()
        .unwrap()
        .command = "rpm -q site".into();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    p.favorites.insert(s.id.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).unwrap();
    let mut read = progress::load(&path, std::slice::from_ref(&s)).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&read).unwrap()
    );
    assert!(progress::current_solved(&read.attempts[&s.id], &s));
    let a = read.attempts.get_mut(&s.id).unwrap();
    let Goal::RpmPackages { definition } = &s.goal else {
        panic!()
    };
    let State::RpmPackages { simulation } = &mut a.state else {
        panic!()
    };
    terminal::execute(
        definition,
        simulation,
        "rpm --reinstall /rpms/renderer-v1.rpm",
    );
    assert!(!progress::current_solved(a, &s));
    *a = Attempt::new(&s, 2);
    workspace::prepare(a);
    assert_eq!(a.state, s.initial);
    assert!(read.favorites.contains(&s.id));
}

#[test]
fn metadata_teaching_capability_must_match_the_assignment_and_legacy_data_stays_exact() {
    let all = pack::accepted305().unwrap();
    let s = all.iter().find(|s| s.id == "lpic1-rpm-provider").unwrap();
    let Goal::RpmPackages { definition: legacy } = &s.goal else {
        panic!()
    };
    assert!(!legacy.allow_metadata_restore);
    assert!(
        serde_json::to_value(legacy)
            .unwrap()
            .get("allow_metadata_restore")
            .is_none()
    );
    let mut lesson = s.rpm_packages_teaching.as_ref().unwrap().clone();
    assert!(lesson.separate_from_definition(legacy));
    lesson.definition.allow_metadata_restore = true;
    lesson.validate().unwrap();
    assert!(!lesson.separate_from_definition(legacy));
    let mut opted = legacy.clone();
    opted.allow_metadata_restore = true;
    assert!(lesson.separate_from_definition(&opted));
    for old in all
        .iter()
        .filter(|s| matches!(s.initial, State::RpmPackages { .. }))
    {
        let Goal::RpmPackages { definition } = &old.goal else {
            panic!()
        };
        assert!(!definition.allow_metadata_restore);
        let raw = serde_json::to_value(definition).unwrap();
        let decoded: work::Definition = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), raw);
    }
}
