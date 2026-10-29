//! Authored project behavior; these fixtures never execute a host command.
use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    ipv4_routing::Topology,
    ipv4_routing_teaching as teach,
    ipv4_routing_workspace::Draft,
    pack::{self, Pack, Scenario},
    progress::{self, Attempt, Progress},
    workspace,
};
use serde_json::{Value, json};
use std::sync::OnceLock;
const ID: &str = "ccna-private-office-plan";
const DATA: &str = include_str!("../scenarios/ipv4-private-planning-v1.json");
const REVIEW: &str = include_str!("fixtures/ipv4-private-planning-review.json");
fn review() -> Value {
    serde_json::from_str(REVIEW).unwrap()
}
fn value<T: serde::Serialize + ?Sized>(x: &T) -> Value {
    serde_json::to_value(x).unwrap()
}
fn commands(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_owned())
        .collect()
}
fn flags(v: &Value) -> Vec<bool> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_bool().unwrap())
        .collect()
}
fn catalog() -> &'static Vec<Scenario> {
    static ALL: OnceLock<Vec<Scenario>> = OnceLock::new();
    ALL.get_or_init(|| {
        let mut before = pack::bundled().unwrap();
        let authored: Pack = serde_json::from_str(DATA).unwrap();
        assert_eq!(authored.scenarios.len(), 1);
        assert_eq!(authored.scenarios[0].id, ID);
        if let Some(existing) = before.iter().find(|s| s.id == ID) {
            assert_eq!(value(existing), value(&authored.scenarios[0]));
            return before;
        }
        let original = value(&before);
        let count = before.len();
        before.extend(authored.scenarios);
        let texts = before
            .chunks(16)
            .map(|rows| {
                serde_json::to_string(&Pack {
                    schema_version: 1,
                    scenarios: rows.to_vec(),
                })
                .unwrap()
            })
            .collect::<Vec<_>>();
        let all =
            pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
        assert_eq!(all.len(), count + 1);
        assert_eq!(
            value(&all[..count]),
            original,
            "Every preexisting scenario stays exact"
        );
        all
    })
}
fn source() -> &'static Scenario {
    catalog().iter().find(|s| s.id == ID).unwrap()
}
fn topology(state: &State) -> &Topology {
    let State::Ipv4Routing { topology } = state else {
        panic!("Real IPv4 state required")
    };
    topology
}
fn fresh() -> Attempt {
    let s = source();
    let mut a = Attempt::new(s, 7);
    workspace::prepare(&mut a);
    teach::prepare(&mut a, s.ipv4_routing_teaching.as_ref().unwrap(), true);
    a
}
fn perform(a: &mut Attempt, commands: &[String]) {
    let s = source();
    for c in commands {
        engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
    }
    // Derive native editor values from the real applied model, never install a grade.
    let draft = Draft::new(topology(&a.state));
    a.workspace.as_mut().unwrap().routing_v4_draft = Some(draft);
}
fn outcomes(a: &Attempt) -> Vec<bool> {
    engine::checks(&a.state, &source().goal)
        .into_iter()
        .map(|r| r.0)
        .collect()
}
#[test]
fn one_distinct_full_project_has_valid_prerequisites_and_append_only_unit() {
    let s = source();
    let r = review();
    assert_eq!(r["authored_lab_count"], 1);
    assert_eq!(r["numeric_variants_counted_as_labs"], 0);
    assert_eq!(s.solution, commands(&r["reference"]));
    assert_eq!(s.prerequisites, ["ccna-ipv4", "ccna-vlsm", "ccna-gateway"]);
    let old: Value = serde_json::from_str(include_str!("../scenarios/units-v1.json")).unwrap();
    let mut expanded = old.clone();
    if !expanded["units"]
        .as_array()
        .unwrap()
        .iter()
        .any(|u| u["id"] == s.unit)
    {
        expanded["units"]
            .as_array_mut()
            .unwrap()
            .push(r["unit"].clone());
        assert_eq!(
            expanded["units"].as_array().unwrap()[..old["units"].as_array().unwrap().len()],
            *old["units"].as_array().unwrap()
        );
    }
    let units = course::load(&expanded.to_string(), catalog()).unwrap();
    let unit = units.iter().find(|u| u.id == s.unit).unwrap();
    assert_eq!(unit.labs, [ID]);
    assert_eq!(unit.prerequisites, ["ipv4-foundations"]);
    assert_eq!(outcomes(&fresh()), flags(&r["initial_expected_flags"]));
    assert!(!progress::current_solved(&fresh(), s));
}
#[test]
fn references_alternate_private_pools_prefixes_and_boundary_choices_pass_state_grading() {
    let s = source();
    let r = review();
    let mut sources = vec![commands(&r["reference"])];
    sources.extend(r["alternatives"].as_array().unwrap().iter().map(commands));
    sources.extend(
        r["additional_positive_boundaries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| commands(&v["commands"])),
    );
    assert_eq!(sources.len(), 5);
    for program in sources {
        let mut a = fresh();
        perform(&mut a, &program);
        assert_eq!(outcomes(&a), vec![true; 4], "{program:?}");
        assert!(progress::current_solved(&a, s));
        assert_eq!(
            a.step, 0,
            "Completion is independent of walkthrough position"
        );
        let mut reversed = fresh();
        let mut reverse = program.clone();
        reverse.reverse();
        perform(&mut reversed, &reverse);
        assert!(
            progress::current_solved(&reversed, s),
            "Both native edit orders work"
        );
    }
}
#[test]
fn wrong_and_genuinely_partial_work_fail_with_the_declared_specific_outcomes() {
    let s = source();
    let r = review();
    assert_eq!(r["wrong_partial"].as_array().unwrap().len(), 13);
    for row in r["wrong_partial"].as_array().unwrap() {
        let mut a = fresh();
        perform(&mut a, &commands(&row["commands"]));
        assert_eq!(
            outcomes(&a),
            flags(&row["expected_flags"]),
            "{}",
            row["name"]
        );
        assert!(!progress::current_solved(&a, s));
        assert!(
            engine::checks(&a.state, &s.goal)
                .iter()
                .filter(|r| !r.0)
                .all(|r| !r.1.trim().is_empty())
        );
    }
    let mut a = fresh();
    perform(&mut a, &["address Alder lan 10.20.3.1/27".into()]);
    assert!(
        engine::checks(&a.state, &s.goal)
            .iter()
            .any(|r| !r.0 && r.1.contains("Birch"))
    );
}
#[test]
fn successful_ping_does_not_override_wrong_masks_and_later_breaks_invalidate_success() {
    let s = source();
    let r = review();
    let mut a = fresh();
    perform(
        &mut a,
        &[
            "address Alder lan 10.20.3.1/27".into(),
            "address Birch lan 10.20.3.2/26".into(),
        ],
    );
    assert!(
        topology(&a.state)
            .probe("Alder", "10.20.3.2".parse().unwrap())
            .is_ok()
    );
    assert_eq!(outcomes(&a), [true, true, false, true]);
    assert!(
        engine::checks(&a.state, &s.goal)
            .iter()
            .any(|r| !r.0 && r.1.contains("matching prefixes"))
    );
    perform(&mut a, &commands(&r["reference"]));
    assert!(progress::current_solved(&a, s));
    perform(&mut a, &commands(&r["damage"]));
    assert_eq!(outcomes(&a), flags(&r["expected_damage_flags"]));
    assert!(!progress::current_solved(&a, s));
    perform(&mut a, &commands(&r["repair"]));
    assert!(progress::current_solved(&a, s));
}
#[test]
fn independent_example_fill_is_not_execution_and_change_repair_never_awards_assignment_credit() {
    let s = source();
    let lesson = s.ipv4_routing_teaching.as_ref().unwrap();
    let r = review();
    let mut a = fresh();
    let initial = value(&a.state);
    let draft = value(&a.workspace.as_ref().unwrap().routing_v4_draft);
    let l = a
        .workspace
        .as_mut()
        .unwrap()
        .routing_v4_learning
        .as_mut()
        .unwrap();
    assert!(!l.checked);
    assert!(
        l.checks(lesson).is_empty(),
        "Unchecked practice does not display a result"
    );
    assert!(
        lesson.outcomes.iter().any(|r| !r.check(&l.topology).0),
        "The underlying initial fixture still needs the taught repair"
    );
    let before = value(&l.topology);
    l.fill_example(lesson).unwrap();
    assert_eq!(value(&l.topology), before);
    l.apply().unwrap();
    assert_eq!(
        l.checks(lesson).iter().map(|r| r.0).collect::<Vec<_>>(),
        vec![true; 4]
    );
    for c in commands(&r["practice_experiment"]) {
        dlms_learning_lab::ipv4_routing_workspace::adapter(&mut l.topology, &c).unwrap();
    }
    l.draft = Draft::new(&l.topology);
    assert_eq!(
        l.checks(lesson).iter().map(|r| r.0).collect::<Vec<_>>(),
        flags(&r["practice_experiment_flags"])
    );
    for c in commands(&r["practice_repair"]) {
        dlms_learning_lab::ipv4_routing_workspace::adapter(&mut l.topology, &c).unwrap();
    }
    l.draft = Draft::new(&l.topology);
    assert_eq!(
        l.checks(lesson).iter().map(|r| r.0).collect::<Vec<_>>(),
        vec![true; 4]
    );
    assert_eq!(value(&a.state), initial);
    assert_eq!(
        value(&a.workspace.as_ref().unwrap().routing_v4_draft),
        draft
    );
    assert!(!progress::current_solved(&a, s));
}
#[test]
fn unsupported_host_prefixes_and_public_endpoint_limit_reject_without_partial_model_changes() {
    let s = source();
    let r = review();
    for cmd in r["unsupported_prefix_controls"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["command"].as_str().unwrap())
        .chain(std::iter::once(r["unsupported_control"].as_str().unwrap()))
    {
        let mut a = fresh();
        let before = value(&a);
        assert!(
            engine::execute_for_goal(&mut a.state, &s.goal, cmd).is_err(),
            "{cmd}"
        );
        assert_eq!(value(&a), before);
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn malformed_or_unreachable_packs_do_not_load_and_no_catalog_placeholder_is_added() {
    let all = catalog();
    let index = all.iter().position(|s| s.id == ID).unwrap();
    for change in 0..4 {
        let mut rows = all.clone();
        match change {
            0 => rows[index].id = "Invalid ID".into(),
            1 => rows[index].solution = vec!["address Alder lan 192.0.2.1/27".into()],
            2 => {
                rows[index]
                    .ipv4_routing_teaching
                    .as_mut()
                    .unwrap()
                    .example_solution = vec!["address Aspen lan 198.51.100.1/24".into()]
            }
            _ => {
                let Goal::Ipv4Routing { requirements } = &mut rows[index].goal else {
                    panic!()
                };
                requirements.clear();
            }
        }
        let texts = rows
            .chunks(16)
            .map(|rows| json!({"schema_version":1,"scenarios":rows}).to_string())
            .collect::<Vec<_>>();
        assert!(pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>()).is_err());
    }
}
#[test]
fn unfinished_both_scopes_drafts_and_original_progress_survive_resume_and_selected_reset() {
    let s = source();
    let mut p = Progress {
        schema_version: 1,
        clock: 20,
        ..Default::default()
    };
    for old in pack::bundled()
        .unwrap()
        .iter()
        .filter(|old| old.id != ID)
        .take(9)
    {
        let mut original = Attempt::new(old, 1);
        workspace::prepare(&mut original);
        p.attempts.insert(old.id.clone(), original);
    }
    let old = value(&p);
    let mut a = fresh();
    a.workspace
        .as_mut()
        .unwrap()
        .routing_v4_draft
        .as_mut()
        .unwrap()
        .devices
        .get_mut("Alder")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address = "10.20.3.1/31".into();
    let l = a
        .workspace
        .as_mut()
        .unwrap()
        .routing_v4_learning
        .as_mut()
        .unwrap();
    l.draft
        .devices
        .get_mut("Willow")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address = "172.22.4.34/32".into();
    p.attempts.insert(ID.into(), a);
    p.favorites.insert(ID.into());
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let mut q = progress::load(&path, catalog()).unwrap();
    assert_eq!(value(&p), value(&q));
    // Loading can produce a genuine migration receipt. Preserve the observed
    // decoded receipt across the next save/reopen and selected reset; never
    // require a legitimate new receipt to equal the original None caller.
    let receipts = value(&q.migration_receipts);
    progress::save(&path, &q).unwrap();
    let reopened = progress::load(&path, catalog()).unwrap();
    assert_eq!(value(&reopened), value(&q));
    assert_eq!(value(&reopened.migration_receipts), receipts);
    q = reopened;
    assert!(!progress::current_solved(&q.attempts[ID], s));
    for (key, v) in old["attempts"].as_object().unwrap() {
        assert_eq!(value(&q)["attempts"][key], *v);
    }
    let clock = q.clock;
    q.attempts.insert(ID.into(), fresh());
    assert_eq!(q.clock, clock);
    assert_eq!(value(&q.migration_receipts), receipts);
    assert_eq!(q.favorites, p.favorites);
    for (key, v) in old["attempts"].as_object().unwrap() {
        assert_eq!(value(&q)["attempts"][key], *v);
    }
    assert!(!progress::current_solved(&q.attempts[ID], s));
}
