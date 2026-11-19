use dlms_learning_lab::virtual_isolation_workspace::*;
#[path = "virtual_routing_fixture.rs"]
mod routing;
#[path = "virtual_workloads_fixture.rs"]
mod workloads;
pub fn task(routing_mode: bool) -> Task {
    let exercise = if routing_mode {
        let (facts, reference) = routing::fixture();
        let mut initial = reference.clone();
        for b in &mut initial.bindings {
            b.context = "blue".into();
        }
        Exercise::Routing {
            facts,
            initial,
            reference,
            pairs: vec![
                Pair {
                    from: "a-client".into(),
                    to: "a-server".into(),
                    reachable: true,
                },
                Pair {
                    from: "b-client".into(),
                    to: "b-server".into(),
                    reachable: true,
                },
                Pair {
                    from: "a-client".into(),
                    to: "b-server".into(),
                    reachable: false,
                },
                Pair {
                    from: "b-client".into(),
                    to: "a-server".into(),
                    reachable: false,
                },
            ],
        }
    } else {
        let (facts, reference) = workloads::fixture();
        let mut initial = reference.clone();
        for p in &mut initial.placements {
            p.host = "north".into();
        }
        Exercise::Workloads {
            facts,
            initial,
            reference,
            services: vec![
                Name {
                    id: "database".into(),
                },
                Name { id: "web".into() },
            ],
        }
    };
    Task {
        id: "independent-task".into(),
        exercise,
    }
}
pub fn definition(routing: bool) -> Definition {
    let assignment = task(routing);
    let mut example = assignment.clone();
    example.id = "separate-guide".into();
    Definition {
        id: "ccna-represented-lab".into(),
        version: 1,
        assignment,
        example,
    }
}
