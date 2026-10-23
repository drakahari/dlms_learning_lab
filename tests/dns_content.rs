use dlms_learning_lab::{
    dns::{Data, Server},
    dns_teaching::{self, Learning},
    dns_workspace::Draft,
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
fn labs() -> Vec<pack::Scenario> {
    pack::bundled()
        .unwrap()
        .into_iter()
        .filter(|s| s.unit == "dns-forward")
        .collect()
}
fn server(state: &State) -> &Server {
    let State::AuthoritativeDns { server } = state else {
        panic!()
    };
    server
}
fn solved(lab: &pack::Scenario) -> Server {
    let mut state = lab.initial.clone();
    for action in &lab.solution {
        engine::execute(&mut state, action).unwrap();
    }
    assert!(
        engine::solved(&state, &lab.goal),
        "{} {:?}",
        lab.id,
        engine::checks(&state, &lab.goal)
    );
    server(&state).clone()
}
fn publish(s: &mut Server, secondary: bool) {
    s.configured.serial = s.primary.serial.wrapping_add(1);
    s.reload_primary().unwrap();
    if secondary {
        s.update_secondary().unwrap();
    }
}
#[test]
fn six_distinct_assignments_and_their_separate_taught_examples_are_reachable_without_assignment_credit()
 {
    let labs = labs();
    assert_eq!(labs.len(), 6);
    let catalog = pack::bundled().unwrap();
    let units = dlms_learning_lab::course::bundled(&catalog).unwrap();
    let unit = units.iter().find(|u| u.id == "dns-forward").unwrap();
    assert_eq!(unit.labs.len(), 6);
    for (i, lab) in labs.iter().enumerate() {
        assert!(!engine::solved(&lab.initial, &lab.goal));
        let mut a = Attempt::new(lab, 1);
        workspace::prepare(&mut a);
        dns_teaching::prepare(&mut a, lab.dns_teaching.as_ref().unwrap(), true);
        let lesson = lab.dns_teaching.as_ref().unwrap();
        assert_ne!(
            server(&lab.initial).configured.origin,
            lesson.fixture.configured.origin
        );
        let assignment = serde_json::to_vec(&a.state).unwrap();
        let mut l = Learning::new(lesson);
        l.fill_example(lesson).unwrap();
        assert!(!l.checked);
        assert_eq!(l.server, lesson.fixture);
        l.draft.save(&mut l.server).unwrap();
        l.server.reload_primary().unwrap();
        if i >= 4 {
            l.server.update_secondary().unwrap();
        }
        l.checked = true;
        assert!(
            l.checks(lesson).iter().all(|c| c.0),
            "{} {:?}",
            lab.id,
            l.checks(lesson)
        );
        assert_eq!(serde_json::to_vec(&a.state).unwrap(), assignment);
        assert!(!progress::current_solved(&a, lab));
        let s = solved(lab);
        let d = Draft::new(&s);
        assert!(d.matches(&s));
        assert!(d.valid());
        assert!(
            lesson
                .concepts
                .iter()
                .all(|c| !c.text.contains("Read the contract"))
        );
    }
}
#[test]
fn canonical_alternatives_and_a_different_newer_serial_pass_every_assignment() {
    for (i, lab) in labs().iter().enumerate() {
        let mut s = solved(lab);
        s.configured.records.reverse();
        s.configured.records.push(s.configured.records[0].clone());
        for r in &mut s.configured.records {
            r.owner = r.owner.to_ascii_uppercase();
            r.ttl = 600;
            match &mut r.data {
                Data::Cname { target } | Data::Ns { target } => {
                    *target = target.to_ascii_uppercase()
                }
                Data::Mx { exchange, .. } => *exchange = exchange.to_ascii_uppercase(),
                _ => {}
            }
        }
        if i == 1 {
            for r in &mut s.configured.records {
                if r.owner.eq_ignore_ascii_case("portal.studio.test.") {
                    r.data = Data::Cname {
                        target: "APP.STUDIO.TEST.".into(),
                    };
                }
                if r.owner.eq_ignore_ascii_case("status.studio.test.") {
                    r.data = Data::Cname {
                        target: "PORTAL.STUDIO.TEST.".into(),
                    };
                }
            }
        }
        s.configured.serial = s.primary.serial.wrapping_add(17);
        s.reload_primary().unwrap();
        if i >= 4 {
            s.update_secondary().unwrap();
        }
        assert!(
            engine::solved(&State::AuthoritativeDns { server: s }, &lab.goal),
            "{}",
            lab.id
        );
    }
}
#[test]
fn plausible_wrong_repairs_and_partial_publication_fail_for_each_meaningful_task() {
    for (i, lab) in labs().iter().enumerate() {
        let mut state = lab.initial.clone();
        engine::execute(&mut state, &lab.solution[0]).unwrap();
        assert!(!engine::solved(&state, &lab.goal), "saved-only {}", lab.id);
        if i >= 4 {
            engine::execute(&mut state, "reload-primary").unwrap();
            assert!(!engine::solved(&state, &lab.goal));
        }
        let mut s = solved(lab);
        match i {
            0 => s.configured.records.retain(|r| {
                !(r.owner == "files.workshop.test." && matches!(r.data, Data::Aaaa { .. }))
            }),
            1 => {
                for r in &mut s.configured.records {
                    if r.owner == "portal.studio.test." {
                        r.data = Data::A {
                            address: "192.0.2.60".parse().unwrap(),
                        };
                    }
                }
            }
            2 => {
                for r in &mut s.configured.records {
                    if let Data::Mx { preference, .. } = &mut r.data {
                        *preference = 30;
                    }
                }
            }
            3 => s.configured.records.retain(|r| {
                !(r.owner == "web.gateway.test."
                    && r.data
                        == Data::A {
                            address: "192.0.2.11".parse().unwrap(),
                        })
            }),
            4 => {
                for r in &mut s.configured.records {
                    if r.owner == "web.branch.test." {
                        r.data = Data::A {
                            address: "203.0.113.8".parse().unwrap(),
                        };
                    }
                }
            }
            5 => s
                .configured
                .records
                .retain(|r| r.owner != "backup.handover.test."),
            _ => unreachable!(),
        };
        publish(&mut s, i >= 4);
        assert!(
            !engine::solved(&State::AuthoritativeDns { server: s }, &lab.goal),
            "wrong repair {}",
            lab.id
        );
    }
}
#[test]
fn deleting_peers_or_nameserver_data_never_satisfies_a_handover_and_alias_loops_fail_again() {
    for (i, lab) in labs().iter().enumerate() {
        let good = solved(lab);
        let peer = match i {
            0 => "db.workshop.test.",
            1 => "legacy.studio.test.",
            2 => "web.mailroom.test.",
            3 => "api.gateway.test.",
            4 => "api.branch.test.",
            5 => "archive.handover.test.",
            _ => unreachable!(),
        };
        for owner in [peer, good.configured.origin.as_str()] {
            let mut s = good.clone();
            s.configured.records.retain(|r| r.owner != owner);
            s.configured.serial += 1;
            let reload = s.reload_primary();
            if reload.is_ok() && i >= 4 {
                s.update_secondary().unwrap();
            }
            assert!(!engine::solved(
                &State::AuthoritativeDns { server: s },
                &lab.goal
            ));
        }
    }
    let lab = &labs()[1];
    let mut s = solved(lab);
    for r in &mut s.configured.records {
        if r.owner == "status.studio.test." {
            r.data = Data::Cname {
                target: "status.studio.test.".into(),
            };
        }
    }
    publish(&mut s, false);
    assert!(!engine::solved(
        &State::AuthoritativeDns { server: s },
        &lab.goal
    ));
}
#[test]
fn stale_secondary_equal_or_older_serial_cannot_earn_current_completion() {
    let lab = &labs()[4];
    let mut s = server(&lab.initial).clone();
    assert_eq!(
        s.update_secondary().unwrap(),
        dlms_learning_lab::dns::Transfer::UnchangedEqual
    );
    assert!(!engine::solved(
        &State::AuthoritativeDns { server: s.clone() },
        &lab.goal
    ));
    s.configured.serial = s.primary.serial - 1;
    s.reload_primary().unwrap();
    assert_eq!(
        s.update_secondary().unwrap(),
        dlms_learning_lab::dns::Transfer::UnchangedOlder
    );
    assert!(!engine::solved(
        &State::AuthoritativeDns { server: s.clone() },
        &lab.goal
    ));
    s.configured.serial = s.secondary.serial.wrapping_add(42);
    s.reload_primary().unwrap();
    s.update_secondary().unwrap();
    assert!(engine::solved(
        &State::AuthoritativeDns { server: s },
        &lab.goal
    ));
}
#[test]
fn all_new_drafts_and_examples_resume_reset_without_rewriting_prior_sixty_four_attempts() {
    let all = pack::bundled().unwrap();
    let previous = all
        .iter()
        .filter(|s| s.unit != "dns-forward")
        .collect::<Vec<_>>();
    assert_eq!(previous.len(), 64);
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in previous {
        p.attempts.insert(s.id.clone(), Attempt::new(s, 1));
    }
    let prior = serde_json::to_value(&p).unwrap();
    for lab in labs() {
        let mut a = Attempt::new(&lab, 2);
        workspace::prepare(&mut a);
        dns_teaching::prepare(&mut a, lab.dns_teaching.as_ref().unwrap(), true);
        a.state = State::AuthoritativeDns {
            server: solved(&lab),
        };
        a.workspace.as_mut().unwrap().dns_draft = Some(Draft::new(server(&a.state)));
        assert!(progress::current_solved(&a, &lab));
        a.workspace
            .as_mut()
            .unwrap()
            .dns_draft
            .as_mut()
            .unwrap()
            .text
            .push_str("\nunfinished record");
        a.workspace
            .as_mut()
            .unwrap()
            .dns_learning
            .as_mut()
            .unwrap()
            .draft
            .text
            .push_str("\nmy example draft");
        assert!(!progress::current_solved(&a, &lab));
        p.attempts.insert(lab.id.clone(), a);
    }
    let bytes = serde_json::to_vec(&p).unwrap();
    let mut restored = progress::decode(&bytes, &all).unwrap();
    assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
    for id in prior["attempts"].as_object().unwrap().keys() {
        assert_eq!(
            serde_json::to_value(&restored).unwrap()["attempts"][id],
            prior["attempts"][id]
        );
    }
    let lab = &labs()[0];
    restored
        .attempts
        .insert(lab.id.clone(), Attempt::new(lab, 2));
    let other = &labs()[1];
    assert!(
        restored.attempts[&other.id]
            .workspace
            .as_ref()
            .unwrap()
            .dns_draft
            .as_ref()
            .unwrap()
            .text
            .contains("unfinished record")
    );
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &restored).unwrap();
    let loaded = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_vec(&loaded).unwrap(),
        serde_json::to_vec(&restored).unwrap()
    );
}

#[test]
fn dualstack_requires_authored_address_records_at_the_requested_host_not_an_alias_substitute() {
    let lab = &labs()[0];
    let mut s = solved(lab);
    let mut helper = Vec::new();
    s.configured.records.retain(|r| {
        if r.owner == "files.workshop.test." {
            let mut r = r.clone();
            r.owner = "helper.workshop.test.".into();
            helper.push(r);
            false
        } else {
            true
        }
    });
    s.configured.records.extend(helper);
    s.configured.records.push(dlms_learning_lab::dns::Record {
        owner: "files.workshop.test.".into(),
        ttl: 300,
        data: Data::Cname {
            target: "helper.workshop.test.".into(),
        },
    });
    publish(&mut s, false);
    let engine::Goal::AuthoritativeDns { requirements } = &lab.goal else {
        panic!()
    };
    let failed = requirements
        .iter()
        .filter(|r| !r.check(&s).0)
        .collect::<Vec<_>>();
    assert_eq!(failed.len(), 2);
    assert!(failed.iter().all(|r| matches!(
        r,
        dlms_learning_lab::dns_workspace::Requirement::Records { .. }
    )));
}

#[test]
fn preserving_peer_records_cannot_be_replaced_by_aliases_with_the_same_answer() {
    for (i, lab) in labs().iter().enumerate() {
        let mut s = solved(lab);
        let peer = match i {
            0 => "db.workshop.test.",
            1 => "legacy.studio.test.",
            2 => "web.mailroom.test.",
            3 => "api.gateway.test.",
            4 => "api.branch.test.",
            5 => "archive.handover.test.",
            _ => unreachable!(),
        };
        let source = s
            .configured
            .records
            .iter()
            .find(|r| r.owner == peer && matches!(r.data, Data::A { .. }))
            .unwrap()
            .clone();
        s.configured.records.retain(|r| r.owner != peer);
        let helper = format!("helper.{}", s.configured.origin);
        let mut copied = source.clone();
        copied.owner = helper.clone();
        s.configured.records.push(copied);
        s.configured.records.push(dlms_learning_lab::dns::Record {
            owner: peer.into(),
            ttl: 300,
            data: Data::Cname { target: helper },
        });
        publish(&mut s, i >= 4);
        let engine::Goal::AuthoritativeDns { requirements } = &lab.goal else {
            panic!()
        };
        let failed = requirements
            .iter()
            .filter(|r| !r.check(&s).0)
            .collect::<Vec<_>>();
        assert_eq!(failed.len(), 1, "{}", lab.id);
        assert!(matches!(
            failed[0],
            dlms_learning_lab::dns_workspace::Requirement::Records { .. }
        ));
    }
}
