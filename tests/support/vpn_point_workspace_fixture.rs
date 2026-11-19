use crate::fixture::peer;
use crate::task_fixture;
use dlms_learning_lab::vpn_point_workspace::{Action, Definition};
pub fn definition() -> Definition {
    let assignment = task_fixture::task();
    let mut example = assignment.clone();
    example.initial.supplied.peers[0].endpoint = "192.0.2.10".parse().unwrap();
    example.initial.saved[1] = example.initial.saved[1].replace("203.0.113.10", "192.0.2.10");
    Definition {
        version: 1,
        source_id: "lpic2-vpn-peer-profile".into(),
        assignment,
        example,
        worked: vec![
            Action::Edit {
                peer: 1,
                text: peer(false).replace("203.0.113.10", "192.0.2.10"),
            },
            Action::Save { peer: 1 },
            Action::Apply,
            Action::Check,
        ],
    }
}
pub fn repair() -> Vec<Action> {
    vec![
        Action::Edit {
            peer: 1,
            text: peer(false),
        },
        Action::Save { peer: 1 },
        Action::Apply,
        Action::Check,
    ]
}
