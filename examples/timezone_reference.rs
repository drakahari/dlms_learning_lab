//! Fixed development observations only: no file/process/network access.
#[path = "support/whereis_digest.rs"]
mod digest;
use dlms_learning_lab::timezones::{Environment, FIRST, command::World};
use serde::Deserialize;
use serde_json::json;
const FIXTURE: &[u8] = include_bytes!("../tests/fixtures/timezone-reference-v1.json");
const INPUTS: &[(&str, &[u8])] = &[
    ("Cargo.toml", include_bytes!("../Cargo.toml")),
    ("Cargo.lock", include_bytes!("../Cargo.lock")),
    ("src/lib.rs", include_bytes!("../src/lib.rs")),
    ("src/timezones.rs", include_bytes!("../src/timezones.rs")),
    (
        "src/timezones/format.rs",
        include_bytes!("../src/timezones/format.rs"),
    ),
    (
        "src/timezones/command.rs",
        include_bytes!("../src/timezones/command.rs"),
    ),
    (
        "src/timezones/command/literal.rs",
        include_bytes!("../src/timezones/command/literal.rs"),
    ),
    (
        "examples/timezone_reference.rs",
        include_bytes!("timezone_reference.rs"),
    ),
    (
        "examples/support/whereis_digest.rs",
        include_bytes!("support/whereis_digest.rs"),
    ),
    ("tests/fixtures/timezone-reference-v1.json", FIXTURE),
];
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    configured: String,
    tz: Option<String>,
    epoch: i64,
    utc: bool,
    format: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    version: u32,
    cases: Vec<Case>,
}
fn main() {
    let fixture: Fixture = serde_json::from_slice(FIXTURE).unwrap();
    assert_eq!(fixture.version, 1);
    assert_eq!(fixture.cases.len(), 30);
    let cases=fixture.cases.iter().map(|case|{
        let mut world=World::new(case.configured.clone(),Environment{tz:case.tz.clone()},FIRST).unwrap();
        let before=world.clone();
        let flag=if case.utc {" -u"}else{""};
        let raw=format!("date --date=@{}{flag} '+{}'",case.epoch,case.format);
        let result=world.command(&raw).unwrap();
        assert_eq!(world,before,"Date display must not change the virtual clock or environment");
        json!({"id":case.id,"status":result.status,"stdout":result.stdout,"stderr":result.stderr})
    }).collect::<Vec<_>>();
    let sources = INPUTS
        .iter()
        .map(|(path, bytes)| json!({"path":path,"bytes":bytes.len(),"sha256":digest::hash(bytes)}))
        .collect::<Vec<_>>();
    println!(
        "{}",
        json!({"purpose":"timezone-capability-reference-v1","manifest_dir":env!("CARGO_MANIFEST_DIR"),"sources":sources,"cases":cases})
    );
}
