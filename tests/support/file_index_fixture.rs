//! Unregistered synthetic source fixture. Construction uses only in-process core; never host tools.
use dlms_learning_lab::script_teaching::Concept;
use dlms_learning_lab::{
    engine::Node,
    file_index as core,
    file_index_workspace::{
        Definition, ExplainedStep, Fixture, Scope, Session, Step, Task, Teaching,
    },
    filesystem::Tree,
};
use std::collections::BTreeMap;
fn node(directory: bool, mode: u16, owner: &str) -> Node {
    Node {
        directory,
        mode,
        owner: owner.into(),
        group: if owner == "root" { "root" } else { "staff" }.into(),
    }
}
fn file(tree: &mut Tree, p: &str, text: &str, mode: u16) {
    tree.nodes.insert(p.into(), node(false, mode, "alex"));
    tree.files.insert(p.into(), text.into());
}
fn config(root: &str) -> String {
    format!("PRUNEPATHS = \"{root}/archive\"\n")
}
pub fn cmd(input: String) -> Step {
    Step::Command { input }
}
fn task(id: &str, role: &str, root: &str, work: &str, indexes: &str, example: bool) -> Task {
    let cfg = format!("{work}/index.conf");
    let database = format!("{indexes}/names.db");
    let handover = format!("{work}/support.txt");
    let context = core::Context {
        version: 1,
        source_id: format!("{id}-{role}"),
        root: root.into(),
        database: database.clone(),
        config: cfg.clone(),
    };
    let mut nodes = BTreeMap::from([("/".into(), node(true, 0o755, "root"))]);
    for p in [
        work.to_string(),
        indexes.to_string(),
        root.to_string(),
        format!("{root}/live"),
        format!("{root}/archive"),
    ] {
        nodes.insert(p, node(true, 0o755, "alex"));
    }
    let mut old = Tree {
        nodes,
        files: BTreeMap::new(),
        users: BTreeMap::from([
            ("root".into(), vec!["root".into()]),
            ("alex".into(), vec!["staff".into()]),
        ]),
        user: "alex".into(),
        home: work.into(),
        cwd: work.into(),
        previous: None,
        umask: 0o022,
        clock: 0,
        modified: BTreeMap::new(),
        command_location: None,
    };
    let current = if example {
        "Gallery current: route amber.\n"
    } else {
        "Catalog current: route cedar.\n"
    };
    file(&mut old, &cfg, &config(root), 0o640);
    file(
        &mut old,
        &format!("{root}/live/retired.txt"),
        "retired old receipt\n",
        0o640,
    );
    file(
        &mut old,
        &format!("{root}/archive/current.txt"),
        "archive route obsolete\n",
        0o640,
    );
    file(&mut old, &format!("{root}/seed.txt"), current, 0o640);
    file(
        &mut old,
        &format!("{work}/private-note.txt"),
        "Preserve this unrelated private note.\n",
        0o600,
    );
    file(&mut old, &handover, "wrong archived support text\n", 0o640);
    let stale = core::rebuild(&context, Some(&context), &old, &old).unwrap();
    let mut initial = old.clone();
    initial.nodes.remove(&format!("{root}/live/retired.txt"));
    initial.files.remove(&format!("{root}/live/retired.txt"));
    file(
        &mut initial,
        &format!("{root}/live/current.txt"),
        current,
        0o640,
    );
    initial
        .files
        .insert(cfg.clone(), format!("PRUNEPATHS = \"{root}/live\"\n"));
    assert!(initial.valid());
    let mut expected = initial.clone();
    expected.files.insert(cfg.clone(), config(root));
    expected.files.insert(handover.clone(), current.into());
    Task {
        initial: Fixture {
            context,
            tree: initial,
            index: Some(stale),
        },
        expected_tree: expected,
        handover: handover.clone(),
        extra_paths: if example {
            vec![
                format!("{root}/live/current.txt"),
                format!("{root}/live/experiment.txt"),
            ]
        } else {
            vec![]
        },
        reference: vec![
            Step::SaveConfig { text: config(root) },
            cmd(format!(
                "updatedb -l 0 -o {database} -U {root} --config-file {cfg}"
            )),
            cmd(format!("cp {root}/live/current.txt {handover}")),
        ],
    }
}
fn explain(title: &str, explanation: &str, step: Step) -> ExplainedStep {
    ExplainedStep {
        title: title.into(),
        explanation: explanation.into(),
        step,
    }
}
pub fn definition() -> Definition {
    let id = "synthetic-private-index";
    let assignment = task(id, "assignment", "/catalog", "/work", "/indexes", false);
    let example = task(
        id,
        "example",
        "/gallery",
        "/practice",
        "/practice-indexes",
        true,
    );
    let worked = example
        .reference
        .iter()
        .enumerate()
        .map(|(i, s)| {
            explain(
                &format!("Worked action{}", i + 1),
                "Synthetic author action; actual native controls must remain explicit.",
                s.clone(),
            )
        })
        .collect();
    let try_steps = vec![
        explain(
            "Remove a supplied practice leaf",
            "Remove the current leaf after building; saved names remain stale.",
            cmd("rm /gallery/live/current.txt".into()),
        ),
        explain(
            "Add an experiment",
            "Copy into the declared experiment slot; the old snapshot does not automatically acquire it.",
            cmd("cp /gallery/seed.txt /gallery/live/experiment.txt".into()),
        ),
    ];
    let repair=vec![explain("Restore current content","Copy the protected seed back to the supplied current path.",cmd("cp /gallery/seed.txt /gallery/live/current.txt".into())),explain("Retire experiment","Remove only the declared experiment leaf.",cmd("rm /gallery/live/experiment.txt".into())),explain("Rebuild explicitly","Refresh from saved configuration; this is not automatic.",cmd("updatedb -l 0 -o /practice-indexes/names.db -U /gallery --config-file /practice/index.conf".into()))];
    Definition{id:id.into(),version:1,assignment,example,teaching:Teaching{prerequisite_note:"Synthetic prerequisite context: file paths, copying and live versus saved names.".into(),concepts:vec![Concept{title:"Saved names".into(),text:"A typed private index is a snapshot; it does not contain document contents or update itself.".into()}],description:"Synthetic isolated gallery work; no authored curriculum or runtime acceptance.".into(),worked,try_steps,repair}}
}
pub fn run(session: &mut Session, d: &Definition, scope: Scope, input: &str) {
    session.set_command(d, scope, input.into()).unwrap();
    session.run(d, scope).unwrap();
}
pub fn solve(session: &mut Session, d: &Definition, scope: Scope) {
    for step in &d.task(scope).reference {
        match step {
            Step::SaveConfig { text } => {
                session.set_config_draft(d, scope, text.clone()).unwrap();
                session.save_config(d, scope).unwrap();
            }
            Step::Command { input } => run(session, d, scope, input),
        }
    }
}
