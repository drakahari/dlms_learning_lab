//! Source-bound draft, behavior, preservation and isolation regressions.
use super::*;
fn abi() -> core::Abi {
    core::Abi {
        class: core::Class::Elf64,
        machine: core::Machine::X86_64,
        byte_order: core::ByteOrder::Little,
    }
}
fn object(id: &str, path: &str, soname: &str, needed: &[&str]) -> core::Object {
    core::Object {
        identity: id.into(),
        path: path.into(),
        soname: soname.into(),
        abi: abi(),
        needed: needed.iter().map(|v| (*v).into()).collect(),
    }
}
fn state(site: &str, cache_task: bool) -> core::State {
    let own = format!("/opt/{site}/lib");
    let fallback = format!("/opt/{site}-fallback/lib");
    let mut catalog = core::Catalog {
        version: 1,
        directories: vec![
            "/lib64".into(),
            "/usr/lib64".into(),
            own.clone(),
            fallback.clone(),
        ],
        trusted: vec!["/lib64".into(), "/usr/lib64".into()],
        objects: vec![
            object("base", "/lib64/libbase.so.1", "libbase.so.1", &[]),
            object(
                "site",
                &format!("{own}/libreport.so.1.4"),
                "libreport.so.1",
                &["libbase.so.1"],
            ),
            object(
                "fallback",
                &format!("{fallback}/libreport.so.1.8"),
                "libreport.so.1",
                &["libbase.so.1"],
            ),
        ],
        programs: vec![
            core::Program {
                path: format!("/usr/bin/{site}-report"),
                abi: abi(),
                needed: vec!["libreport.so.1".into()],
            },
            core::Program {
                path: format!("/usr/bin/{site}-viewer"),
                abi: abi(),
                needed: vec!["libbase.so.1".into()],
            },
        ],
        aliases: vec![
            core::Alias {
                path: format!("{own}/libreport.so.1"),
                object: format!("{own}/libreport.so.1.4"),
            },
            core::Alias {
                path: format!("{fallback}/libreport.so.1"),
                object: format!("{fallback}/libreport.so.1.8"),
            },
        ],
    };
    if cache_task {
        catalog.objects.push(object(
            "helper",
            &format!("{own}/libhelper.so.2"),
            "libhelper.so.2",
            &["libbase.so.1"],
        ));
        catalog.programs.push(core::Program {
            path: format!("/usr/bin/{site}-worker"),
            abi: abi(),
            needed: vec!["libhelper.so.2".into()],
        });
    }
    let mut state = core::State::new(catalog, if cache_task { &fallback } else { "" }).unwrap();
    if cache_task {
        state.rebuild_cache().unwrap();
    }
    state
}
fn rule(name: &str, ids: &[&str]) -> SelectionRule {
    SelectionRule {
        needed: name.into(),
        allowed_identities: ids.iter().map(|s| (*s).into()).collect(),
    }
}
fn plan(site: &str, cache_task: bool) -> Plan {
    let mut launches = vec![LaunchGoal {
        program: format!("/usr/bin/{site}-report"),
        mode: if cache_task {
            LaunchMode::Clean
        } else {
            LaunchMode::SavedRecipe
        },
        selections: vec![
            rule(
                "libreport.so.1",
                if cache_task {
                    &["site"]
                } else {
                    &["site", "fallback"]
                },
            ),
            rule("libbase.so.1", &["base"]),
        ],
    }];
    if cache_task {
        launches.push(LaunchGoal {
            program: format!("/usr/bin/{site}-worker"),
            mode: LaunchMode::Clean,
            selections: vec![
                rule("libhelper.so.2", &["helper"]),
                rule("libbase.so.1", &["base"]),
            ],
        });
    }
    Plan {
        launches,
        protected_programs: vec![format!("/usr/bin/{site}-viewer")],
        require_current_cache: cache_task,
        require_clean_recipes: cache_task,
        preserve_configuration_and_cache: !cache_task,
    }
}
fn definition(cache_task: bool) -> Definition {
    let site = if cache_task { "harbor" } else { "cedar" };
    let example = if cache_task { "workshop" } else { "gallery" };
    let action = if cache_task {
        WorkedAction::Configuration {
            text: format!("/opt/{example}/lib\n"),
        }
    } else {
        WorkedAction::LaunchRecipe {
            program: format!("/usr/bin/{example}-report"),
            environment: Some(format!("/opt/{example}/lib")),
        }
    };
    let mut worked = vec![Step {
        title: "Use a supplied search directory".into(),
        explanation:
            "Save this example's setting explicitly; the assignment has different supplied paths."
                .into(),
        action,
    }];
    if cache_task {
        worked.push(Step {
            title: "Rebuild the virtual cache".into(),
            explanation: "Saving configuration alone leaves the old cached selections in use."
                .into(),
            action: WorkedAction::RebuildCache,
        });
    }
    Definition { initial:state(site,cache_task),initial_recipes:vec![],plan:plan(site,cache_task),lesson:Lesson {
        prerequisite_note:"Know the difference between a path, a filename and a process environment setting.".into(),
        description:"Inspect a dependency chain using supplied metadata; no ELF code executes.".into(),
        concepts:vec![Concept { title:"Names and supplied builds".into(),text:"A dependency name is looked up in search directories; the selected object can have its own further dependencies. SONAME is a name, not proof of ABI compatibility.".into() }],
        initial:state(example,cache_task),initial_recipes:vec![],plan:plan(example,cache_task),worked } }
}
fn save_launch(s: &mut Session, d: &Definition, site: &str, scope: Scope, dirs: &str) {
    let program = format!("/usr/bin/{site}-report");
    s.edit_launch(d, scope, &program, dirs.into()).unwrap();
    s.save_launch(d, scope, &program).unwrap();
}
fn solve_cache(s: &mut Session, d: &Definition, site: &str, scope: Scope, text: Option<&str>) {
    s.edit_configuration(
        d,
        scope,
        text.map(str::to_owned)
            .unwrap_or_else(|| format!("/opt/{site}/lib\n")),
    )
    .unwrap();
    s.save_configuration(d, scope).unwrap();
    s.rebuild_cache(d, scope).unwrap();
}
#[test]
fn both_distinct_plans_have_separate_worked_example_sources_and_reachable_teaching() {
    for cache in [false, true] {
        let d = definition(cache);
        d.validate().unwrap();
        d.verify_worked().unwrap();
        assert_ne!(d.initial.catalog, d.lesson.initial.catalog);
        let s = Session::new(&d).unwrap();
        assert!(!s.current_success(&d));
        assert_eq!(d.plan.launches.len(), if cache { 2 } else { 1 });
    }
}
#[test]
fn temporary_env_inspection_never_saves_a_recipe_or_awards_assignment_credit() {
    let d = definition(false);
    let mut s = Session::new(&d).unwrap();
    let old = s.assignment.state.clone();
    let row = s
        .run(
            &d,
            Scope::Assignment,
            "env LD_LIBRARY_PATH=/opt/cedar/lib ldd /usr/bin/cedar-report",
        )
        .unwrap();
    assert_eq!(row.outcome, Outcome::Supported);
    assert!(
        row.output
            .contains("libreport.so.1 => /opt/cedar/lib/libreport.so.1")
    );
    assert!(row.output.contains("no simulated launch recipe"));
    assert_eq!(s.assignment.state, old);
    assert!(s.assignment.recipes.iter().all(|r| r.environment.is_none()));
    assert!(!s.assignment.checked);
    assert!(!s.check(&d, Scope::Assignment).unwrap().passed());
    assert!(!s.current_success(&d));
    let clean = s
        .run(&d, Scope::Assignment, "ldd /usr/bin/cedar-report")
        .unwrap();
    assert!(clean.output.contains("not found"));
}
#[test]
fn saved_recipe_is_behavior_graded_and_live_drafts_invalidate_success() {
    let d = definition(false);
    let mut s = Session::new(&d).unwrap();
    save_launch(&mut s, &d, "cedar", Scope::Assignment, "/opt/cedar/lib");
    assert!(
        !s.current_success(&d),
        "Applying is not automatically checking"
    );
    assert!(s.check(&d, Scope::Assignment).unwrap().passed());
    assert!(s.current_success(&d));
    let before = s.assignment.state.clone();
    s.edit_launch(
        &d,
        Scope::Assignment,
        "/usr/bin/cedar-report",
        "/missing\0path".into(),
    )
    .unwrap();
    assert!(!s.current_success(&d));
    assert!(!s.check(&d, Scope::Assignment).unwrap().passed());
    assert!(
        s.save_launch(&d, Scope::Assignment, "/usr/bin/cedar-report")
            .is_err()
    );
    assert_eq!(s.assignment.state, before);
    assert_eq!(s.assignment.launch_drafts[0].value, "/missing\0path");
}
#[test]
fn equivalent_supplied_search_directories_and_ordered_path_forms_pass() {
    let d = definition(false);
    for dirs in [
        "/opt/cedar-fallback/lib",
        "/opt/cedar/lib:/opt/cedar-fallback/lib",
        "/opt/cedar-fallback/lib;/opt/cedar/lib",
        "/opt/cedar/lib:/opt/cedar/lib",
    ] {
        let mut s = Session::new(&d).unwrap();
        save_launch(&mut s, &d, "cedar", Scope::Assignment, dirs);
        assert!(s.check(&d, Scope::Assignment).unwrap().passed(), "{dirs}");
        assert!(s.assignment.state.library_path.is_none());
    }
}
#[test]
fn cache_handover_requires_save_then_explicit_rebuild_for_multiple_programs() {
    let d = definition(true);
    let mut s = Session::new(&d).unwrap();
    let cache = s.assignment.state.cache.clone();
    let old = s.check(&d, Scope::Assignment).unwrap();
    assert!(!old.passed());
    assert!(old.checks.iter().any(|c| c.passed));
    s.edit_configuration(&d, Scope::Assignment, "/opt/harbor/lib\n".into())
        .unwrap();
    assert!(!s.current_report(&d, Scope::Assignment).unwrap().passed());
    assert_eq!(s.assignment.state.cache, cache);
    s.save_configuration(&d, Scope::Assignment).unwrap();
    assert_eq!(s.assignment.state.cache, cache);
    let stale = s.check(&d, Scope::Assignment).unwrap();
    assert!(
        stale
            .checks
            .iter()
            .any(|r| r.label.contains("cache") && !r.passed)
    );
    let row = s.run(&d, Scope::Assignment, "ldconfig").unwrap();
    assert_eq!(row.outcome, Outcome::Supported);
    assert!(s.check(&d, Scope::Assignment).unwrap().passed());
    assert!(s.current_success(&d));
    assert!(
        s.run(&d, Scope::Assignment, "ldconfig -p")
            .unwrap()
            .output
            .contains("libhelper.so.2")
    );
    assert!(
        s.current_success(&d),
        "Read-only cache inspection does not change the saved working state"
    );
}
#[test]
fn equivalent_configuration_comments_duplicates_and_default_directories_pass() {
    let d = definition(true);
    for text in [
        "/opt/harbor/lib",
        "# site\n/opt/harbor/lib # supplied directory\n",
        "/lib64\n/opt/harbor/lib\n/opt/harbor/lib\n/usr/lib64\n",
    ] {
        let mut s = Session::new(&d).unwrap();
        solve_cache(&mut s, &d, "harbor", Scope::Assignment, Some(text));
        assert!(s.check(&d, Scope::Assignment).unwrap().passed());
    }
}
#[test]
fn rebuilding_before_saving_uses_applied_configuration_not_editor_text() {
    let d = definition(true);
    let mut s = Session::new(&d).unwrap();
    s.edit_configuration(&d, Scope::Assignment, "/opt/harbor/lib".into())
        .unwrap();
    let row = s.run(&d, Scope::Assignment, "ldconfig").unwrap();
    assert!(row.output.contains("Unsaved editor drafts were not used"));
    assert!(s.assignment.state.configuration.contains("fallback"));
    assert!(!s.check(&d, Scope::Assignment).unwrap().passed());
}
#[test]
fn cache_ambiguity_and_malformed_drafts_refuse_atomically_and_remain_resumable() {
    let d = definition(true);
    let mut s = Session::new(&d).unwrap();
    s.edit_configuration(
        &d,
        Scope::Assignment,
        "/opt/harbor/lib\n/opt/harbor-fallback/lib\n".into(),
    )
    .unwrap();
    s.save_configuration(&d, Scope::Assignment).unwrap();
    let before = s.assignment.state.clone();
    let row = s.run(&d, Scope::Assignment, "ldconfig").unwrap();
    assert_eq!(row.outcome, Outcome::Unsupported);
    assert_eq!(s.assignment.state, before);
    s.edit_configuration(
        &d,
        Scope::Assignment,
        "include /etc/ld.so.conf.d/*.conf\n".into(),
    )
    .unwrap();
    assert!(s.save_configuration(&d, Scope::Assignment).is_err());
    assert_eq!(s.assignment.state, before);
    let raw = s.to_json(&d).unwrap();
    let restored = Session::from_json(&raw, &d).unwrap();
    assert_eq!(restored, s);
    assert_eq!(
        restored.assignment.configuration_draft,
        "include /etc/ld.so.conf.d/*.conf\n"
    );
}
#[test]
fn worked_example_progress_and_selected_reset_are_isolated_from_assignment() {
    let d = definition(false);
    let mut s = Session::new(&d).unwrap();
    let assignment = s.assignment.clone();
    save_launch(&mut s, &d, "gallery", Scope::Example, "/opt/gallery/lib");
    assert!(s.check(&d, Scope::Example).unwrap().passed());
    assert_eq!(s.assignment, assignment);
    assert!(!s.current_success(&d));
    s.set_view(&d, View::Assignment).unwrap();
    s.set_view(&d, View::Introduction).unwrap();
    s.set_view(&d, View::Example).unwrap();
    let example = s.example.clone();
    save_launch(&mut s, &d, "cedar", Scope::Assignment, "/opt/cedar/lib");
    s.check(&d, Scope::Assignment).unwrap();
    s.reset(&d, Scope::Assignment).unwrap();
    assert_eq!(s.example, example);
    assert_eq!(s.assignment, assignment);
    assert!(!s.current_success(&d));
    let raw = s.to_json(&d).unwrap();
    assert_eq!(Session::from_json(&raw, &d).unwrap(), s);
}
#[test]
fn source_binding_and_recomputed_grading_refuse_forged_catalog_or_credit() {
    let d = definition(false);
    let s = Session::new(&d).unwrap();
    let raw = s.to_json(&d).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    value["assignment"]["checked"] = true.into();
    let forged = Session::from_json(&serde_json::to_vec(&value).unwrap(), &d).unwrap();
    assert!(
        !forged.current_success(&d),
        "A checked boolean is never success truth"
    );
    let mut value: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    value["assignment"]["state"]["catalog"]["objects"][0]["identity"] = "invented".into();
    assert!(Session::from_json(&serde_json::to_vec(&value).unwrap(), &d).is_err());
    assert!(Session::from_json(&raw, &definition(true)).is_err());
}
#[test]
fn quoted_semicolon_is_data_but_unquoted_composition_is_refused_before_any_change() {
    let d = definition(false);
    let mut s = Session::new(&d).unwrap();
    let row=s.run(&d,Scope::Assignment,"env 'LD_LIBRARY_PATH=/opt/cedar/lib;/opt/cedar-fallback/lib' ldd '/usr/bin/cedar-report'").unwrap();
    assert_eq!(row.outcome, Outcome::Supported);
    assert!(row.output.contains("/opt/cedar/lib/libreport.so.1"));
    for input in [
        "env LD_LIBRARY_PATH=/opt/cedar/lib;/opt/cedar-fallback/lib ldd /usr/bin/cedar-report",
        "ldconfig; ldd /usr/bin/cedar-report",
        "ldd /usr/bin/cedar-report | cat",
        "ldconfig > /tmp/cache",
        "ldd $(whoami)",
        "ldd $HOME/report",
        "ldconfig && ldconfig -p",
        "ldd /usr/bin/*",
        "ldd `uname`",
    ] {
        let before = s.assignment.state.clone();
        let recipes = s.assignment.recipes.clone();
        let row = s.run(&d, Scope::Assignment, input).unwrap();
        assert_eq!(row.outcome, Outcome::Unsupported, "{input}");
        assert_eq!(s.assignment.state, before);
        assert_eq!(s.assignment.recipes, recipes);
    }
}
#[test]
fn syntax_unsupported_and_missing_metadata_have_distinct_useful_feedback() {
    let d = definition(false);
    let mut s = Session::new(&d).unwrap();
    let bad = s.run(&d, Scope::Assignment, "ldd 'unfinished").unwrap();
    assert_eq!(bad.outcome, Outcome::Invalid);
    assert!(bad.output.contains("matching closing quote"));
    assert_eq!(
        s.run(&d, Scope::Assignment, "ldd --version")
            .unwrap()
            .outcome,
        Outcome::Unsupported
    );
    assert_eq!(
        s.run(&d, Scope::Assignment, "ldd /usr/bin/unknown")
            .unwrap()
            .outcome,
        Outcome::NotFound
    );
    let missing = s
        .run(&d, Scope::Assignment, "ldd /usr/bin/cedar-report")
        .unwrap();
    assert_eq!(missing.outcome, Outcome::Supported);
    assert!(missing.output.contains("not found"));
    assert!(!s.check(&d, Scope::Assignment).unwrap().passed());
}
#[test]
fn transient_workaround_does_not_satisfy_clean_cache_or_clean_recipe_policy() {
    let d = definition(true);
    let mut s = Session::new(&d).unwrap();
    let row = s
        .run(
            &d,
            Scope::Assignment,
            "env LD_LIBRARY_PATH=/opt/harbor/lib ldd /usr/bin/harbor-report",
        )
        .unwrap();
    assert_eq!(row.outcome, Outcome::Supported);
    assert!(!s.check(&d, Scope::Assignment).unwrap().passed());
    solve_cache(&mut s, &d, "harbor", Scope::Assignment, None);
    save_launch(&mut s, &d, "harbor", Scope::Assignment, "/opt/harbor/lib");
    let report = s.check(&d, Scope::Assignment).unwrap();
    assert!(!report.passed());
    assert!(
        report
            .checks
            .iter()
            .any(|c| c.label == "Clean launch settings" && !c.passed)
    );
    save_launch(&mut s, &d, "harbor", Scope::Assignment, "");
    assert!(s.check(&d, Scope::Assignment).unwrap().passed());
}
#[test]
fn command_and_saved_data_limits_are_bounded_without_overwriting_previous_work() {
    let d = definition(false);
    let mut s = Session::new(&d).unwrap();
    let before = s.clone();
    assert!(
        s.run(&d, Scope::Assignment, &"x".repeat(MAX_COMMAND + 1))
            .is_err()
    );
    assert_eq!(s, before);
    assert!(
        s.edit_configuration(&d, Scope::Assignment, "x".repeat(core::MAX_CONFIG + 1))
            .is_err()
    );
    assert_eq!(s, before);
    assert!(Session::from_json(&vec![b' '; MAX_SESSION + 1], &d).is_err());
    for _ in 0..30 {
        s.run(&d, Scope::Assignment, "unsupported").unwrap();
    }
    assert_eq!(s.assignment.transcript.len(), MAX_ROWS);
    assert!(s.to_json(&d).unwrap().len() <= MAX_SESSION);
    let mut value = serde_json::to_value(&s).unwrap();
    let row = value["assignment"]["transcript"][0].clone();
    value["assignment"]["transcript"] = serde_json::Value::Array(vec![row; MAX_ROWS + 1]);
    assert!(Session::from_json(&serde_json::to_vec(&value).unwrap(), &d).is_err());
}

#[test]
fn global_cache_workaround_does_not_satisfy_a_scoped_temporary_search_assignment() {
    let d = definition(false);
    let mut s = Session::new(&d).unwrap();
    solve_cache(&mut s, &d, "cedar", Scope::Assignment, None);
    assert!(
        s.inspect_recipe(&d, Scope::Assignment, "/usr/bin/cedar-report")
            .unwrap()
            .complete()
    );
    assert!(
        s.assignment.recipes[0].environment.is_none(),
        "This is the global workaround, not a scoped recipe"
    );
    let report = s.check(&d, Scope::Assignment).unwrap();
    assert!(!report.passed());
    let protected = report
        .checks
        .iter()
        .find(|c| c.label == "Preserve system search settings")
        .unwrap();
    assert!(!protected.passed);
    assert!(
        protected
            .feedback
            .contains("configuration changed, cache changed")
    );
}
#[test]
fn unrelated_saved_recipe_cannot_change_even_when_its_clean_resolution_still_passes() {
    let d = definition(false);
    let mut s = Session::new(&d).unwrap();
    save_launch(&mut s, &d, "cedar", Scope::Assignment, "/opt/cedar/lib");
    s.edit_launch(
        &d,
        Scope::Assignment,
        "/usr/bin/cedar-viewer",
        "/opt/cedar/lib".into(),
    )
    .unwrap();
    s.save_launch(&d, Scope::Assignment, "/usr/bin/cedar-viewer")
        .unwrap();
    assert!(
        s.inspect_recipe(&d, Scope::Assignment, "/usr/bin/cedar-viewer")
            .unwrap()
            .complete()
    );
    let report = s.check(&d, Scope::Assignment).unwrap();
    assert!(!report.passed());
    let protected = report
        .checks
        .iter()
        .find(|c| c.label == "Preserve /usr/bin/cedar-viewer")
        .unwrap();
    assert!(!protected.passed);
    assert!(protected.feedback.contains("clean selections preserved"));
    assert!(protected.feedback.contains("saved recipe changed"));
}
#[test]
fn non_shell_control_characters_do_not_become_argument_separators() {
    for command in [
        "ldd\u{b}/usr/bin/cedar-report",
        "ldd\u{c}/usr/bin/cedar-report",
    ] {
        assert_eq!(parse(command).unwrap_err().outcome, Outcome::Unsupported);
    }
    assert!(parse("ldd\t/usr/bin/cedar-report").is_ok());
}

#[test]
fn exact_objective_plans_are_bound_even_when_initial_catalogs_are_identical() {
    let original = definition(false);
    let mut s = Session::new(&original).unwrap();
    save_launch(
        &mut s,
        &original,
        "cedar",
        Scope::Assignment,
        "/opt/cedar-fallback/lib",
    );
    assert!(s.check(&original, Scope::Assignment).unwrap().passed());
    let bytes = s.to_json(&original).unwrap();
    let mut changed = original.clone();
    changed.plan.launches[0].selections[0].allowed_identities = vec!["site".into()];
    changed.validate().unwrap();
    assert!(
        Session::from_json(&bytes, &changed).is_err(),
        "A new objective cannot silently inherit old checked credit"
    );
    assert_eq!(
        Session::from_json(&bytes, &original).unwrap(),
        s,
        "Old source and exact original work remain recoverable"
    );
}

fn temporary_cache_definition() -> Definition {
    let mut d = definition(true);
    d.initial_recipes = vec![
        LaunchRecipe {
            program: "/usr/bin/harbor-report".into(),
            environment: Some("/opt/harbor-fallback/lib".into()),
        },
        LaunchRecipe {
            program: "/usr/bin/harbor-worker".into(),
            environment: Some("/opt/harbor/lib".into()),
        },
    ];
    d.lesson.initial_recipes = vec![LaunchRecipe {
        program: "/usr/bin/workshop-report".into(),
        environment: Some("/opt/workshop/lib".into()),
    }];
    d.lesson.worked.push(Step { title:"Remove the temporary simulated override".into(),
        explanation:"Clear the worked application's saved launch setting and apply it; now its recipe uses the repaired clean cache.".into(),
        action:WorkedAction::LaunchRecipe { program:"/usr/bin/workshop-report".into(),environment:None } });
    d
}
#[test]
fn source_declared_temporary_recipes_are_real_initial_work_and_not_dirty_drafts() {
    let d = temporary_cache_definition();
    d.validate().unwrap();
    d.verify_worked().unwrap();
    let s = Session::new(&d).unwrap();
    assert!(!s.assignment.pending());
    assert!(!s.example.pending());
    assert_eq!(
        s.assignment.launch_drafts[0].value,
        "/opt/harbor-fallback/lib"
    );
    assert_eq!(
        s.assignment.launch_drafts[1].value, "",
        "Roster order includes the unrelated clean viewer"
    );
    assert_eq!(s.assignment.launch_drafts[2].value, "/opt/harbor/lib");
    assert!(
        s.inspect_recipe(&d, Scope::Assignment, "/usr/bin/harbor-worker")
            .unwrap()
            .complete()
    );
    assert!(
        !s.assignment
            .state
            .resolve("/usr/bin/harbor-worker", core::Environment::Clean)
            .unwrap()
            .complete()
    );
    assert!(
        s.inspect_recipe(&d, Scope::Example, "/usr/bin/workshop-report")
            .unwrap()
            .complete()
    );
    assert!(!s.current_success(&d));
}
#[test]
fn repaired_cache_still_requires_clearing_each_real_temporary_recipe() {
    let d = temporary_cache_definition();
    let mut s = Session::new(&d).unwrap();
    solve_cache(&mut s, &d, "harbor", Scope::Assignment, None);
    let report = s.check(&d, Scope::Assignment).unwrap();
    assert!(!report.passed());
    assert!(
        report
            .checks
            .iter()
            .filter(|c| c.label.contains("clean launch"))
            .all(|c| c.passed)
    );
    assert!(
        report
            .checks
            .iter()
            .any(|c| c.label == "Clean launch settings" && !c.passed)
    );
    save_launch(&mut s, &d, "harbor", Scope::Assignment, "");
    assert!(!s.check(&d, Scope::Assignment).unwrap().passed());
    s.edit_launch(&d, Scope::Assignment, "/usr/bin/harbor-worker", "".into())
        .unwrap();
    s.save_launch(&d, Scope::Assignment, "/usr/bin/harbor-worker")
        .unwrap();
    assert!(s.check(&d, Scope::Assignment).unwrap().passed());
}
#[test]
fn source_recipes_survive_resume_and_selected_reset_restores_the_supplied_workarounds() {
    let d = temporary_cache_definition();
    let mut s = Session::new(&d).unwrap();
    let original = s.clone();
    s.edit_launch(
        &d,
        Scope::Assignment,
        "/usr/bin/harbor-report",
        "bad raw draft\0".into(),
    )
    .unwrap();
    let bytes = s.to_json(&d).unwrap();
    let mut resumed = Session::from_json(&bytes, &d).unwrap();
    assert_eq!(resumed, s);
    let example = resumed.example.clone();
    resumed.reset(&d, Scope::Assignment).unwrap();
    assert_eq!(resumed.assignment, original.assignment);
    assert_eq!(resumed.example, example);
    assert_eq!(resumed.initial_recipes, d.initial_recipes);
    assert!(!resumed.current_success(&d));
}
#[test]
fn initial_recipe_source_pins_refuse_forgery_and_different_declarations() {
    let d = temporary_cache_definition();
    let s = Session::new(&d).unwrap();
    let raw = s.to_json(&d).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    value["initial_recipes"][0]["environment"] = "/opt/harbor/lib".into();
    assert!(Session::from_json(&serde_json::to_vec(&value).unwrap(), &d).is_err());
    let mut changed = d.clone();
    changed.initial_recipes[0].environment = None;
    changed.validate().unwrap();
    assert!(Session::from_json(&raw, &changed).is_err());
    assert_eq!(Session::from_json(&raw, &d).unwrap(), s);
}
#[test]
fn initial_recipe_keys_values_duplicates_and_contradictory_preservation_are_validated() {
    let d = temporary_cache_definition();
    let mut unknown = d.clone();
    unknown.initial_recipes[0].program = "/usr/bin/unknown".into();
    assert!(unknown.validate().is_err());
    let mut duplicate = d.clone();
    duplicate
        .initial_recipes
        .push(duplicate.initial_recipes[0].clone());
    assert!(duplicate.validate().is_err());
    let mut missing = d.clone();
    missing.initial_recipes[0].environment = Some("/home/personal/lib".into());
    assert!(missing.validate().is_err());
    let mut huge = d.clone();
    huge.initial_recipes[0].environment = Some("x".repeat(core::MAX_PATH_LIST + 1));
    assert!(huge.validate().is_err());
    let mut conflict = d;
    conflict.initial_recipes.push(LaunchRecipe {
        program: "/usr/bin/harbor-viewer".into(),
        environment: Some("/opt/harbor/lib".into()),
    });
    assert!(
        conflict
            .validate()
            .unwrap_err()
            .contains("preserve an initially nonempty protected recipe")
    );
}
#[test]
fn protected_recipe_preservation_uses_the_actual_declared_baseline_not_assumed_none() {
    let mut d = definition(false);
    d.initial_recipes = vec![LaunchRecipe {
        program: "/usr/bin/cedar-viewer".into(),
        environment: Some("/opt/cedar/lib".into()),
    }];
    d.validate().unwrap();
    let mut s = Session::new(&d).unwrap();
    save_launch(&mut s, &d, "cedar", Scope::Assignment, "/opt/cedar/lib");
    assert!(s.check(&d, Scope::Assignment).unwrap().passed());
    s.edit_launch(&d, Scope::Assignment, "/usr/bin/cedar-viewer", "".into())
        .unwrap();
    s.save_launch(&d, Scope::Assignment, "/usr/bin/cedar-viewer")
        .unwrap();
    let report = s.check(&d, Scope::Assignment).unwrap();
    assert!(!report.passed());
    assert!(
        report
            .checks
            .iter()
            .any(|c| c.label == "Preserve /usr/bin/cedar-viewer" && !c.passed)
    );
}
#[test]
fn absent_initial_recipe_fields_default_to_clean_without_inventing_work() {
    let d = definition(false);
    let mut value = serde_json::to_value(&d).unwrap();
    value.as_object_mut().unwrap().remove("initial_recipes");
    value["lesson"]
        .as_object_mut()
        .unwrap()
        .remove("initial_recipes");
    let decoded = Definition::from_json(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(decoded.initial_recipes.is_empty());
    assert!(decoded.lesson.initial_recipes.is_empty());
    let session = Session::new(&decoded).unwrap();
    assert!(
        session
            .assignment
            .recipes
            .iter()
            .all(|r| r.environment.is_none())
    );
    assert!(!session.assignment.checked);
}

#[test]
fn harmless_rebuild_of_unchanged_entries_allows_fresh_outcome_check() {
    let mut d = definition(false);
    d.initial.rebuild_cache().unwrap();
    d.lesson.initial.rebuild_cache().unwrap();
    let mut s = Session::new(&d).unwrap();
    save_launch(&mut s, &d, "cedar", Scope::Assignment, "/opt/cedar/lib");
    assert!(s.check(&d, Scope::Assignment).unwrap().passed());
    let before = s.assignment.state.cache.clone().unwrap();
    let row = s.run(&d, Scope::Assignment, "ldconfig").unwrap();
    assert_eq!(row.outcome, Outcome::Supported);
    assert!(
        !s.assignment.checked,
        "A rebuild requires a fresh explicit Check"
    );
    assert!(!s.current_success(&d));
    let after = s.assignment.state.cache.as_ref().unwrap();
    assert_eq!(after.generation, before.generation + 1);
    assert_eq!(after.source_configuration, before.source_configuration);
    assert_eq!(after.directories, before.directories);
    assert_eq!(after.entries, before.entries);
    s.bound(&d).unwrap();
    assert!(s.check(&d, Scope::Assignment).unwrap().passed());
    assert!(s.current_success(&d));
}
#[test]
fn restored_configuration_with_a_different_cache_cannot_bypass_scoped_repair() {
    let mut d = definition(false);
    d.initial.rebuild_cache().unwrap();
    let mut s = Session::new(&d).unwrap();
    s.edit_configuration(&d, Scope::Assignment, "/opt/cedar/lib".into())
        .unwrap();
    s.save_configuration(&d, Scope::Assignment).unwrap();
    s.rebuild_cache(&d, Scope::Assignment).unwrap();
    s.edit_configuration(&d, Scope::Assignment, d.initial.configuration.clone())
        .unwrap();
    s.save_configuration(&d, Scope::Assignment).unwrap();
    assert_eq!(s.assignment.state.configuration, d.initial.configuration);
    assert_ne!(
        s.assignment.state.cache.as_ref().unwrap().entries,
        d.initial.cache.as_ref().unwrap().entries
    );
    let report = s.check(&d, Scope::Assignment).unwrap();
    assert!(
        report
            .checks
            .iter()
            .find(|c| c.label.contains("cedar-report"))
            .unwrap()
            .passed,
        "A global workaround can resolve the target without the required scoped setting"
    );
    assert!(
        !report
            .checks
            .iter()
            .find(|c| c.label == "Preserve system search settings")
            .unwrap()
            .passed
    );
    assert!(!report.passed());
    assert!(!s.current_success(&d));
}

#[test]
fn cache_comment_snapshot_does_not_reject_restored_configuration_and_identical_selections() {
    let mut d = definition(false);
    d.initial.rebuild_cache().unwrap();
    let mut s = Session::new(&d).unwrap();
    save_launch(&mut s, &d, "cedar", Scope::Assignment, "/opt/cedar/lib");
    assert!(s.check(&d, Scope::Assignment).unwrap().passed());
    s.edit_configuration(
        &d,
        Scope::Assignment,
        format!("{}\n# explanatory note\n", d.initial.configuration),
    )
    .unwrap();
    s.save_configuration(&d, Scope::Assignment).unwrap();
    s.rebuild_cache(&d, Scope::Assignment).unwrap();
    assert!(
        !s.check(&d, Scope::Assignment).unwrap().passed(),
        "The applied configuration still differs"
    );
    s.edit_configuration(&d, Scope::Assignment, d.initial.configuration.clone())
        .unwrap();
    s.save_configuration(&d, Scope::Assignment).unwrap();
    let actual = s.assignment.state.cache.as_ref().unwrap();
    let expected = d.initial.cache.as_ref().unwrap();
    assert_ne!(actual.source_configuration, expected.source_configuration);
    assert_eq!(actual.directories, expected.directories);
    assert_eq!(actual.entries, expected.entries);
    s.bound(&d).unwrap();
    assert!(s.check(&d, Scope::Assignment).unwrap().passed());
    assert!(s.current_success(&d));
}

#[test]
fn authored_tasks_and_examples_must_start_with_meaningful_unfinished_work() {
    for example in [false, true] {
        let mut d = definition(false);
        let (site, recipes) = if example {
            ("gallery", &mut d.lesson.initial_recipes)
        } else {
            ("cedar", &mut d.initial_recipes)
        };
        recipes.push(LaunchRecipe {
            program: format!("/usr/bin/{site}-report"),
            environment: Some(format!("/opt/{site}/lib")),
        });
        let error = d.validate().unwrap_err();
        assert!(error.contains("meaningful unfinished"), "{error}");
    }
}
#[test]
fn example_cannot_reuse_the_assignment_inventory_as_a_disguised_solution() {
    let mut d = definition(false);
    d.lesson.initial = d.initial.clone();
    d.lesson.plan = d.plan.clone();
    assert!(
        d.validate()
            .unwrap_err()
            .contains("different supplied data")
    );
}
