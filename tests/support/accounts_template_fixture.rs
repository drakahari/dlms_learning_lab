//! Synthetic native-adapter integration fixture, not an authored catalog lab.
use dlms_learning_lab::{
    accounts::{Entry, Kind},
    pack,
};
use serde_json::{Value, json};
pub const ASSIGNMENT_PATH: &str = "/etc/skel-field/.profile";
pub const EXAMPLE_PATH: &str = "/etc/skel-sample/.profile";
pub const ASSIGNMENT_TEXT: &str = "TEAM_DEFAULT=field\nexport TEAM_DEFAULT\n";
pub const EXAMPLE_TEXT: &str = "GALLERY_DEFAULT=sample\nexport GALLERY_DEFAULT\n";
fn entry(text: &str, uid: u32, gid: u32) -> Value {
    serde_json::to_value(Entry {
        uid,
        gid,
        mode: 0o640,
        data: Kind::Text {
            contents: text.into(),
        },
    })
    .unwrap()
}
fn expected(rows: &mut Value, path: &str, body: &str, home: Option<(&str, u32, u32)>) {
    for row in rows.as_array_mut().unwrap() {
        if row["kind"] == "Entry" && row["path"] == path {
            row["expected"] = entry(body, 0, 0);
        }
        if let Some((home, uid, gid)) = home
            && row["kind"] == "Entry"
            && row["path"] == home
        {
            row["expected"] = entry(body, uid, gid);
        }
        if row["kind"] == "Configuration" {
            row["expected"]["entries"][path] = entry(body, 0, 0);
            if let Some((home, uid, gid)) = home {
                row["expected"]["entries"][home] = entry(body, uid, gid);
            }
        }
    }
    rows.as_array_mut()
        .unwrap()
        .push(json!({"kind":"Entry","path":path,"expected":entry(body,0,0)}));
}
fn save(path: &str, text: &str) -> Value {
    json!({"action":"save","path":path,"contents":text,"mode":0o640})
}
pub fn value() -> Value {
    let all: Value =
        serde_json::from_str(include_str!("../../scenarios/accounts-lifecycle-v1.json")).unwrap();
    let mut s = all["scenarios"][0].clone();
    s["id"] = json!("template-workspace-fixture");
    s["title"] = json!("Synthetic native template integration fixture");
    s["initial"]["simulation"]["policy"]["editable_skeleton_leaves"] =
        json!({ASSIGNMENT_PATH:{"uid":0,"gid":0,"allow_remove":false}});
    s["goal"]["initial"] = s["initial"]["simulation"].clone();
    expected(
        &mut s["goal"]["requirements"],
        ASSIGNMENT_PATH,
        ASSIGNMENT_TEXT,
        Some(("/home/nola/.profile", 1200, 2400)),
    );
    let l = &mut s["accounts_teaching"];
    l["fixture"]["policy"]["editable_skeleton_leaves"] =
        json!({EXAMPLE_PATH:{"uid":0,"gid":0,"allow_remove":false}});
    l["template_teaching"] = json!({"assignment":[save(ASSIGNMENT_PATH,ASSIGNMENT_TEXT)],"example":[save(EXAMPLE_PATH,EXAMPLE_TEXT)],"try_steps":[save(EXAMPLE_PATH,"TRIAL_DEFAULT=temporary\n")],"repair":[save(EXAMPLE_PATH,EXAMPLE_TEXT)]});
    expected(
        &mut l["outcomes"],
        EXAMPLE_PATH,
        EXAMPLE_TEXT,
        Some(("/home/iva/.profile", 3200, 4400)),
    );
    expected(
        &mut l["try_outcomes"],
        EXAMPLE_PATH,
        "TRIAL_DEFAULT=temporary\n",
        None,
    );
    s
}
pub fn scenario() -> pack::Scenario {
    pack::load(&serde_json::to_string(&json!({"schema_version":1,"scenarios":[value()]})).unwrap())
        .expect("synthetic native reference then real account commands")
        .remove(0)
}
