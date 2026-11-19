//! Focused model checks; catalog integration and native acceptance are separate.
use dlms_learning_lab::virtual_workloads::*;
pub fn fixture() -> (Facts, Configuration) {
    let host = |id: &str, kernel| Host {
        id: id.into(),
        kernel,
        hypervisor: true,
        container_runtime: kernel == Kernel::Linux,
        memory_mib: 4096,
        cpu_units: 4,
        up: true,
    };
    let work = |id: &str, service: &str, environment| Workload {
        id: id.into(),
        service: service.into(),
        environment,
        memory_mib: 512,
        cpu_units: 1,
    };
    let facts = Facts {
        hosts: vec![
            host("north", Kernel::Linux),
            host("south", Kernel::Linux),
            host("bsd", Kernel::FreeBsd),
        ],
        workloads: vec![
            work(
                "db-one",
                "database",
                Environment::VirtualMachine {
                    kernel: Kernel::Windows,
                },
            ),
            work(
                "db-two",
                "database",
                Environment::VirtualMachine {
                    kernel: Kernel::Windows,
                },
            ),
            work("web-one", "web", Environment::LinuxContainer),
            work("web-two", "web", Environment::LinuxContainer),
        ],
    };
    let placements = facts
        .workloads
        .iter()
        .enumerate()
        .map(|(i, w)| Placement {
            workload: w.id.clone(),
            host: if i % 2 == 0 { "north" } else { "south" }.into(),
            up: true,
        })
        .collect();
    (facts, Configuration { placements })
}
