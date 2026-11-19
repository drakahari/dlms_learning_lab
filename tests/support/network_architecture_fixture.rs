use dlms_learning_lab::network_architecture::*;
pub fn node(id: &str, role: Role, block: Option<&str>, ports: usize) -> Node {
    Node {
        id: id.into(),
        label: id.into(),
        role,
        block: block.map(str::to_string),
        ports: (1..=ports)
            .map(|i| Port {
                id: format!("p{i}"),
            })
            .collect(),
    }
}
pub fn link(id: &str, a: &str, ap: usize, b: &str, bp: usize) -> Connection {
    Connection {
        wire: id.into(),
        a: Some(Jack {
            node: a.into(),
            port: format!("p{ap}"),
        }),
        b: Some(Jack {
            node: b.into(),
            port: format!("p{bp}"),
        }),
    }
}
pub fn facts(
    design: Design,
    nodes: Vec<Node>,
    connections: Vec<Connection>,
) -> (Facts, Configuration) {
    let wires = connections
        .iter()
        .map(|c| Wire {
            id: c.wire.clone(),
            label: c.wire.clone(),
        })
        .collect();
    (
        Facts {
            schema_version: 1,
            design,
            nodes,
            wires,
        },
        Configuration { connections },
    )
}
pub fn two() -> (Facts, Configuration) {
    facts(
        Design::TwoTier,
        vec![
            node("north", Role::Access, Some("north"), 3),
            node("south", Role::Access, Some("south"), 3),
            node("west", Role::CollapsedCore, None, 3),
            node("east", Role::CollapsedCore, None, 3),
        ],
        vec![
            link("nw", "north", 1, "west", 1),
            link("ne", "north", 2, "east", 1),
            link("sw", "south", 1, "west", 2),
            link("se", "south", 2, "east", 2),
        ],
    )
}
pub fn three() -> (Facts, Configuration) {
    let mut nodes = vec![
        node("north", Role::Access, Some("north"), 2),
        node("south", Role::Access, Some("south"), 2),
        node("west", Role::Core, None, 4),
        node("east", Role::Core, None, 4),
    ];
    let mut links = vec![];
    for (i, (id, block, access, port)) in [
        ("dn1", "north", "north", 1),
        ("dn2", "north", "north", 2),
        ("ds1", "south", "south", 1),
        ("ds2", "south", "south", 2),
    ]
    .into_iter()
    .enumerate()
    {
        nodes.push(node(id, Role::Distribution, Some(block), 3));
        links.push(link(&format!("a-{id}"), access, port, id, 1));
        links.push(link(&format!("w-{id}"), id, 2, "west", i + 1));
        links.push(link(&format!("e-{id}"), id, 3, "east", i + 1));
    }
    facts(Design::ThreeTier, nodes, links)
}
pub fn failure(id: &str) -> Experiment {
    Experiment {
        unavailable: vec![Failure::Node { id: id.into() }],
    }
}

pub fn leaf_spine() -> (Facts, Configuration) {
    let mut nodes = vec![];
    let mut links = vec![];
    for leaf in 0..4 {
        nodes.push(node(
            &format!("leaf-{leaf}"),
            Role::Leaf,
            Some(&format!("rack-{leaf}")),
            3,
        ));
    }
    for spine in 0..3 {
        nodes.push(node(&format!("spine-{spine}"), Role::Spine, None, 4));
    }
    for leaf in 0..4 {
        for spine in 0..3 {
            links.push(link(
                &format!("l{leaf}-s{spine}"),
                &format!("leaf-{leaf}"),
                spine + 1,
                &format!("spine-{spine}"),
                leaf + 1,
            ));
        }
    }
    facts(Design::SpineLeaf, nodes, links)
}
