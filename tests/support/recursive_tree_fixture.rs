//! Original controlled virtual authoring fixtures, no host operations.
use dlms_learning_lab::{
    engine::Node,
    filesystem::{Tree, recursive as core},
    recursive_tree_workspace::{Activity, Definition, Task},
};
use std::collections::BTreeMap;
fn node(directory: bool, mode: u16, owner: &str, group: &str) -> Node {
    Node {
        directory,
        mode,
        owner: owner.into(),
        group: group.into(),
    }
}
fn activity(root: &str, source: &str, destination: &str, private: bool, retire: bool) -> Activity {
    let source = format!("{root}/{source}");
    let destination = format!("{root}/{destination}");
    let owner = if retire { "alex" } else { "bea" };
    let group = if retire { "staff" } else { "team" };
    let mut nodes = BTreeMap::from([
        ("/".into(), node(true, 0o755, "root", "root")),
        ("/home".into(), node(true, 0o755, "root", "root")),
        ("/home/alex".into(), node(true, 0o700, "alex", "staff")),
        (root.into(), node(true, 0o755, "alex", "staff")),
        (source.clone(), node(true, 0o755, owner, group)),
        (format!("{source}/docs"), node(true, 0o755, owner, group)),
        (
            format!("{source}/empty-cache"),
            node(true, 0o755, owner, group),
        ),
        (
            format!("{root}/keep.txt"),
            node(false, 0o600, "alex", "staff"),
        ),
    ]);
    let mut files = BTreeMap::from([(
        format!("{root}/keep.txt"),
        "Retain this unrelated private note\n".into(),
    )]);
    for (name, text, mode) in [
        ("README.txt", "Overview\n", 0o644),
        (".index", "hidden index without LF", 0o644),
        ("docs/runbook.txt", "Runbook: café\n\n", 0o444),
    ] {
        let p = format!("{source}/{name}");
        nodes.insert(p.clone(), node(false, mode, owner, group));
        files.insert(
            p,
            format!(
                "{}: {text}",
                if root == "/work" {
                    "Independent"
                } else {
                    "Practice"
                }
            ),
        );
    }
    let mut tree = Tree {
        nodes,
        files,
        users: BTreeMap::from([
            ("root".into(), vec!["root".into()]),
            ("alex".into(), vec!["staff".into()]),
            ("bea".into(), vec!["team".into(), "staff".into()]),
        ]),
        user: "alex".into(),
        home: "/home/alex".into(),
        cwd: root.into(),
        previous: None,
        umask: 0o022,
        clock: 0,
        modified: BTreeMap::new(),
        command_location: None,
    };
    if retire {
        let sibling = format!("{source}-current");
        tree.nodes
            .insert(sibling.clone(), node(true, 0o755, "alex", "staff"));
        tree.nodes.insert(
            format!("{sibling}/live.txt"),
            node(false, 0o644, "alex", "staff"),
        );
        tree.files.insert(
            format!("{sibling}/live.txt"),
            "Live project: keep this tree\n".into(),
        );
        core::execute(&mut tree, &format!("cp -R {source} {destination}")).unwrap();
        for missing in [".index", "docs/runbook.txt"] {
            tree.execute(&format!("rm {destination}/{missing}"))
                .unwrap();
        }
    }
    assert!(tree.valid());
    Activity {
        root: root.into(),
        initial: tree,
        task: if retire {
            Task::Retire {
                source,
                destination,
            }
        } else {
            Task::Stage {
                source,
                destination,
                private_defaults: private,
            }
        },
    }
}
pub fn definitions() -> Vec<Definition> {
    vec![
 Definition{version:1,source_id:"lpic1-tree-private-stage".into(),assignment:activity("/work","incoming","private-copy",true,false),example:activity("/practice","template","working",true,false),reference:vec!["umask 077".into(),"cp -R /work/incoming /work/private-copy".into()],example_commands:vec!["find /practice/template -type f".into(),"umask 077".into(),"cp -R /practice/template /practice/working".into(),"ls -ld /practice/working".into(),"ls -l /practice/working".into()]},
 Definition{version:1,source_id:"lpic1-tree-verified-retirement".into(),assignment:activity("/work","old-project","handover",false,true),example:activity("/practice","retired-notes","ready-notes",false,true),reference:vec!["cp /work/old-project/.index /work/handover/.index".into(),"cp /work/old-project/docs/runbook.txt /work/handover/docs/runbook.txt".into(),"rm -r /work/old-project".into()],example_commands:vec!["find /practice/ready-notes -type f".into(),"find /practice/retired-notes -type f".into(),"cp /practice/retired-notes/.index /practice/ready-notes/.index".into(),"cp /practice/retired-notes/docs/runbook.txt /practice/ready-notes/docs/runbook.txt".into(),"rm -r /practice/retired-notes".into(),"find /practice/ready-notes -type f".into()]}
]
}
