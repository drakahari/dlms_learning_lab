//! Fixed development observations; no caller input or host package execution.
use dlms_learning_lab::{rpm_configuration as cfg, rpm_packages as rpm};
#[path = "../tests/support/rpm_metadata_fixture.rs"]
mod f;
#[path = "../tests/support/rpm_configuration_fixture.rs"]
mod shared;
fn main() {
    assert_eq!(
        std::env::args_os().count(),
        1,
        "Fixed probe has no selectors"
    );
    let value: serde_json::Value = serde_json::from_str(include_str!(
        "../tests/fixtures/rpm-configuration-reference.json"
    ))
    .unwrap();
    let observations: Vec<_> = value["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|case| {
            let kind = match case["kind"].as_str().unwrap() {
                "config" => rpm::FileKind::Config,
                "noreplace" => rpm::FileKind::ConfigNoreplace,
                _ => panic!("Fixed kind"),
            };
            let state = shared::fixture(
                kind,
                case["current"].as_str(),
                case["incoming"].as_str().unwrap_or("route=new\n"),
            );
            let action = match case["operation"].as_str().unwrap() {
                "upgrade" => rpm::Action::UpgradeSet {
                    archives: vec!["site-v2".into()],
                },
                "erase" => rpm::Action::EraseSet {
                    packages: vec![f::identity("site")],
                },
                _ => panic!("Fixed operation"),
            };
            let result = cfg::transition(&state, &action).unwrap();
            let observe = |suffix: &str| {
                result
                    .state
                    .runtime()
                    .files
                    .get(&format!("{}{suffix}", shared::PATH))
                    .map(|file| file.contents.clone())
            };
            serde_json::json!({"id": case["id"], "file": observe(""),
                "rpmnew": observe(".rpmnew"), "rpmsave": observe(".rpmsave")})
        })
        .collect();
    println!("{}", serde_json::to_string(&observations).unwrap());
}
