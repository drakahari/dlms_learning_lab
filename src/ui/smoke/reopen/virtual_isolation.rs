//! Fixed normal-profile cold load for BOTH new draft kinds; actual native repairs.
use super::*;
use crate::ui::smoke::virtual_isolation::{Op, repair};
use dlms_learning_lab::{virtual_isolation_source as binding, virtual_isolation_workspace as work};
const IDS: [&str; 2] = ["ccna-virtual-host-resilience", "ccna-vrf-overlap-repair"];
pub(super) const CAPTURES: [&str; 8] = [
    "reopen-virtual-writer-hosts",
    "reopen-virtual-writer-routing",
    "reopen-virtual-reader-host-assignment",
    "reopen-virtual-reader-host-example",
    "reopen-virtual-reader-routing-assignment",
    "reopen-virtual-reader-routing-example",
    "reopen-virtual-reader-host-repaired",
    "reopen-virtual-reader-routing-repaired",
];
fn expected_partial(t: &work::Task) -> work::Draft {
    let mut d = t.reference();
    match &mut d {
        work::Draft::Workloads(c) => c.placements[0].up = false,
        work::Draft::Routing(c) => {
            let context = c.bindings.last().unwrap().context.clone();
            c.bindings[1].context = context;
        }
    }
    d
}
fn plan(read: bool) -> &'static Vec<Op> {
    static W: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    static R: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    (if read { &R } else { &W }).get_or_init(|| {
        let labs = pack::load(include_str!(
            "../../../../scenarios/virtual-isolation-lessons-v1.json"
        ))
        .unwrap();
        let mut p = vec![];
        if !read {
            p.push(Op::ColdAppearance);
        }
        for (n, id) in IDS.iter().enumerate() {
            let s = labs.iter().find(|s| s.id == *id).unwrap();
            let d = binding::source(s).unwrap();
            p.push(Op::Open((*id).into()));
            if read {
                p.push(Op::Shot(
                    CAPTURES[2 + n * 2].into(),
                    match &d.assignment.exercise {
                        work::Exercise::Workloads { reference, .. } => {
                            format!("placement:{}", reference.placements[0].workload)
                        }
                        work::Exercise::Routing { reference, .. } => {
                            format!("context:{}", reference.bindings[1].interface)
                        }
                    },
                ));
                // Re-select all native reference values; up controls repair the host draft.
                repair(&mut p, &d.assignment);
                if n == 0 {
                    p.push(Op::Click("up:ledger-a".into()));
                }
                p.extend([
                    Op::Click("check".into()),
                    Op::Check(true, false),
                    Op::Shot(CAPTURES[6 + n].into(), "result-title".into()),
                    Op::Click("example".into()),
                ]);
                p.push(Op::Shot(
                    CAPTURES[3 + n * 2].into(),
                    match &d.example.exercise {
                        work::Exercise::Workloads { reference, .. } => {
                            format!("placement:{}", reference.placements[0].workload)
                        }
                        work::Exercise::Routing { reference, .. } => {
                            format!("context:{}", reference.bindings[1].interface)
                        }
                    },
                ));
                repair(&mut p, &d.example);
                if n == 0 {
                    p.push(Op::Click("up:archive-a".into()));
                }
                p.extend([Op::Click("check".into()), Op::Check(true, true)]);
            } else {
                p.push(Op::Click("example".into()));
                repair(&mut p, &d.example);
                p.extend([
                    Op::Click("check".into()),
                    Op::Check(true, true),
                    Op::Uncredited,
                ]);
                p.push(if n == 0 {
                    Op::Click("up:archive-a".into())
                } else {
                    Op::Choose("context:studio-out".into(), "birch".into())
                });
                p.push(Op::Check(false, true));
                p.push(Op::Click("assignment".into()));
                repair(&mut p, &d.assignment);
                p.extend([Op::Click("check".into()), Op::Check(true, false)]);
                p.push(if n == 0 {
                    Op::Click("up:ledger-a".into())
                } else {
                    Op::Choose("context:east-out".into(), "green".into())
                });
                p.push(Op::Check(false, false));
                p.push(Op::Shot(
                    CAPTURES[n].into(),
                    if n == 0 {
                        "placement:ledger-a"
                    } else {
                        "context:east-out"
                    }
                    .into(),
                ));
            }
        }
        p.push(Op::ColdFinish);
        p
    })
}
impl LabApp {
    pub(super) fn reopen_virtual_tick(&mut self, ctx: &egui::Context, session: &Session) {
        assert_eq!(session.case, Case::VirtualIsolation);
        assert!(!self.smoke && self._smoke_profile.is_none() && self.profile_lock.is_some());
        assert!(!self.read_only && !self.preferences_read_only);
        session.verify_profile_paths(&self.path).unwrap();
        if session.role == Role::Read {
            let key = egui::Id::new("virtual-normal-load-observed");
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
                for id in IDS {
                    let source = self.scenarios.iter().find(|s| s.id == id).unwrap();
                    let a = &self.progress.attempts[id];
                    binding::context(a, source).unwrap();
                    let engine::State::VirtualIsolation { simulation } = &a.state else {
                        panic!()
                    };
                    let d = binding::source(source).unwrap();
                    assert_eq!(
                        simulation.assignment.configuration,
                        expected_partial(&d.assignment)
                    );
                    assert_eq!(
                        simulation.practice.configuration,
                        expected_partial(&d.example)
                    );
                    assert!(!simulation.assignment.checked && !simulation.practice.checked);
                    assert!(!progress::current_solved(a, source));
                    assert!(
                        simulation.assignment.actual.is_none()
                            && simulation.practice.actual.is_none()
                    );
                }
                assert_eq!(self.scale, 1.5);
                assert_eq!(self.theme_id, THEME);
                session.write("reader-loaded.json", &loaded).unwrap();
                ctx.data_mut(|d| d.insert_temp(key, true));
            }
        }
        self.virtual_flow(ctx, plan(session.role == Role::Read));
    }
    pub(in crate::ui) fn finish_virtual_reopen(&mut self, ctx: &egui::Context) {
        let session = self.reopen.as_ref().unwrap().clone();
        assert_eq!(session.case, Case::VirtualIsolation);
        for id in IDS {
            let source = self.scenarios.iter().find(|s| s.id == id).unwrap();
            let a = &self.progress.attempts[id];
            let engine::State::VirtualIsolation { simulation } = &a.state else {
                panic!()
            };
            let d = binding::source(source).unwrap();
            if session.role == Role::Write {
                assert_eq!(
                    simulation.assignment.configuration,
                    expected_partial(&d.assignment)
                );
                assert_eq!(
                    simulation.practice.configuration,
                    expected_partial(&d.example)
                );
                assert!(!simulation.assignment.checked && !simulation.practice.checked);
                assert!(!progress::current_solved(a, source));
            } else {
                assert!(progress::current_solved(a, source));
                assert!(simulation.practice.solved(&d.example));
            }
        }
        let key = egui::Id::new("virtual-cold-finished");
        if !ctx.data(|d| d.get_temp::<bool>(key)).unwrap_or(false) {
            self.save();
            assert!(!self.dirty && self.storage_error.is_none());
            let (observed, raw, prefs) = session.observed(self);
            let (progress, preferences, observation) = if session.role == Role::Write {
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
            session.write_raw(progress, &raw, 16_000_000).unwrap();
            session.write_raw(preferences, &prefs, 4096).unwrap();
            session.write(observation, &observed).unwrap();
            ctx.data_mut(|d| d.insert_temp(key, true));
            println!(
                "PASS native reopen {}: actual host and VRF configuration kinds, four independent partial drafts and native repairs",
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
fn virtual_cold_plan_uses_real_preferences_and_both_kinds_without_style_injection() {
    for read in [false, true] {
        let p = plan(read);
        assert_eq!(
            p.iter()
                .filter_map(|o| if let Op::Open(id) = o {
                    Some(id.as_str())
                } else {
                    None
                })
                .collect::<Vec<_>>(),
            IDS
        );
        assert!(!p.iter().any(|o| matches!(o, Op::Style(..))));
        assert!(matches!(p.last(), Some(Op::ColdFinish)));
        assert_eq!(
            p.iter().filter(|o| matches!(o, Op::ColdAppearance)).count(),
            usize::from(!read)
        );
    }
    let writer = plan(false);
    for k in ["up:ledger-a", "up:archive-a"] {
        assert!(writer.iter().any(|o| matches!(o,Op::Click(v)if v==k)));
    }
    for k in ["context:east-out", "context:studio-out"] {
        assert!(writer.iter().any(|o| matches!(o,Op::Choose(v,_)if v==k)));
    }
}
