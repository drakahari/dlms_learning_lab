use dlms_learning_lab::kernel_modules::*;
use std::collections::BTreeMap;
pub fn catalogue() -> Catalogue {
    fn module(n: &str, deps: &[&str]) -> Module {
        Module {
            path: format!("/lib/modules/6.8.0-lab/kernel/drivers/{n}.ko"),
            description: format!("Supplied {n} teaching module"),
            kernel_release: "6.8.0-lab".into(),
            size: 16384,
            dependencies: deps.iter().map(|s| s.to_string()).collect(),
            parameters: BTreeMap::new(),
            external_users: 0,
        }
    }
    let mut modules = BTreeMap::from([
        ("bus_core".into(), module("bus_core", &[])),
        ("sensor_bus".into(), module("sensor_bus", &["bus_core"])),
        ("lab_sensor".into(), module("lab_sensor", &["sensor_bus"])),
        ("audit_link".into(), module("audit_link", &["bus_core"])),
        ("old_sensor".into(), module("old_sensor", &[])),
    ]);
    modules.get_mut("lab_sensor").unwrap().parameters.insert(
        "queues".into(),
        Parameter {
            description: "Number of supplied queues".into(),
            minimum: 1,
            maximum: 8,
            default: 1,
        },
    );
    let old = modules.get_mut("old_sensor").unwrap();
    old.kernel_release = "6.7.0-lab".into();
    old.path = "/lib/modules/6.7.0-lab/kernel/drivers/old_sensor.ko".into();
    Catalogue {
        kernel_release: "6.8.0-lab".into(),
        modules,
        devices: vec![Device {
            slot: "00:05.0".into(),
            description: "Teaching sensor controller".into(),
            driver: "lab_sensor".into(),
        }],
    }
}
pub fn runtime(c: &Catalogue) -> Runtime {
    Runtime {
        index: c
            .modules
            .iter()
            .filter(|(_, m)| m.kernel_release == c.kernel_release)
            .map(|(n, _)| n.clone())
            .collect(),
        ..Runtime::default()
    }
}
pub fn ok(r: &mut Runtime, c: &Catalogue, cmd: &str) -> String {
    let out = r.execute(c, cmd).unwrap();
    assert_eq!(out.status, 0, "{cmd}: {}", out.stderr);
    out.stdout
}
