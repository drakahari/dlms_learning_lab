//! Synthetic source/persistence fixture, never registered or counted as curriculum.
use dlms_learning_lab::{
    command_location::Context,
    engine::{Goal, Node, State},
    filesystem::{
        Requirement, Tree,
        command_location_policy::{Definition, Selection},
    },
    pack, terminal_teaching,
};
use std::collections::BTreeMap;
fn tree(prefix: &str, main: &str, side: &str, identity: &str) -> Tree {
    let node = |directory, mode| Node {
        directory,
        mode,
        owner: "alex".into(),
        group: "staff".into(),
    };
    let mut nodes = BTreeMap::new();
    for path in [
        "/".into(),
        "/work".into(),
        prefix.into(),
        format!("{prefix}/preferred"),
        format!("{prefix}/fallback"),
    ] {
        nodes.insert(path, node(true, 0o755));
    }
    let mut files = BTreeMap::new();
    for (path, mode, text) in [
        (
            format!("{prefix}/preferred/{main}"),
            0o600,
            "Preferred supplied artifact\n",
        ),
        (
            format!("{prefix}/fallback/{main}"),
            0o700,
            "Fallback supplied artifact\n",
        ),
        (
            format!("{prefix}/fallback/{side}"),
            0o700,
            "Preserve sibling\n",
        ),
        ("/work/keep.txt".into(), 0o600, "Preserve unrelated work\n"),
    ] {
        nodes.insert(path.clone(), node(false, mode));
        files.insert(path, text.into());
    }
    Tree {
        nodes,
        files,
        users: BTreeMap::from([("alex".into(), vec!["staff".into()])]),
        user: "alex".into(),
        home: "/work".into(),
        cwd: "/work".into(),
        previous: None,
        umask: 0o022,
        clock: 0,
        modified: BTreeMap::new(),
        command_location: Some(Context {
            version: 1,
            source_id: identity.into(),
            path: format!("{prefix}/preferred:{prefix}/fallback"),
            names: vec![main.into(), side.into()],
        }),
    }
}
fn definition(initial: &Tree, prefix: &str, main: &str, side: &str) -> Definition {
    let mut expected = initial.clone();
    expected
        .nodes
        .get_mut(&format!("{prefix}/preferred/{main}"))
        .unwrap()
        .mode = 0o700;
    Definition {
        version: 1,
        initial: Box::new(initial.clone()),
        expected: Box::new(expected),
        selected: vec![
            Selection {
                name: main.into(),
                path: format!("{prefix}/preferred/{main}"),
            },
            Selection {
                name: side.into(),
                path: format!("{prefix}/fallback/{side}"),
            },
        ],
    }
}
pub fn value() -> serde_json::Value {
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../../scenarios/linux-files-v1.json")).unwrap();
    let mut s = raw["scenarios"][0].clone();
    let initial = tree(
        "/tools",
        "report",
        "audit",
        "command-location-fixture-assignment",
    );
    let gallery = tree(
        "/gallery",
        "demo-report",
        "demo-audit",
        "command-location-fixture-example",
    );
    let mut lesson: terminal_teaching::Lesson =
        serde_json::from_value(s["terminal_teaching"].clone()).unwrap();
    lesson.fixture = gallery.clone();
    lesson.example = vec![
        "which demo-report".into(),
        "chmod u+x /gallery/preferred/demo-report".into(),
        "type -a -p demo-report".into(),
    ];
    lesson.outcomes = vec![Requirement::CommandLocation {
        definition: Box::new(definition(
            &gallery,
            "/gallery",
            "demo-report",
            "demo-audit",
        )),
    }];
    s["id"] = "command-location-fixture".into();
    s["title"] = "Synthetic command-location source fixture".into();
    s["prerequisites"] = serde_json::json!([]);
    s["initial"] = serde_json::to_value(State::Filesystem {
        tree: initial.clone(),
    })
    .unwrap();
    s["goal"] = serde_json::to_value(Goal::Filesystem {
        requirements: vec![
            Requirement::Mode {
                path: "/tools/preferred/report".into(),
                mode: 0o700,
            },
            Requirement::CommandLocation {
                definition: Box::new(definition(&initial, "/tools", "report", "audit")),
            },
        ],
    })
    .unwrap();
    s["terminal_teaching"] = serde_json::to_value(lesson).unwrap();
    s["solution"] = serde_json::json!(["chmod u+x /tools/preferred/report"]);
    s
}
pub fn scenario() -> pack::Scenario {
    pack::load(
        &serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":[value()]}))
            .unwrap(),
    )
    .expect("synthetic native-source author gate")
    .remove(0)
}
