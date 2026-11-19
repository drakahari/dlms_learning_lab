//! Fixed real-widget workflow for one expanded lesson, never learner host input.
use super::*;
pub(super) fn route(labs: &[Scenario]) -> Vec<Op> {
    let source = labs.iter().find(|s| s.id == "lpic1-host-resolver").unwrap();
    let engine::Goal::HostNetwork { definition } = &source.goal else {
        panic!()
    };
    assert!(definition.direct_dns.is_some());
    let mut ops = vec![
        Op::Open(source.id.clone()),
        Op::ControlShot("direct-dns-introduction".into(), "example".into()),
        Op::Click("example".into()),
    ];
    for (input, expected) in [
        ("dig @10.73.0.53 service.lab.test. A +short", "10.73.0.80\n"),
        ("dig +short @10.73.0.54 service.lab.test. A", "10.73.0.90\n"),
        ("host -t A inventory.lab.test. 10.73.0.53", "10.73.0.90\n"),
        ("host -t A missing.lab.test. 10.73.0.53", "Unsupported name"),
    ] {
        action(
            &mut ops,
            &work::Event::Command {
                input: input.into(),
            },
        );
        ops.push(Op::TranscriptContains(expected));
    }
    ops.extend([
        Op::Example(false),
        Op::ControlShot("direct-dns-practice-refusal".into(), "command".into()),
    ]);
    action(
        &mut ops,
        &work::Event::Resolver {
            text: "nameserver 10.73.0.53\n".into(),
        },
    );
    ops.extend([
        Op::Click("check".into()),
        Op::Example(true),
        Op::Shot("direct-dns-practice-pass".into()),
        Op::Click("assignment".into()),
        Op::ControlShot("direct-dns-assignment".into(), "command".into()),
    ]);
    for (input, expected) in [
        (
            "dig @203.0.113.54 service.lab.test. A +short",
            "203.0.113.90\n",
        ),
        (
            "host -4 -t A service.lab.test. 203.0.113.53",
            "203.0.113.80\n",
        ),
    ] {
        action(
            &mut ops,
            &work::Event::Command {
                input: input.into(),
            },
        );
        ops.push(Op::TranscriptContains(expected));
    }
    ops.extend([
        Op::Click("check".into()),
        Op::Solved(false),
        Op::Shot("direct-dns-unrepaired-client".into()),
    ]);
    for (text, pass) in [
        ("nameserver 203.0.113.54\nnameserver 203.0.113.53\n", false),
        (
            "# chosen after comparison\nnameserver 203.0.113.53\nnameserver 203.0.113.54\n",
            true,
        ),
    ] {
        action(&mut ops, &work::Event::Resolver { text: text.into() });
        ops.extend([Op::Click("check".into()), Op::Solved(pass)]);
    }
    ops.extend([Op::Save, Op::Shot("direct-dns-repaired-client".into())]);
    action(
        &mut ops,
        &work::Event::Command {
            input: "host -t A service.lab.test. 203.0.113.54".into(),
        },
    );
    ops.extend([
        Op::Solved(true),
        Op::TranscriptContains("203.0.113.90\n"),
        Op::Theme(1),
        Op::ControlShot("direct-dns-light150-command".into(), "command".into()),
        Op::Theme(2),
        Op::ControlShot("direct-dns-dark200-command".into(), "command".into()),
    ]);
    action(
        &mut ops,
        &work::Event::Resolver {
            text: "nameserver 203.0.113.54\n".into(),
        },
    );
    ops.extend([Op::Click("check".into()), Op::Solved(false), Op::Save]);
    ops
}
