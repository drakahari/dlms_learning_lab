use dlms_learning_lab::{
    shell::{
        self, EnvironmentVariable,
        startup::{File, Files, Invocation},
    },
    startup_workspace::{Case, Definition, Task},
};
use std::collections::BTreeMap;
pub fn task(practice: bool) -> Task {
    let id = if practice {
        "startup-example"
    } else {
        "startup-assignment"
    };
    let good = "APP_NAME=\"$PROJECT-app\"\nexport APP_NAME\n";
    let starter = Files(BTreeMap::from([
        (
            "~/.bash_profile".into(),
            File {
                text: "APP_NAME=legacy\nexport APP_NAME\nLOCAL_NOTE=only-here\n".into(),
            },
        ),
        ("~/.bash_login".into(), File { text: good.into() }),
        ("~/.profile".into(), File { text: good.into() }),
    ]));
    let projects = if practice {
        ["linden", "fern room", "granite", "willow"]
    } else {
        ["harbor", "river team", "niche", "elm"]
    };
    let mut cases = vec![];
    for (n, project) in projects.into_iter().enumerate() {
        let site = format!("site-{n}");
        let files = BTreeMap::from([("/work/keep.txt".into(), format!("Retain sample {n}.\n"))]);
        let input = shell::Case {
            name: format!("{id}-input-{n}"),
            args: vec![],
            stdin: String::new(),
            env: BTreeMap::from([
                ("PROJECT".into(), project.into()),
                ("SITE".into(), site.clone()),
            ]),
            files: files.clone(),
            stdout: String::new(),
            stderr: String::new(),
            status: 0,
            after: files.clone(),
        };
        let child = shell::Case {
            name: format!("{id}-child-{n}"),
            args: vec![],
            stdin: String::new(),
            env: BTreeMap::new(),
            files: files.clone(),
            stdout: format!("{project}-app|{site}\n"),
            stderr: String::new(),
            status: 0,
            after: files,
        };
        let parent = BTreeMap::from([
            (
                "APP_NAME".into(),
                EnvironmentVariable {
                    value: Some(format!("{project}-app")),
                    exported: true,
                },
            ),
            (
                "BASE".into(),
                EnvironmentVariable {
                    value: Some(site),
                    exported: true,
                },
            ),
            (
                "LOCAL_NOTE".into(),
                EnvironmentVariable {
                    value: if n < 2 {
                        Some("only-here".into())
                    } else {
                        None
                    },
                    exported: false,
                },
            ),
        ]);
        let present = match n {
            0 | 1 => vec!["~/.bash_profile", "~/.bash_login", "~/.profile"],
            2 => vec!["~/.bash_login", "~/.profile"],
            _ => vec!["~/.profile"],
        }
        .into_iter()
        .map(str::to_owned)
        .collect();
        cases.push(Case {
            id: format!("{id}-case-{n}"),
            label: format!("Supplied login sample {}", n + 1),
            invocation: Invocation::InteractiveLogin { noprofile: false },
            present,
            system_profile: Some("BASE=\"$SITE\"\nexport BASE\n".into()),
            input,
            parent,
            child,
        });
    }
    Task{id:id.into(),instructions:"Repair application settings for the supplied login sessions without changing system defaults or leaking the local note. Preserve filenames, quiet startup and the sample files.".into(),starter,child_source:"printf '%s|%s\\n' \"$APP_NAME\" \"$BASE\"".into(),cases}
}
pub fn solved(task: &Task) -> Files {
    let mut files = task.starter.clone();
    files.0.get_mut("~/.bash_profile").unwrap().text =
        "APP_NAME=\"$PROJECT-app\"\nexport APP_NAME\nLOCAL_NOTE=only-here\n".into();
    files
}
pub fn definition() -> Definition {
    let initial = task(false);
    let example = task(true);
    let example_worked = solved(&example);
    Definition {
        initial,
        example,
        example_worked,
    }
}
