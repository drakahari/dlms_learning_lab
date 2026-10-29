//! Exact private Cron pack plus its genuine normal prerequisite closure.
//! Uses existing catalog validation; no prerequisite is removed or invented.
use dlms_learning_lab::pack::{self, Scenario};
use std::collections::{BTreeMap, BTreeSet};
const IDS: [&str; 4] = [
    "lpic1-cron-weekday-report",
    "lpic1-cron-environment-output",
    "lpic1-cron-access-retirement",
    "lpic1-cron-calendar-handover",
];
pub fn lessons() -> Vec<Scenario> {
    static LABS: std::sync::OnceLock<Vec<Scenario>> = std::sync::OnceLock::new();
    LABS.get_or_init(||{
        let raw=include_str!("../fixtures/cron-lessons-v1.json");
        let value:serde_json::Value=serde_json::from_str(raw).unwrap();
        let rows=value["scenarios"].as_array().unwrap();
        assert_eq!(rows.iter().map(|s|s["id"].as_str().unwrap()).collect::<Vec<_>>(),IDS);
        let normal=pack::bundled().unwrap();
        assert_eq!(normal.len(),176);
        let by_id: BTreeMap<_,_>=normal.iter().map(|s|(s.id.as_str(),s)).collect();
        let mut pending:Vec<String>=rows.iter().flat_map(|s|s["prerequisites"].as_array().unwrap()).map(|id|id.as_str().unwrap().into()).collect();
        let mut needed=BTreeSet::new();
        while let Some(id)=pending.pop(){
            if IDS.contains(&id.as_str())||!needed.insert(id.clone()){continue}
            let source=by_id.get(id.as_str()).unwrap_or_else(||panic!("Real Cron prerequisite {id} must exist in the normal catalog"));
            pending.extend(source.prerequisites.iter().cloned());
        }
        let originals:Vec<_>=needed.iter().map(|id|*by_id.get(id.as_str()).unwrap()).collect();
        let mut packs:Vec<String>=originals.chunks(4).map(|chunk|serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":chunk})).unwrap()).collect();
        packs.push(raw.into());
        let strings:Vec<_>=packs.iter().map(String::as_str).collect();
        let all=pack::load_catalog(&strings).expect("Normal prerequisite closure plus private Cron pack validates without loader exceptions");
        for source in originals {let actual=all.iter().find(|s|s.id==source.id).unwrap();assert_eq!(serde_json::to_value(actual).unwrap(),serde_json::to_value(source).unwrap(),"Every borrowed prerequisite remains exact");}
        let labs:Vec<_>=all.into_iter().filter(|s|IDS.contains(&s.id.as_str())).collect();
        assert_eq!(labs.iter().map(|s|s.id.as_str()).collect::<Vec<_>>(),IDS);
        for s in &labs { let actual=normal.iter().find(|normal|normal.id==s.id).unwrap(); assert_eq!(serde_json::to_value(actual).unwrap(),serde_json::to_value(s).unwrap(),"Registered authored task equals original independently validated fixture"); }
        labs
    }).clone()
}
