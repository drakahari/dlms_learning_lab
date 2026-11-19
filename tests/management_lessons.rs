//! Authored remote-management tasks: outcome grading, source admission and retention.
use dlms_learning_lab::{
    course,
    engine::{self, State},
    management_workspace as w, pack,
    progress::{self, Attempt},
    workspace,
};
use std::sync::OnceLock;
const IDS: [&str; 2] = ["ccna-management-protocols", "ccna-management-source-policy"];
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted356().unwrap())
}
fn lab(id: &str) -> &'static pack::Scenario {
    catalog().iter().find(|s| s.id == id).unwrap()
}
fn attempt(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    a
}
#[test]
fn registered_unit_and_all_354_previous_scenarios_and_125_units_are_preserved() {
    let old = pack::accepted354().unwrap();
    assert_eq!(old.len(), 354);
    assert_eq!(catalog().len(), 356);
    assert_eq!(
        serde_json::to_value(&catalog()[..354]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let previous = course::accepted354(&old).unwrap();
    let units = course::accepted356(catalog()).unwrap();
    assert_eq!((previous.len(), units.len()), (125, 126));
    for u in previous {
        let retained = units.iter().find(|v| v.id == u.id).unwrap();
        assert_eq!(
            serde_json::to_value(&u).unwrap(),
            serde_json::to_value(retained).unwrap()
        );
    }
    let u = units
        .iter()
        .find(|u| u.id == "management-policy-foundations")
        .unwrap();
    assert_eq!(u.labs, IDS);
}
#[test]
fn taught_references_and_equivalent_supported_documents_pass_without_exact_matching() {
    for id in IDS {
        let s = lab(id);
        let d = w::source(s).unwrap();
        d.validate().unwrap();
        let mut a = attempt(s);
        assert!(!progress::current_solved(&a, s));
        engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
        assert!(progress::current_solved(&a, s));
        let State::ManagementPolicy { simulation } = &mut a.state else {
            panic!()
        };
        let alternate = d.initial.reference.replace(
            "\n transport input ssh\n",
            "\n! Supported comment\n transport input ssh\n",
        );
        let alternate = if id == IDS[1] {
            alternate.replace(
                "permit host 192.0.2.10\n permit host 192.0.2.11",
                "permit 192.0.2.10 0.0.0.1",
            )
        } else {
            alternate.replace("line vty 0 15\n transport input ssh\nexit","line vty 0 7\n transport input ssh\nexit\nline vty 8 15\n transport input ssh\nexit")
        };
        assert_ne!(alternate, d.initial.reference);
        simulation.assignment.edit(alternate).unwrap();
        assert!(
            simulation
                .assignment
                .check(&d.initial)
                .unwrap()
                .iter()
                .all(|r| r.passed())
        );
        assert!(progress::current_solved(&a, s));
    }
}
#[test]
fn partial_wrong_layer_overbroad_single_admin_and_uncovered_line_cannot_pass() {
    let s = lab(IDS[1]);
    let d = w::source(s).unwrap();
    let r = d.initial.reports(&d.initial.starter).unwrap();
    assert_eq!((r.iter().filter(|r| r.passed()).count(), r.len()), (4, 6));
    for doc in [
        d.initial
            .reference
            .replace("permit host 192.0.2.10", "permit 192.0.2.0 0.0.0.127"),
        d.initial.reference.replace(" permit host 192.0.2.11\n", ""),
        d.initial
            .reference
            .replace("transport input ssh", "transport input none"),
        d.initial
            .reference
            .replace("line vty 0 15", "line vty 0 14"),
    ] {
        assert_ne!(doc, d.initial.reference);
        assert!(match d.initial.reports(&doc) {
            Err(_) => true,
            Ok(r) => r.iter().any(|r| !r.passed()),
        });
    }
    let s = lab(IDS[0]);
    let d = w::source(s).unwrap();
    let doc = d
        .initial
        .reference
        .replace("line vty 0 15", "line vty 0 14")
        + "line vty 15\n transport input ssh telnet\nexit\n";
    let r = d.initial.reports(&doc).unwrap();
    let fail = r.iter().find(|r| !r.passed()).unwrap();
    assert_eq!((fail.matched_lines, fail.total_lines), (15, 16));
    assert!(fail.first_failure.as_ref().unwrap().contains("15"));
}
#[test]
fn example_copy_cannot_earn_credit_and_edit_reset_restore_recompute_current_success() {
    for id in IDS {
        let s = lab(id);
        let d = w::source(s).unwrap();
        let mut a = attempt(s);
        let State::ManagementPolicy { simulation } = &mut a.state else {
            panic!()
        };
        simulation.fill_example(d).unwrap();
        simulation.practice.check(&d.example).unwrap();
        assert!(
            simulation
                .assignment
                .check(&d.initial)
                .unwrap()
                .iter()
                .any(|r| !r.passed())
        );
        simulation
            .assignment
            .edit(d.example.reference.clone())
            .unwrap();
        assert!(simulation.assignment.check(&d.initial).is_err());
        simulation
            .assignment
            .edit(d.initial.reference.clone())
            .unwrap();
        simulation.assignment.check(&d.initial).unwrap();
        let text = serde_json::to_string(&simulation).unwrap();
        let restored = w::Session::from_json(d, &text).unwrap();
        assert!(restored.checks(d).iter().all(|v| v.0));
        simulation
            .assignment
            .edit("line vty unfinished".into())
            .unwrap();
        assert!(!progress::current_solved(&a, s));
        let State::ManagementPolicy { simulation } = &mut a.state else {
            panic!()
        };
        let practice = simulation.practice.clone();
        simulation.reset_assignment(d).unwrap();
        assert_eq!(simulation.practice, practice);
        assert!(!progress::current_solved(&a, s));
    }
}

#[test]
fn both_authored_management_lessons_remain_exact_in_the_actual_current_catalog() {
    let current = pack::current().unwrap();
    let units = course::current(&current).unwrap();
    assert_eq!((current.len(), units.len()), (445, 176));
    let old_units = course::accepted356(catalog()).unwrap();
    let old_unit = old_units
        .iter()
        .find(|u| u.id == "management-policy-foundations")
        .unwrap();
    let unit = units.iter().find(|u| u.id == old_unit.id).unwrap();
    assert_eq!(
        serde_json::to_value(unit).unwrap(),
        serde_json::to_value(old_unit).unwrap()
    );
    for id in IDS {
        let now = current.iter().find(|s| s.id == id).unwrap();
        assert_eq!(
            serde_json::to_value(now).unwrap(),
            serde_json::to_value(lab(id)).unwrap()
        );
        assert_eq!(now.unit, unit.id);
        assert!(unit.labs.iter().any(|s| s == id));
    }
}
