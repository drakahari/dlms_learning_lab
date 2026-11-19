#![allow(dead_code)]
#[path = "bond_policy.rs"]
mod policy_fixture;
use dlms_learning_lab::nm_workspace as w;
pub fn task(id: &str, base: u8) -> w::Task {
    let good = policy_fixture::files(base);
    let mut starter = good.clone();
    let b = starter.get_mut("bond.nmconnection").unwrap();
    *b = b.replace("autoconnect-ports=1", "autoconnect-ports=0");
    let b = starter.get_mut("backup.nmconnection").unwrap();
    *b = b.replace("controller=bond0\nport-type=bond\n", "");
    let req = vec![
        w::Requirement::CurrentUnchanged {
            interface: "reports0".into(),
        },
        w::Requirement::CurrentUnchanged {
            interface: "audit0".into(),
        },
        w::Requirement::FuturePath {
            target: policy_fixture::ip(&format!("10.{base}.0.50")),
            interface: "reports0".into(),
        },
        w::Requirement::FuturePath {
            target: policy_fixture::ip(&format!("10.{}.0.50", base + 1)),
            interface: "audit0".into(),
        },
        w::Requirement::FutureNoDefault {
            interface: "audit0".into(),
        },
    ];
    let cases = [
        ("both", "Both supplied physical members available"),
        ("primary", "Preferred member unavailable"),
        ("native", "Other member unavailable"),
    ]
    .into_iter()
    .map(|(which, label)| {
        let mut env = policy_fixture::environment(base);
        if which != "both" {
            env.vlan
                .as_mut()
                .unwrap()
                .fabric
                .hosts
                .get_mut(which)
                .unwrap()
                .up = false;
        }
        w::Case {
            current_bridge: None,
            current_controls: Some(policy_fixture::current()),
            id: format!("{which}-state"),
            label: label.into(),
            environment: env,
            current_files: good.clone(),
            initially_selected: vec![
                format!("00000000-0000-4000-8000-{base:012}"),
                format!("00000000-0000-4000-8000-{:012}", base + 1),
            ],
            requirements: req.clone(),
        }
    })
    .collect();
    w::Task{id:id.into(),instructions:"Prepare the saved bond/member dependencies so both supplied VLAN peers remain reachable at next startup with either physical member unavailable. Preserve actual current controller/membership and VLAN settings. Keep the Audit profile unchanged and do not add its default route.".into(),starter,protected_profiles:vec!["audit.nmconnection".into()],cases}
}
pub fn solved(t: &w::Task) -> w::Files {
    t.cases[0].current_files.clone()
}
pub fn partial(t: &w::Task) -> w::Files {
    let mut f = t.starter.clone();
    let s = f.get_mut("bond.nmconnection").unwrap();
    *s = s.replace("autoconnect-ports=0", "autoconnect-ports=1");
    f
}
pub fn definition() -> w::Definition {
    let example = task("bond-capability-practice", 30);
    w::Definition {
        initial: task("bond-capability-assignment", 20),
        example_worked: solved(&example),
        example,
    }
}
