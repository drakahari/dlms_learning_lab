use dlms_learning_lab::automation_workshop::{
    site_connectivity::Outage, site_connectivity_task::Task,
};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
struct Cases {
    example: Task,
    assignment: Task,
}

fn cases() -> BTreeMap<String, Cases> {
    serde_json::from_str(include_str!("fixtures/site_connectivity_tasks_v1.json")).unwrap()
}

fn score(
    task: &Task,
    edit: &dlms_learning_lab::automation_workshop::site_connectivity_task::Edit,
) -> Vec<bool> {
    task.outcomes(edit)
        .unwrap()
        .into_iter()
        .map(|row| row.passed)
        .collect()
}

#[test]
fn valid_source_has_real_unfinished_examples_and_assignments() {
    for cases in cases().into_values() {
        for task in [&cases.example, &cases.assignment] {
            task.validate().unwrap();
            let rows = score(task, &task.initial_edit());
            assert!(rows.contains(&true));
            assert!(rows.contains(&false));
        }
    }
}

#[test]
fn two_connections_over_one_carrier_do_not_make_an_independent_path() {
    let source = cases();
    let task = &source["ccna-diverse-branch-paths"].example;
    let mut work = task.initial_edit();
    let before = task.observe(&work, 0).unwrap().unwrap();
    assert_eq!(before.nodes, [0, 1, 2, 4, 5]);
    assert!(task.observe(&work, 1).unwrap().is_none());
    assert!(task.observe(&work, 2).unwrap().is_some());
    work.set_link(task, 5, false).unwrap();
    work.set_link(task, 6, true).unwrap();
    assert!(score(task, &work).into_iter().all(|ok| ok));
    let path = task.observe(&work, 1).unwrap().unwrap();
    assert_eq!(path.nodes, [0, 1, 3, 4, 5]);
    assert!(path.links.contains(&6));
    assert_eq!(
        task.observe(&work, 4).unwrap().unwrap().nodes,
        [5, 4, 3, 1, 0]
    );
    work.set_link(task, 8, true).unwrap();
    let result = score(task, &work);
    assert!(!result[0]);
    assert!(!result[4]);
}

#[test]
fn different_supplied_wan_choices_pass_without_matching_reference_ids() {
    let source = cases();
    let task = &source["ccna-diverse-branch-paths"].assignment;
    for left in [4, 5] {
        for right in [6, 7] {
            let mut work = task.initial_edit();
            work.active = task.fixed;
            work.set_link(task, left, true).unwrap();
            work.set_link(task, right, true).unwrap();
            assert!(score(task, &work).into_iter().all(|ok| ok));
        }
    }
}

#[test]
fn both_location_and_outage_matter_and_a_printer_is_not_transit() {
    let source = cases();
    let task = &source["ccna-local-and-hosted-connectivity"].example;
    let mut work = task.initial_edit();
    assert_eq!(
        task.observe(&work, 0).unwrap(),
        task.observe(&work, 2).unwrap()
    );
    assert!(task.observe(&work, 3).unwrap().is_none());
    work.set_link(task, 7, true).unwrap();
    assert!(score(task, &work).into_iter().all(|ok| ok));
    work.set_link(task, 6, false).unwrap();
    assert!(!score(task, &work)[5]);
    assert!(task.observe(&work, 0).unwrap().is_some());
    let mut world = task.topology.clone();
    world.links[4].ends = [0, 2];
    world.links[5].ends = [2, 1];
    let active = (work.active | (1 << 4) | (1 << 5)) & !(1 << 0);
    assert!(world.path(active, 0, 2, Outage::None).unwrap().is_some());
    assert!(world.path(active, 0, 6, Outage::None).unwrap().is_none());
}

#[test]
fn both_local_paths_pass_and_redundancy_cannot_hide_an_excess_option() {
    let source = cases();
    let task = &source["ccna-local-and-hosted-connectivity"].assignment;
    for printer in [4, 5] {
        let mut work = task.initial_edit();
        work.set_link(task, 8, false).unwrap();
        work.set_link(task, printer, true).unwrap();
        work.set_link(task, 7, true).unwrap();
        assert!(score(task, &work).into_iter().all(|ok| ok));
        let other_printer = if printer == 4 { 5 } else { 4 };
        work.set_link(task, other_printer, true).unwrap();
        let result = score(task, &work);
        assert!(!result[0]);
        assert!(result[1..].iter().all(|ok| *ok));
    }
}

#[test]
fn malformed_saved_edits_cannot_unlock_fixed_links_or_observers() {
    let source = cases();
    let task = &source["ccna-diverse-branch-paths"].assignment;
    let mut work = task.initial_edit();
    assert!(work.set_link(task, 0, false).is_err());
    assert!(work.set_link(task, usize::MAX, true).is_err());
    assert!(task.observe(&work, usize::MAX).is_err());
    work.active |= 1 << 31;
    assert!(task.outcomes(&work).is_err());
    assert!(
        task.topology
            .path(task.initial, 0, 5, Outage::CircuitGroup { id: 99 })
            .is_err()
    );
}
