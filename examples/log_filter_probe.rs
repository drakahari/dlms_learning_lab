//! Fixed development observations only. No learner input, path or command selector.
use dlms_learning_lab::log_filter::{Disposition, Filter, inspect};
use serde_json::{Value, json};
fn main() {
    assert_eq!(std::env::args_os().count(), 1, "No selectors");
    let fixture: Value =
        serde_json::from_str(include_str!("../tests/fixtures/log-filter-reference.json")).unwrap();
    let output:Vec<_>=fixture["cases"].as_array().unwrap().iter().map(|case| {
        let filter=Filter::parse(case["filter"].as_str().unwrap()).unwrap();
        let lines:Vec<String>=case["lines"].as_array().unwrap().iter().map(|v|v.as_str().unwrap().to_owned()).collect();
        let rows=inspect(&filter,&lines).unwrap();
        json!({"id":case["id"],"rows":rows.into_iter().map(|r|json!({"line":r.line,"text":r.text,"disposition":match r.disposition {Disposition::Matched=>"matched",Disposition::Ignored=>"ignored",Disposition::Missed=>"missed"}})).collect::<Vec<_>>()})
    }).collect();
    println!("{}", serde_json::to_string(&output).unwrap());
}
