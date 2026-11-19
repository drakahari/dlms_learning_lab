//! Normal two-process restoration of source-bound server and observer work.
use super::*;
use crate::ui::smoke::service_exposure::{Op, translate};
use dlms_learning_lab::{service_exposure_source as binding, service_exposure_workspace as work};
const ID: &str = "lpic2-exposure-client-policy";
pub(super) const CAPTURES: [&str; 8] = [
    "reopen-service-exposure-writer-example-command",
    "reopen-service-exposure-writer-assignment-command",
    "reopen-service-exposure-reader-assignment-command",
    "reopen-service-exposure-reader-assignment-observer",
    "reopen-service-exposure-reader-example-command",
    "reopen-service-exposure-reader-example-observer",
    "reopen-service-exposure-repaired-assignment",
    "reopen-service-exposure-repaired-example",
];
fn pending(source: &pack::Scenario, practice: bool) -> Vec<work::Action> {
    let d = binding::source(source).unwrap();
    let t = if practice { &d.example } else { &d.assignment };
    let mut actions: Vec<work::Action> = if practice {
        d.worked.clone()
    } else {
        source
            .solution
            .iter()
            .map(|r| {
                let e: serde_json::Value =
                    serde_json::from_str(r.strip_prefix(binding::PREFIX).unwrap()).unwrap();
                serde_json::from_value(e["action"].clone()).unwrap()
            })
            .collect()
    };
    let prefix = if practice {
        "10.50.0.0/24"
    } else {
        "10.20.0.0/24"
    };
    // New GUI actions record version three. Existing Scan/ProbeV2 histories
    // retain their original interpreters and stored observations.
    for action in &mut actions {
        if let work::Action::Event {
            event: work::Event::Scan { input },
        } = action
        {
            *action = work::Action::Event {
                event: work::Event::ObserverV3 {
                    input: input.clone(),
                },
            };
        }
    }
    let scan = format!("nc -n -v -z {} 22", t.initial.server);
    actions.extend([
        work::Action::Event {
            event: work::Event::Admin {
                input: format!("iptables -I INPUT -s {prefix} -p tcp --dport 443 -j DROP"),
            },
        },
        work::Action::Event {
            event: work::Event::Select { viewpoint: 1 },
        },
    ]);
    if practice {
        actions.extend([
            work::Action::Event {
                event: work::Event::ObserverV3 {
                    input: format!("telnet -c {} 443", t.initial.server),
                },
            },
            work::Action::Event {
                event: work::Event::TelnetEscapeV3,
            },
        ]);
    } else {
        actions.push(work::Action::Event {
            event: work::Event::ObserverV3 {
                input: scan.clone(),
            },
        });
    }
    actions.extend([
        work::Action::AdminInput {
            text: "iptables -D INPUT 1".into(),
        },
        work::Action::ScanInput {
            text: if practice { "qui".into() } else { scan },
        },
    ]);
    actions
}
fn verify_partial(a: &LabApp) {
    let source = a.scenarios.iter().find(|s| s.id == ID).unwrap();
    let attempt = &a.progress.attempts[ID];
    binding::context(attempt, source).unwrap();
    let d = binding::source(source).unwrap();
    let engine::State::ServiceExposure { simulation } = &attempt.state else {
        panic!()
    };
    for (scope, t, practice) in [
        (&simulation.assignment, &d.assignment, false),
        (&simulation.example, &d.example, true),
    ] {
        let mut expected = work::Scope::new(t).unwrap();
        for action in pending(source, practice) {
            expected.apply(t, &action).unwrap();
        }
        assert_eq!(
            scope, &expected,
            "Restore actual history, sources, observations, selected observer and queued drafts before repair"
        );
        assert!(!scope.checked);
        assert_eq!(scope.viewpoint, 1);
        assert!(scope.last_scan.is_some());
        assert_eq!(scope.last_scan.as_ref().unwrap().rows.len(), 1);
        if practice {
            assert_eq!(
                scope.telnet,
                Some(dlms_learning_lab::telnet_client::Mode::Command { port: 443 })
            );
            assert_eq!(scope.scan_status, None);
            assert_eq!(scope.scan_input, "qui");
            assert!(matches!(
                scope.history.last(),
                Some(work::Event::TelnetEscapeV3)
            ));
            assert_eq!(
                scope.last_scan.as_ref().unwrap().rows[0].state,
                dlms_learning_lab::service_exposure::PortState::Open
            );
        } else {
            assert!(scope.telnet.is_none());
            assert_eq!(scope.scan_status, Some(1));
            assert!(matches!(
                scope.history.last(),
                Some(work::Event::ObserverV3 { .. })
            ));
            assert_eq!(
                scope.last_scan.as_ref().unwrap().rows[0].state,
                dlms_learning_lab::service_exposure::PortState::Filtered
            );
        }
        assert!(!t.checks(&scope.world).iter().all(|c| c.passed));
        assert!(!scope.admin_input.is_empty() && !scope.scan_input.is_empty());
    }
    assert_ne!(
        simulation.assignment.scan_input,
        simulation.example.scan_input
    );
    assert!(!progress::current_solved(attempt, source));
}
fn plan(read: bool, source: &pack::Scenario) -> &'static Vec<Op> {
    static W: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    static R: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    (if read { &R } else { &W }).get_or_init(|| {
        assert_eq!(source.id, ID);
        let mut p = vec![];
        if !read {
            p.push(Op::ColdAppearance);
        }
        p.push(Op::Open(ID.into()));
        for practice in if read { [false, true] } else { [true, false] } {
            p.push(Op::Click(
                if practice { "example" } else { "assignment" }.into(),
            ));
            if read {
                p.extend([
                    Op::Shot(
                        CAPTURES[if practice { 4 } else { 2 }].into(),
                        "admin-input".into(),
                    ),
                    Op::Shot(
                        CAPTURES[if practice { 5 } else { 3 }].into(),
                        "scan-input".into(),
                    ),
                ]);
                if practice {
                    p.extend([
                        Op::Text("scan-input".into(), "quit".into()),
                        Op::Click("scan-run".into()),
                    ]);
                }
                p.push(Op::Click("admin-run".into()));
                if practice {
                    let d = binding::source(source).unwrap();
                    p.push(Op::Text(
                        "scan-input".into(),
                        format!("nc -n -v -z {} 22", d.example.initial.server),
                    ));
                }
                p.extend([
                    Op::Click("scan-run".into()),
                    Op::Click("check".into()),
                    Op::Check(true, practice),
                    Op::Shot(
                        CAPTURES[if practice { 7 } else { 6 }].into(),
                        "result".into(),
                    ),
                ]);
            } else {
                for action in pending(source, practice) {
                    translate(&mut p, &action);
                }
                p.extend([
                    Op::NotRun(practice),
                    Op::Shot(
                        CAPTURES[if practice { 0 } else { 1 }].into(),
                        "admin-input".into(),
                    ),
                ]);
            }
        }
        p.push(Op::ColdFinish);
        p
    })
}
impl LabApp {
    pub(super) fn reopen_service_exposure_tick(&mut self, ctx: &egui::Context, session: &Session) {
        assert_eq!(session.case, Case::ServiceExposure);
        assert!(!self.smoke && self._smoke_profile.is_none() && self.profile_lock.is_some());
        assert!(!self.read_only && !self.preferences_read_only);
        session.verify_profile_paths(&self.path).unwrap();
        if session.role == Role::Read {
            let key = egui::Id::new("service-exposure-normal-load-observed");
            if !ctx.data(|d| d.get_temp::<bool>(key)).unwrap_or(false) {
                let expected: Observed = serde_json::from_slice(
                    &bounded_file(&session.root.join("writer-observed.json"), JSON_CAP).unwrap(),
                )
                .unwrap();
                let (loaded, raw, prefs) = session.observed(self);
                assert_eq!(loaded, expected);
                assert_eq!(
                    raw,
                    bounded_file(&session.root.join("writer-progress.raw"), 16_000_000).unwrap()
                );
                assert_eq!(
                    prefs,
                    bounded_file(&session.root.join("writer-preferences.raw"), 4096).unwrap()
                );
                verify_partial(self);
                assert_eq!(self.scale, SCALE);
                assert_eq!(self.theme_id, THEME);
                session.write("reader-loaded.json", &loaded).unwrap();
                ctx.data_mut(|d| d.insert_temp(key, true));
            }
        }
        let source = self.scenarios.iter().find(|s| s.id == ID).unwrap().clone();
        self.service_exposure_flow(ctx, plan(session.role == Role::Read, &source));
    }
    pub(in crate::ui) fn finish_service_exposure_reopen(&mut self, ctx: &egui::Context) {
        let session = self.reopen.as_ref().unwrap().clone();
        assert_eq!(session.case, Case::ServiceExposure);
        if session.role == Role::Write {
            verify_partial(self);
        } else {
            let source = self.scenarios.iter().find(|s| s.id == ID).unwrap();
            let attempt = &self.progress.attempts[ID];
            let d = binding::source(source).unwrap();
            let engine::State::ServiceExposure { simulation } = &attempt.state else {
                panic!()
            };
            assert!(progress::current_solved(attempt, source));
            for (scope, t) in [
                (&simulation.assignment, &d.assignment),
                (&simulation.example, &d.example),
            ] {
                assert!(scope.solved(t));
                assert!(scope.admin_input.is_empty() && scope.scan_input.is_empty());
                assert!(scope.last_scan.is_some());
                assert!(scope.telnet.is_none());
                assert_eq!(scope.viewpoint, 1);
            }
        }
        let key = egui::Id::new("service-exposure-cold-finished");
        if !ctx.data(|d| d.get_temp::<bool>(key)).unwrap_or(false) {
            self.save();
            assert!(!self.dirty && self.storage_error.is_none());
            let (observed, raw, prefs) = session.observed(self);
            let names = if session.role == Role::Write {
                (
                    "writer-progress.raw",
                    "writer-preferences.raw",
                    "writer-observed.json",
                )
            } else {
                (
                    "reader-progress.raw",
                    "reader-preferences.raw",
                    "reader-repaired.json",
                )
            };
            session.write_raw(names.0, &raw, 16_000_000).unwrap();
            session.write_raw(names.1, &prefs, 4096).unwrap();
            session.write(names.2, &observed).unwrap();
            ctx.data_mut(|d| d.insert_temp(key, true));
            println!(
                "PASS native reopen {}: actual filter/listener state, selected observer, scan evidence and separate queued commands restored before native repair",
                if session.role == Role::Write {
                    "writer"
                } else {
                    "reader"
                }
            );
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}
#[test]
fn service_exposure_cold_plan_restores_real_partial_work_then_repairs_via_queued_commands() {
    let labs = pack::current().unwrap();
    let source = labs.iter().find(|s| s.id == ID).unwrap();
    let d = binding::source(source).unwrap();
    for read in [false, true] {
        let p = plan(read, source);
        assert!(matches!(p.last(), Some(Op::ColdFinish)));
        assert!(!p.iter().any(|o| matches!(o, Op::Style(..))));
    }
    for (practice, t) in [(false, &d.assignment), (true, &d.example)] {
        let mut scope = work::Scope::new(t).unwrap();
        for a in pending(source, practice) {
            scope.apply(t, &a).unwrap();
        }
        assert!(!scope.checked && !t.checks(&scope.world).iter().all(|c| c.passed));
        assert!(scope.last_scan.is_some());
        if practice {
            assert_eq!(
                scope.telnet,
                Some(dlms_learning_lab::telnet_client::Mode::Command { port: 443 })
            );
            scope
                .apply(
                    t,
                    &work::Action::Event {
                        event: work::Event::ObserverV3 {
                            input: "quit".into(),
                        },
                    },
                )
                .unwrap();
        }
        for event in [
            work::Event::Admin {
                input: scope.admin_input.clone(),
            },
            work::Event::ObserverV3 {
                input: format!("nc -n -v -z {} 22", t.initial.server),
            },
            work::Event::Check,
        ] {
            scope.apply(t, &work::Action::Event { event }).unwrap();
        }
        assert!(scope.solved(t));
    }
}
