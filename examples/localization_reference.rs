//! Fixed new-capability observations; no host utility, script or file access.
#[path = "support/whereis_digest.rs"]
mod digest;
use dlms_learning_lab::localization::{self, Category, Environment, Locale, Variable};
use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeMap;
const FIXTURE: &[u8] = include_bytes!("../tests/fixtures/localization-reference-v1.json");
const INPUTS: &[(&str, &[u8])] = &[
    ("Cargo.toml", include_bytes!("../Cargo.toml")),
    ("Cargo.lock", include_bytes!("../Cargo.lock")),
    ("src/lib.rs", include_bytes!("../src/lib.rs")),
    (
        "src/localization.rs",
        include_bytes!("../src/localization.rs"),
    ),
    (
        "src/localization/encoding.rs",
        include_bytes!("../src/localization/encoding.rs"),
    ),
    (
        "src/cron_workspace.rs",
        include_bytes!("../src/cron_workspace.rs"),
    ),
    (
        "examples/localization_reference.rs",
        include_bytes!("localization_reference.rs"),
    ),
    (
        "examples/support/whereis_digest.rs",
        include_bytes!("support/whereis_digest.rs"),
    ),
    ("tests/fixtures/localization-reference-v1.json", FIXTURE),
];
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Conversion {
    id: String,
    input: Vec<u8>,
    from: localization::encoding::Encoding,
    to: localization::encoding::Encoding,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Query {
    id: String,
    env: BTreeMap<String, String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    version: u32,
    conversions: Vec<Conversion>,
    queries: Vec<Query>,
}
fn main() {
    let fixture: Fixture = serde_json::from_slice(FIXTURE).unwrap();
    assert_eq!(fixture.version, 1);
    assert_eq!(fixture.conversions.len(), 16);
    assert_eq!(fixture.queries.len(), 6);
    let conversions = fixture.conversions.iter().map(|case| {
        let result = localization::encoding::convert(&case.input, case.from, case.to).unwrap();
        json!({"id":case.id,"bytes":result.bytes,"failed":result.failure.is_some(),"failure":result.failure.map(|f|json!({"position":f.position,"kind":format!("{:?}",f.kind)}))})
    }).collect::<Vec<_>>();
    let queries = fixture.queries.iter().map(|case| {
        let mut env=Environment::default();
        for (name,value) in &case.env {env.set(Variable::parse(name).unwrap(),if value.is_empty(){None}else{Some(Locale::parse(value).unwrap())});}
        json!({"id":case.id,"charmap":env.effective(Category::Ctype).charmap(),"decimal_point":env.effective(Category::Numeric).decimal().to_string()})
    }).collect::<Vec<_>>();
    let sources = INPUTS
        .iter()
        .map(|(path, bytes)| json!({"path":path,"bytes":bytes.len(),"sha256":digest::hash(bytes)}))
        .collect::<Vec<_>>();
    println!(
        "{}",
        json!({"purpose":"localization-capability-reference-v1","manifest_dir":env!("CARGO_MANIFEST_DIR"),"sources":sources,"conversions":conversions,"queries":queries})
    );
}
