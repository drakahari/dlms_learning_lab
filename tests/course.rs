use dlms_learning_lab::{
    catalog::{self, Filter},
    course, engine, pack,
    progress::{Attempt, Progress},
};
#[test]
fn every_authored_lab_belongs_to_one_valid_unit_and_wrong_indexes_fail() {
    let labs = pack::bundled().unwrap();
    let units = course::bundled(&labs).unwrap();
    assert_eq!(
        units.iter().map(|u| u.labs.len()).sum::<usize>(),
        labs.len()
    );
    let mut data = serde_json::json!({"version":1,"units":units});
    data["units"][0]["labs"][0] = serde_json::json!("unknown-lab");
    assert!(course::load(&data.to_string(), &labs).is_err());
    let mut data = serde_json::json!({"version":1,"units":course::bundled(&labs).unwrap()});
    data["units"][0]["prerequisites"] = serde_json::json!(["ipv4-foundations"]);
    assert!(course::load(&data.to_string(), &labs).is_err());
    let mut data = serde_json::json!({"version":1,"units":course::bundled(&labs).unwrap()});
    data["units"][0]["track"] = serde_json::json!("LPIC-2");
    assert!(course::load(&data.to_string(), &labs).is_err());
}
#[test]
fn course_continuation_skips_completed_work_and_never_jumps_tracks() {
    let labs = pack::bundled().unwrap();
    let units = course::bundled(&labs).unwrap();
    let unit = &units[0];
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    assert_eq!(course::next(unit, &labs[0].id, &labs, &p), Some(1));
    for s in labs.iter().take(3) {
        let mut a = Attempt::new(s, 1);
        for c in &s.solution {
            engine::execute(&mut a.state, c).unwrap();
        }
        p.attempts.insert(s.id.clone(), a);
    }
    assert_eq!(course::next(unit, &labs[2].id, &labs, &p), None);
    p.attempts.remove(&labs[0].id);
    assert_eq!(course::next(unit, &labs[2].id, &labs, &p), Some(0));
}
#[test]
fn split_packs_validate_global_dependencies_without_relaxing_size_or_cycle_checks() {
    let labs = pack::bundled().unwrap();
    let mut a = labs[0].clone();
    let mut b = labs[1].clone();
    a.prerequisites = vec![b.id.clone()];
    b.prerequisites.clear();
    let text_a = serde_json::to_string(&pack::Pack {
        schema_version: 1,
        scenarios: vec![a.clone()],
    })
    .unwrap();
    let text_b = serde_json::to_string(&pack::Pack {
        schema_version: 1,
        scenarios: vec![b.clone()],
    })
    .unwrap();
    assert!(pack::load(&text_a).is_err());
    assert!(pack::load_catalog(&[&text_a, &text_b]).is_ok());
    b.prerequisites = vec![a.id];
    let text_b = serde_json::to_string(&pack::Pack {
        schema_version: 1,
        scenarios: vec![b],
    })
    .unwrap();
    assert!(pack::load_catalog(&[&text_a, &text_b]).is_err());
    assert!(pack::load_catalog(&[&" ".repeat(2_000_001)]).is_err());
}
#[test]
fn synthetic_large_catalog_still_filters_units_without_becoming_curriculum() {
    let labs = pack::bundled().unwrap();
    let items = catalog::synthetic_catalog(&labs, 900);
    let filter = Filter {
        unit: "shell-foundations".into(),
        ..Default::default()
    };
    let found: Vec<_> = items
        .iter()
        .filter(|s| catalog::matches(s, &filter, false, false, false))
        .collect();
    assert!(!found.is_empty());
    assert!(
        found
            .iter()
            .all(|s| s.id.starts_with("synthetic-") && s.track == "Bash/Shell")
    );
}

#[test]
fn synthetic_metadata_capacity_accepts256_units_and_refuses257_without_relaxing_other_bounds() {
    // Metadata-only fixture: these cloned states are never registered or counted
    // as authored learning scenarios, and no simulation is run here.
    let base = pack::bundled().unwrap().remove(0);
    fn data(base: &pack::Scenario, count: usize) -> (Vec<pack::Scenario>, serde_json::Value) {
        let mut labs = Vec::new();
        let mut units = Vec::new();
        for n in 0..count {
            let mut s = base.clone();
            s.id = format!("synthetic-capacity-{n:03}");
            s.unit = format!("synthetic-unit-{n:03}");
            s.prerequisites.clear();
            units.push(course::Unit {
                id: s.unit.clone(),
                track: s.track.clone(),
                title: "Synthetic metadata boundary".into(),
                summary: "Not authored curriculum; parser capacity only.".into(),
                prerequisites: if n == 0 {
                    vec![]
                } else {
                    vec![format!("synthetic-unit-{:03}", n - 1)]
                },
                labs: vec![s.id.clone()],
            });
            labs.push(s);
        }
        (labs, serde_json::json!({"version":1,"units":units}))
    }
    assert_eq!(course::MAX_UNITS, 256);
    let (labs, index) = data(&base, 256);
    assert!(index.to_string().len() < 256_000);
    assert_eq!(course::load(&index.to_string(), &labs).unwrap().len(), 256);
    let (labs, too_many) = data(&base, 257);
    assert_eq!(
        course::load(&too_many.to_string(), &labs).unwrap_err(),
        "Unsupported unit index"
    );
    let (labs, mut index) = data(&base, 2);
    index["units"][0]["prerequisites"] = serde_json::json!(["synthetic-unit-001"]);
    assert!(
        course::load(&index.to_string(), &labs)
            .unwrap_err()
            .contains("Cyclic")
    );
    let (labs, mut index) = data(&base, 1);
    index["units"][0]["summary"] = serde_json::json!("x".repeat(1001));
    assert!(course::load(&index.to_string(), &labs).is_err());
    let (labs, mut index) = data(&base, 1);
    index["units"][0]["labs"] = serde_json::json!(vec!["synthetic-capacity-000"; 201]);
    assert!(course::load(&index.to_string(), &labs).is_err());
    assert!(
        course::load(&" ".repeat(256_001), &[])
            .unwrap_err()
            .contains("size limit")
    );
}
