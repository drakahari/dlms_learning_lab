//! Fixed normal-catalog PTR acceptance through actual input events. No State injection.
use super::*;
use dlms_learning_lab::{
    dns::{Data, QueryType, Zone},
    dns_workspace::{self, Draft, Requirement, Snapshot},
    engine::{Goal, State},
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
const START: usize = 27000;
const END: usize = 28000;
const IDS: [&str; 3] = [
    "lpic2-dns-reverse-investigation",
    "lpic2-dns-reverse-ipv6",
    "lpic2-dns-reverse-handover",
];
#[derive(Clone, Copy, Debug, PartialEq)]
enum Field {
    Records,
    Serial,
    Query,
}
impl Field {
    fn key(self) -> &'static str {
        match self {
            Self::Records => "records",
            Self::Serial => "serial",
            Self::Query => "query",
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
enum Target {
    Try,
    Assignment,
    Example,
    Fill,
    Field(Field),
    Save,
    Reload,
    Update,
    Check,
    Type,
    Copy,
    Choice(String),
    Lookup,
    Reset,
    Confirm,
    Cancel,
    Challenge,
    Learn,
}
#[derive(Clone, Debug, PartialEq)]
enum Step {
    Library,
    Filter(usize),
    Start,
    Wait,
    Click(Target),
    Text(Field, String),
    BeforeFill,
    FillOnly,
    Rows(bool, Vec<bool>, bool),
    AppliedSnapshot,
    UnchangedApplied,
    Lookup(String),
    Remember,
    Resume,
    SaveReadback,
    Snapshot,
    CancelExact,
    ResetExact,
    Theme(bool, f32, bool),
    Geometry(f32, bool),
    Mode(bool),
    Shot(String, u8),
    Finish,
}
fn reference(s: &Scenario, practice: bool) -> Zone {
    let commands = if practice {
        &s.dns_teaching.as_ref().unwrap().example_solution
    } else {
        &s.solution
    };
    let zone: Zone = serde_json::from_str(
        commands
            .first()
            .unwrap()
            .strip_prefix("zone ")
            .expect("fixed authoring data, never a learner command"),
    )
    .unwrap();
    zone.check_configuration().unwrap();
    zone
}
fn text(zone: &Zone) -> String {
    zone.records
        .iter()
        .map(dns_workspace::record_text)
        .collect::<Vec<_>>()
        .join("\n")
}
fn field(p: &mut Vec<Step>, which: Field, value: String) {
    p.extend([Step::Click(Target::Field(which)), Step::Text(which, value)]);
}
fn edit(p: &mut Vec<Step>, z: &Zone) {
    field(p, Field::Records, text(z));
    field(p, Field::Serial, z.serial.to_string());
}
fn publish(p: &mut Vec<Step>, secondary: bool) {
    p.extend([Step::Click(Target::Save), Step::Click(Target::Reload)]);
    if secondary {
        p.push(Step::Click(Target::Update));
    }
}
fn check(p: &mut Vec<Step>, practice: bool, rows: Vec<bool>, pass: bool, shot: String) {
    p.extend([
        Step::Click(Target::Check),
        Step::Wait,
        Step::Rows(practice, rows, pass),
        Step::Shot(shot, 1),
    ]);
}
fn experiment(s: &Scenario, n: usize) -> Zone {
    let mut z = reference(s, true);
    z.serial = z.serial.checked_add(1).unwrap();
    if n == 2 {
        let old = s
            .dns_teaching
            .as_ref()
            .unwrap()
            .outcomes
            .iter()
            .find_map(|r| {
                if let Requirement::NoAnswer {
                    copy: Snapshot::Primary,
                    name,
                    ..
                } = r
                {
                    Some(name)
                } else {
                    None
                }
            })
            .unwrap();
        let target = z
            .records
            .iter()
            .find_map(|r| {
                if let Data::Ptr { target } = &r.data {
                    if !r.owner.starts_with("53.") {
                        Some(target.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap();
        z.records.push(dlms_learning_lab::dns::Record {
            owner: old.clone(),
            ttl: 300,
            data: Data::Ptr { target },
        });
    } else {
        let owner = s
            .dns_teaching
            .as_ref()
            .unwrap()
            .outcomes
            .iter()
            .find_map(|r| {
                if let Requirement::Records {
                    copy: Snapshot::Configured,
                    record_type: QueryType::Ptr,
                    owner,
                    ..
                } = r
                {
                    Some(owner)
                } else {
                    None
                }
            })
            .unwrap();
        let row = z.records.iter_mut().find(|r| &r.owner == owner).unwrap();
        if n == 0 {
            row.data = Data::Ptr {
                target: "wrong.training.example.test.".into(),
            };
        } else {
            assert!(row.owner.starts_with("9."));
            row.owner.replace_range(..1, "8");
        }
    }
    z
}
fn plan(scenarios: &[Scenario]) -> Vec<Step> {
    let mut p = Vec::new();
    for (n, id) in IDS.iter().enumerate() {
        let s = scenarios
            .iter()
            .find(|s| s.id == *id)
            .expect("PTR native flow requires normal registered IDs; never appends a fixture");
        assert_eq!(s.unit, "dns-reverse");
        let lesson = s.dns_teaching.as_ref().unwrap();
        let count = if n == 2 { 13 } else { 8 };
        let good = vec![true; count];
        assert_eq!(lesson.outcomes.len(), count);
        let Goal::AuthoritativeDns { requirements } = &s.goal else {
            panic!()
        };
        assert_eq!(requirements.len(), count);
        let shot = |label: &str| format!("dns-reverse-{n}-{label}");
        p.extend([
            Step::Library,
            Step::Filter(n),
            Step::Wait,
            Step::Start,
            Step::Wait,
            Step::Shot(shot("introduction"), 0),
            Step::Click(Target::Try),
            Step::Wait,
            Step::BeforeFill,
            Step::Click(Target::Fill),
            Step::FillOnly,
        ]);
        publish(&mut p, n == 2);
        check(
            &mut p,
            true,
            good.clone(),
            true,
            shot("example-complete-ungraded"),
        );
        let changed = experiment(s, n);
        edit(&mut p, &changed);
        p.push(Step::Click(Target::Save));
        if n != 0 {
            p.push(Step::Click(Target::Reload));
        }
        let mut bad = good.clone();
        for index in match n {
            0 => vec![0, 4],
            1 => vec![0, 1],
            _ => vec![6, 8],
        } {
            bad[index] = false;
        }
        check(&mut p, true, bad, false, shot("example-experiment"));
        let mut repair = reference(s, true);
        repair.serial = repair.serial.checked_add(2).unwrap();
        edit(&mut p, &repair);
        publish(&mut p, n == 2);
        check(&mut p, true, good.clone(), true, shot("example-repaired"));
        p.extend([
            Step::Click(Target::Assignment),
            Step::Wait,
            Step::Shot(shot("assignment"), 0),
        ]);
        let initial = if n == 2 {
            vec![
                false, false, false, false, false, false, false, false, false, true, true, true,
                true,
            ]
        } else {
            vec![false, false, false, false, true, true, true, true]
        };
        check(&mut p, false, initial, false, shot("initial-feedback"));
        let reference = reference(s, false);
        edit(&mut p, &reference);
        p.push(Step::Click(Target::Save));
        let partial = if n == 2 {
            vec![
                true, false, false, true, false, false, false, false, false, true, true, true, true,
            ]
        } else {
            vec![true, false, true, false, false, true, true, true]
        };
        check(&mut p, false, partial, false, shot("saved-only-feedback"));
        p.push(Step::Click(Target::Reload));
        if n == 2 {
            p.push(Step::Click(Target::Update));
        }
        check(&mut p, false, good.clone(), true, shot("complete"));
        // These are presentation-only commands, not global desktop changes.
        if n == 1 {
            p.extend([Step::Theme(true, 1.5, false), Step::Geometry(1.5, false)]);
        }
        if n == 2 {
            p.extend([
                Step::Theme(true, 1.5, true),
                Step::Geometry(1.5, true),
                Step::Shot(shot("light150-wide"), 0),
                Step::Theme(false, 2., false),
                Step::Geometry(2., false),
            ]);
        }
        let mut alternate = reference.clone();
        alternate.serial = alternate.serial.checked_add(10).unwrap();
        alternate.records.reverse();
        for r in &mut alternate.records {
            r.ttl = 600;
        }
        if n == 2 {
            let old = requirements
                .iter()
                .find_map(|r| {
                    if let Requirement::NoAnswer {
                        copy: Snapshot::Primary,
                        name,
                        ..
                    } = r
                    {
                        Some(name)
                    } else {
                        None
                    }
                })
                .unwrap();
            alternate.records.push(dlms_learning_lab::dns::Record {
                owner: old.clone(),
                ttl: 600,
                data: Data::A {
                    address: "192.0.2.36".parse().unwrap(),
                },
            });
        }
        edit(&mut p, &alternate);
        publish(&mut p, n == 2);
        check(
            &mut p,
            false,
            good.clone(),
            true,
            shot(if n == 0 {
                "alternate"
            } else if n == 1 {
                "light150-typed-alternate"
            } else {
                "dark200-typed-nodata-alternate"
            }),
        );
        // Real lookups use field text and actual type/copy pickers; query output is never grade authority.
        field(
            &mut p,
            Field::Query,
            requirements
                .iter()
                .find_map(|r| {
                    if let Requirement::Resolves {
                        copy: Snapshot::Primary,
                        name,
                        ..
                    } = r
                    {
                        Some(name.clone())
                    } else {
                        None
                    }
                })
                .unwrap(),
        );
        p.extend([
            Step::Click(Target::Type),
            Step::Click(Target::Choice("type:PTR".into())),
            Step::Click(Target::Lookup),
            Step::Lookup("PTR".into()),
            Step::Rows(false, good.clone(), true),
            Step::Shot(shot("primary-ptr-query"), 2),
        ]);
        if n == 2 {
            let old = requirements
                .iter()
                .find_map(|r| {
                    if let Requirement::NoAnswer {
                        copy: Snapshot::Secondary,
                        name,
                        ..
                    } = r
                    {
                        Some(name.clone())
                    } else {
                        None
                    }
                })
                .unwrap();
            field(&mut p, Field::Query, old);
            p.extend([
                Step::Click(Target::Copy),
                Step::Click(Target::Choice("copy:Secondary".into())),
                Step::Click(Target::Lookup),
                Step::Lookup("Noerror".into()),
                Step::Rows(false, good.clone(), true),
                Step::Shot(shot("secondary-retired-query"), 2),
            ]);
        }
        p.push(Step::AppliedSnapshot);
        field(&mut p, Field::Serial, "bad serial".into());
        p.push(Step::Click(Target::Save));
        p.push(Step::UnchangedApplied);
        check(
            &mut p,
            false,
            good.clone(),
            false,
            shot("malformed-save-feedback"),
        );
        p.push(Step::Shot(shot("malformed-save-diagnostic"), 3));
        let mut retry = alternate.clone();
        retry.serial = retry.serial.checked_add(2).unwrap();
        edit(&mut p, &retry);
        publish(&mut p, n == 2);
        check(
            &mut p,
            false,
            good.clone(),
            true,
            shot("typed-retry-complete"),
        );
        p.extend([
            Step::Click(Target::Challenge),
            Step::Wait,
            Step::Mode(true),
            Step::Shot(shot("challenge"), 0),
            Step::Click(Target::Learn),
            Step::Wait,
            Step::Mode(false),
        ]);
        field(
            &mut p,
            Field::Records,
            format!("{}\nunfinished assignment", text(&retry)),
        );
        field(&mut p, Field::Serial, "raw asgn".into());
        p.extend([Step::Click(Target::Example), Step::Wait]);
        field(
            &mut p,
            Field::Records,
            format!("{}\nindependent practice draft", text(&repair)),
        );
        field(&mut p, Field::Serial, "raw exmp".into());
        p.extend([
            Step::Remember,
            Step::SaveReadback,
            Step::Library,
            Step::Filter(n),
            Step::Wait,
            Step::Start,
            Step::Wait,
            Step::Resume,
            Step::Shot(shot("separate-drafts-resumed"), 0),
            Step::Click(Target::Assignment),
            Step::Wait,
            Step::Snapshot,
            Step::Click(Target::Reset),
            Step::Click(Target::Cancel),
            Step::CancelExact,
            Step::Click(Target::Reset),
            Step::Click(Target::Confirm),
            Step::Wait,
            Step::ResetExact,
            Step::Click(Target::Assignment),
            Step::Wait,
        ]);
        edit(&mut p, &reference);
        publish(&mut p, n == 2);
        check(&mut p, false, good, true, shot("reset-repaired"));
        p.push(Step::SaveReadback);
    }
    p.push(Step::Finish);
    assert!(p.len() < END - START);
    assert!(
        p.iter()
            .all(|s| !matches!(s,Step::Text(_,t) if t.len()>dns_workspace::MAX_TEXT))
    );
    p
}
fn hold_close(ctx: &egui::Context) -> (bool, Option<String>) {
    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    (false, None)
}
impl LabApp {
    fn ptr_draft(&self, i: usize) -> &Draft {
        let w = self.progress.attempts[&self.scenarios[i].id]
            .workspace
            .as_ref()
            .unwrap();
        if self.dns_view(i) == teaching::View::Example {
            &w.dns_learning.as_ref().unwrap().draft
        } else {
            w.dns_draft.as_ref().unwrap()
        }
    }
    fn ptr_target(&self, t: &Target) -> Option<egui::Rect> {
        match t {
            Target::Try => self.metrics.example_action,
            Target::Assignment => self
                .metrics
                .assignment_action
                .or_else(|| self.metrics.lesson_tabs.get(2).copied()),
            Target::Example => self.metrics.lesson_tabs.get(1).copied(),
            Target::Fill => self.metrics.insert,
            Target::Field(f) => self
                .metrics
                .dns_fields
                .iter()
                .find(|(k, _, _)| k == f.key())
                .map(|(_, r, _)| *r),
            Target::Save => self.metrics.editor_save,
            Target::Reload => self.metrics.dns_reload,
            Target::Update => self.metrics.dns_update,
            Target::Check => self.metrics.check,
            Target::Type => self.metrics.dns_type,
            Target::Copy => self.metrics.dns_copy,
            Target::Choice(k) => self
                .metrics
                .dns_choices
                .iter()
                .find(|(name, _)| name == k)
                .map(|(_, r)| *r),
            Target::Lookup => self.metrics.probe,
            Target::Reset => self.metrics.reset,
            Target::Confirm => self.metrics.confirm_reset,
            Target::Cancel => self.metrics.reset_cancel,
            Target::Challenge => self.metrics.mode_challenge,
            Target::Learn => self.metrics.mode_learn,
        }
    }
    fn ptr_click(&mut self, ctx: &egui::Context, target: &Target) -> bool {
        let key = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(key)).unwrap_or(0) >= 2 {
            return self.click_visible_smoke_control(ctx, None);
        }
        if matches!(target, Target::Choice(_)) {
            return self.mac_popup_choice(ctx, self.ptr_target(target), self.metrics.dns_popup);
        }
        let rect = if matches!(target, Target::Field(Field::Records)) {
            let r = self.metrics.editor_viewport.unwrap();
            let h = ctx.style().spacing.interact_size.y.min(r.height());
            Some(egui::Rect::from_min_size(r.min, egui::vec2(r.width(), h)))
        } else {
            self.ptr_target(target)
        };
        self.click_visible_smoke_control(ctx, rect)
    }
    fn ptr_settled(&self, ctx: &egui::Context, scale: f32, wide: bool) -> bool {
        let rect = ctx.content_rect();
        let zoom = ctx.zoom_factor();
        let ppp = ctx.pixels_per_point();
        let native_width = rect.width() * zoom;
        assert!(rect.is_finite() && ppp.is_finite() && ppp > 0.);
        let (tries, stable) = ctx.data_mut(|d| {
            let key = egui::Id::new(("ptr-geometry", self.smoke_stage));
            let (tries, old, stable) =
                d.get_temp::<(u8, egui::Rect, u8)>(key)
                    .unwrap_or((0, egui::Rect::NOTHING, 0));
            let match_width = if wide {
                native_width >= 1600.
            } else {
                native_width <= 1500.
            };
            let stable = if old == rect && (zoom - scale).abs() < 0.01 && match_width {
                stable + 1
            } else {
                0
            };
            d.insert_temp(key, (tries + 1, rect, stable));
            (tries + 1, stable)
        });
        assert!(
            tries <= 30,
            "bounded PTR real geometry/zoom: {rect:?} zoom{zoom} wide{wide}"
        );
        if stable < 2 {
            ctx.request_repaint();
        }
        stable >= 2
    }
    fn ptr_start(&mut self, ctx: &egui::Context, id: &str) -> bool {
        let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
        let key = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(key)).unwrap_or(0) >= 2 {
            let done = self.click_visible_smoke_control(ctx, None);
            assert_eq!(self.selected, Some(i));
            return done;
        }
        assert!(self.selected.is_none());
        let visible: Vec<_> = self
            .scenarios
            .iter()
            .filter(|s| {
                catalog::matches(
                    s,
                    &self.filter,
                    self.progress.attempts.contains_key(&s.id),
                    self.progress
                        .attempts
                        .get(&s.id)
                        .is_some_and(|a| progress::current_solved(a, s)),
                    self.progress.favorites.contains(&s.id),
                )
            })
            .collect();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].id, id);
        assert_eq!(self.metrics.card_actions.len(), 1);
        self.click_visible_smoke_control(ctx, self.metrics.card_actions.first().map(|r| r.0))
    }
    pub(super) fn dns_reverse_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        assert!((START..END).contains(&stage));
        let p = plan(&self.scenarios);
        let offset = stage - START;
        let step = &p[offset];
        let n = p[..=offset]
            .iter()
            .rev()
            .find_map(|s| {
                if let Step::Filter(n) = s {
                    Some(*n)
                } else {
                    None
                }
            })
            .unwrap_or(0);
        let id = IDS[n];
        let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
        let scenario = self.scenarios[i].clone();
        if std::env::var_os("DLMS_SMOKE_DIAGNOSTICS").is_some() {
            eprintln!("ptr-native {stage} {id} {step:?}");
        }
        match step {
            Step::Library => {
                let key = egui::Id::new(("native-click-phases", stage));
                let ready = if ctx.data(|d| d.get_temp::<u8>(key)).unwrap_or(0) >= 2 {
                    self.click_visible_smoke_control(ctx, None)
                } else if self.selected.is_some() {
                    self.click_visible_smoke_control(ctx, self.metrics.back)
                } else {
                    true
                };
                if !ready {
                    return (false, None);
                }
                assert!(self.selected.is_none());
            }
            Step::Filter(_) => {
                assert!(self.selected.is_none());
                self.filter = Filter::default();
                self.filter.search = scenario.title.clone();
                self.filter.max_minutes = 120;
                self.page = 0;
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1268., 632.)));
            }
            Step::Start => {
                if !self.ptr_start(ctx, id) {
                    return (false, None);
                }
            }
            Step::Wait => {}
            Step::Click(target) => {
                if !self.ptr_click(ctx, target) {
                    return (false, None);
                }
                if let Target::Choice(key) = target {
                    let d = self.ptr_draft(i);
                    let current = if key.starts_with("type:") {
                        format!("type:{}", dns_workspace::type_text(d.query_type))
                    } else {
                        format!("copy:{}", d.query_copy.label())
                    };
                    assert_eq!(&current, key, "actual popup selected requested value");
                }
            }
            Step::Text(field, text) => {
                let (_, _, id) = self
                    .metrics
                    .dns_fields
                    .iter()
                    .find(|(k, _, _)| k == field.key())
                    .unwrap();
                assert_eq!(
                    ctx.memory(|m| m.focused()),
                    Some(*id),
                    "real field focus before typing"
                );
                self.pending_events = vec![
                    egui::Event::Key {
                        key: egui::Key::A,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers {
                            ctrl: true,
                            command: true,
                            ..Default::default()
                        },
                    },
                    egui::Event::Text(text.clone()),
                ];
            }
            Step::BeforeFill => {
                let a = &self.progress.attempts[id];
                let l = a.workspace.as_ref().unwrap().dns_learning.as_ref().unwrap();
                let value = (
                    serde_json::to_value(&a.state).unwrap(),
                    serde_json::to_value(a.workspace.as_ref().unwrap().dns_draft.as_ref().unwrap())
                        .unwrap(),
                    serde_json::to_value(&l.server).unwrap(),
                );
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("ptr-before-fill"), value));
            }
            Step::FillOnly => {
                let b = ctx
                    .data(|d| d.get_temp::<(Value, Value, Value)>(egui::Id::new("ptr-before-fill")))
                    .unwrap();
                let a = &self.progress.attempts[id];
                let w = a.workspace.as_ref().unwrap();
                let l = w.dns_learning.as_ref().unwrap();
                assert_eq!(serde_json::to_value(&a.state).unwrap(), b.0);
                assert_eq!(
                    serde_json::to_value(w.dns_draft.as_ref().unwrap()).unwrap(),
                    b.1
                );
                assert_eq!(serde_json::to_value(&l.server).unwrap(), b.2);
                assert!(!l.checked && !l.draft.matches(&l.server));
                assert_eq!(l.draft.text, text(&reference(&scenario, true)));
                assert!(!progress::current_solved(a, &scenario));
            }
            Step::Rows(practice, expected, pass) => {
                let a = &self.progress.attempts[id];
                let rows = if *practice {
                    assert_eq!(a.state, scenario.initial);
                    assert!(!progress::current_solved(a, &scenario));
                    let l = a.workspace.as_ref().unwrap().dns_learning.as_ref().unwrap();
                    assert!(l.checked);
                    l.checks(scenario.dns_teaching.as_ref().unwrap())
                } else {
                    assert_eq!(progress::current_solved(a, &scenario), *pass);
                    assert_eq!(self.metrics.completion, *pass);
                    engine::checks(&a.state, &scenario.goal)
                };
                assert_eq!(rows.iter().map(|c| c.0).collect::<Vec<_>>(), *expected);
                assert!(self.metrics.dns_result.is_some());
            }
            Step::AppliedSnapshot => {
                let v = serde_json::to_value(&self.progress.attempts[id].state).unwrap();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("ptr-applied"), v));
            }
            Step::UnchangedApplied => {
                let before = ctx
                    .data(|d| d.get_temp::<Value>(egui::Id::new("ptr-applied")))
                    .unwrap();
                assert_eq!(
                    serde_json::to_value(&self.progress.attempts[id].state).unwrap(),
                    before
                );
                assert_eq!(self.ptr_draft(i).serial, "bad serial");
                assert!(
                    self.ptr_draft(i)
                        .notice
                        .as_ref()
                        .unwrap()
                        .starts_with("Cannot save:")
                );
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
            }
            Step::Lookup(value) => {
                let d = self.ptr_draft(i);
                assert_eq!(d.query_type, QueryType::Ptr);
                assert!(d.output.contains(value));
                assert!(self.metrics.output.is_some());
            }
            Step::Remember => {
                let a = &self.progress.attempts[id];
                let w = a.workspace.as_ref().unwrap();
                assert!(
                    w.dns_draft
                        .as_ref()
                        .unwrap()
                        .text
                        .ends_with("unfinished assignment")
                );
                assert!(
                    w.dns_learning
                        .as_ref()
                        .unwrap()
                        .draft
                        .text
                        .ends_with("independent practice draft")
                );
                assert_eq!(w.dns_draft.as_ref().unwrap().serial, "raw asgn");
                assert_eq!(w.dns_learning.as_ref().unwrap().draft.serial, "raw exmp");
                let v = serde_json::to_value(a).unwrap();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("ptr-resume"), v));
            }
            Step::Resume => {
                let mut before = ctx
                    .data(|d| d.get_temp::<Value>(egui::Id::new("ptr-resume")))
                    .unwrap();
                before["visited"] = serde_json::json!(self.progress.clock);
                assert_eq!(
                    serde_json::to_value(&self.progress.attempts[id]).unwrap(),
                    before
                );
                assert_eq!(self.dns_view(i), teaching::View::Example);
            }
            Step::SaveReadback => {
                self.save();
                assert!(self.storage_error.is_none());
                let bytes = progress::read_bounded(&self.path, 16_000_000).unwrap();
                let raw: Value = serde_json::from_slice(&bytes).unwrap();
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                let portable = serde_json::to_value(&self.progress).unwrap();
                assert_eq!(serde_json::to_value(&loaded).unwrap(), portable);
                let mut exact = portable;
                exact["migration_receipts"] =
                    serde_json::to_value(&loaded.migration_receipts).unwrap();
                assert_eq!(raw, exact);
                assert_eq!(
                    progress::read_bounded(&self.path, 16_000_000).unwrap(),
                    bytes
                );
            }
            Step::Snapshot => {
                let a = serde_json::to_value(&self.progress.attempts[id]).unwrap();
                let others: BTreeMap<_, _> = self
                    .progress
                    .attempts
                    .iter()
                    .filter(|(key, _)| key.as_str() != id)
                    .map(|(k, a)| (k.clone(), serde_json::to_value(a).unwrap()))
                    .collect();
                let favorites = self.progress.favorites.clone();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("ptr-reset"), (a, others, favorites)));
            }
            Step::CancelExact | Step::ResetExact => {
                let before = ctx
                    .data(|d| {
                        d.get_temp::<(Value, BTreeMap<String, Value>, BTreeSet<String>)>(
                            egui::Id::new("ptr-reset"),
                        )
                    })
                    .unwrap();
                let a = &self.progress.attempts[id];
                assert!(!self.reset_pending);
                if matches!(step, Step::CancelExact) {
                    assert_eq!(serde_json::to_value(a).unwrap(), before.0);
                } else {
                    assert_eq!(a.state, scenario.initial);
                    assert!(a.transcript.is_empty());
                    assert!(!progress::current_solved(a, &scenario));
                    let w = a.workspace.as_ref().unwrap();
                    let State::AuthoritativeDns { server } = &a.state else {
                        panic!()
                    };
                    assert_eq!(w.dns_draft.as_ref().unwrap(), &Draft::new(server));
                    assert_eq!(
                        w.dns_learning.as_ref().unwrap().server,
                        scenario.dns_teaching.as_ref().unwrap().fixture
                    );
                }
                assert_eq!(self.progress.favorites, before.2);
                for (key, a) in before.1 {
                    assert_eq!(
                        serde_json::to_value(&self.progress.attempts[&key]).unwrap(),
                        a
                    );
                }
            }
            Step::Theme(light, scale, wide) => {
                self.theme_id = if *light { "dlms-light" } else { "dlms-dark" }.into();
                self.scale = *scale;
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(*wide));
                if !wide {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                        1268., 632.,
                    )));
                }
                self.lesson_scroll_reset = true;
            }
            Step::Geometry(scale, wide) => {
                if !self.ptr_settled(ctx, *scale, *wide) {
                    return (false, None);
                }
            }
            Step::Mode(challenge) => {
                assert_eq!(self.progress.attempts[id].challenge, *challenge);
                assert_eq!(self.dns_view(i), teaching::View::Assignment);
                if *challenge {
                    assert!(
                        self.metrics.lesson_tabs.is_empty()
                            && !self.metrics.guide
                            && !self.help_open
                    );
                }
            }
            Step::Shot(name, kind) => {
                let r = match kind {
                    0 => None,
                    1 => Some(self.metrics.dns_result.expect("measured current feedback")),
                    2 => Some(self.metrics.output.expect("measured current lookup output")),
                    3 => Some(
                        self.metrics
                            .dns_notice
                            .expect("measured current Save diagnostic"),
                    ),
                    _ => panic!("unknown fixed PTR capture kind"),
                };
                if let Some(r) = r
                    && !self.reveal_smoke_control(ctx, r)
                {
                    return (false, None);
                }
                return (true, Some(name.clone()));
            }
            Step::Finish => {
                for id in IDS {
                    let s = self.scenarios.iter().find(|s| s.id == id).unwrap();
                    assert!(progress::current_solved(&self.progress.attempts[id], s));
                }
                let key = egui::Id::new("ptr-finish-announced");
                if !ctx.data(|d| d.get_temp::<bool>(key)).unwrap_or(false) {
                    println!(
                        "PASS three normal PTR lessons: actual controls, independent practice, saved-only/wrong/current/alternate states, query/negative-copy, typed150/200 retry, exact drafts/navigation/reset/readback. No novice/cold/other-platform claim."
                    );
                    ctx.data_mut(|d| d.insert_temp(key, true));
                }
                return hold_close(ctx);
            }
        }
        (true, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame(ctx: &egui::Context, app: &mut LabApp, events: Vec<egui::Event>) {
        let modifiers = events
            .iter()
            .find_map(|e| {
                if let egui::Event::Key { modifiers, .. } = e {
                    Some(*modifiers)
                } else {
                    None
                }
            })
            .unwrap_or_default();
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1268., 1600.),
                )),
                events,
                modifiers,
                ..Default::default()
            },
            |ctx| app.render(ctx),
        );
    }
    fn click(ctx: &egui::Context, app: &mut LabApp, target: Target) {
        app.smoke_stage += 1;
        for _ in 0..24 {
            let done = app.ptr_click(ctx, &target);
            let events = std::mem::take(&mut app.pending_events);
            frame(ctx, app, events);
            if done {
                return;
            }
        }
        panic!("bounded real PTR control click: {target:?}");
    }
    #[test]
    fn ptr_native_normal_plan_is_bounded_separate_and_covers_every_requested_path() {
        let scenarios = pack::bundled().unwrap();
        let p = plan(&scenarios);
        assert!(START + p.len() < END);
        assert_eq!(p.last(), Some(&Step::Finish));
        for n in 0..3 {
            assert_eq!(
                p.iter()
                    .filter(|s| matches!(s,Step::Filter(i)if *i==n))
                    .count(),
                2
            );
            let begin = p
                .iter()
                .position(|s| matches!(s,Step::Filter(i)if *i==n))
                .unwrap();
            let end = p[begin + 1..]
                .iter()
                .position(|s| matches!(s,Step::Filter(i)if *i!=n))
                .map_or(p.len(), |i| begin + 1 + i);
            let part = &p[begin..end];
            for wanted in [
                Step::BeforeFill,
                Step::FillOnly,
                Step::AppliedSnapshot,
                Step::UnchangedApplied,
                Step::Remember,
                Step::Resume,
                Step::CancelExact,
                Step::ResetExact,
                Step::Mode(true),
                Step::Mode(false),
            ] {
                assert!(part.contains(&wanted), "task{n}: {wanted:?}");
            }
            assert!(
                part.iter()
                    .any(|s| matches!(s,Step::Rows(false,rows,false)if rows.iter().any(|v|*v)))
            );
            assert!(
                part.iter()
                    .any(|s| matches!(s,Step::Rows(false,rows,true)if rows.iter().all(|v|*v)))
            );
            assert!(
                part.iter()
                    .any(|s| matches!(s,Step::Click(Target::Choice(v))if v=="type:PTR"))
            );
            assert!(part.iter().all(|s|!matches!(s,Step::Text(_,v)if v.len()>dns_workspace::MAX_TEXT||v.starts_with("zone {"))));
        }
        for scale in [1.5, 2.] {
            let start = p
                .iter()
                .position(|s| matches!(s,Step::Geometry(v,_)if *v==scale))
                .unwrap();
            let end = p[start + 1..]
                .iter()
                .position(|s| matches!(s, Step::Theme(..) | Step::Filter(_)))
                .map_or(p.len(), |i| start + 1 + i);
            assert!(
                p[start..end]
                    .iter()
                    .any(|s| matches!(s, Step::Text(Field::Records, _)))
            );
            assert!(p[start..end].contains(&Step::Click(Target::Save)));
            assert!(p[start..end].contains(&Step::Click(Target::Reload)));
        }
        assert!(p.contains(&Step::Geometry(1.5, true)));
        assert!(
            p.iter()
                .any(|s| matches!(s,Step::Click(Target::Choice(v))if v=="copy:Secondary"))
        );
        let incomplete: Vec<_> = scenarios.into_iter().filter(|s| s.id != IDS[2]).collect();
        assert!(
            std::panic::catch_unwind(|| plan(&incomplete)).is_err(),
            "missing normal IDs must not append private fixtures"
        );
    }
    #[test]
    fn ptr_native_actual_geometry_phase_samples_context_outside_its_data_lock() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context(&ctx, true, false);
        app.smoke = false;
        app.smoke_stage = START + 1;
        for expected in 1..=3 {
            let mut done = false;
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1268., 632.),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    done = app.ptr_settled(ctx, 1., false);
                },
            );
            let sample = ctx
                .data(|d| {
                    d.get_temp::<(u8, egui::Rect, u8)>(egui::Id::new(("ptr-geometry", START + 1)))
                })
                .unwrap();
            assert_eq!(sample.0, expected);
            assert_eq!(sample.1, ctx.content_rect());
            assert_eq!(done, expected == 3);
        }
    }
    #[test]
    fn ptr_native_click_completion_precedes_disappeared_editor_metrics() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context(&ctx, true, false);
        app.smoke = false;
        app.smoke_stage = START + 2;
        ctx.data_mut(|d| d.insert_temp(egui::Id::new(("native-click-phases", START + 2)), 2_u8));
        assert!(app.metrics.editor_viewport.is_none());
        assert!(app.ptr_click(&ctx, &Target::Field(Field::Records)));
        assert!(app.pending_events.is_empty());
    }
    #[test]
    fn ptr_native_finish_rejects_unsolved_models_and_close_holds_its_phase() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context(&ctx, true, false);
        app.smoke = false;
        let p = plan(&app.scenarios);
        for id in IDS {
            let i = app.scenarios.iter().position(|s| s.id == id).unwrap();
            app.launch(i);
        }
        let stage = START + p.len() - 1;
        app.smoke_stage = stage;
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || app.dns_reverse_native(&ctx, stage)
            ))
            .is_err(),
            "Finish may not announce success for pristine tasks"
        );
        for _ in 0..3 {
            let mut result = (true, None);
            let out = ctx.run(egui::RawInput::default(), |ctx| result = hold_close(ctx));
            assert_eq!(result, (false, None));
            assert_eq!(app.smoke_stage, stage);
            assert!(
                out.viewport_output[&egui::ViewportId::ROOT]
                    .commands
                    .contains(&egui::ViewportCommand::Close)
            );
        }
    }
    #[test]
    fn ptr_native_measured_serial_typing_and_failed_save_preserve_applied_state() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context(&ctx, true, false);
        app.smoke = false;
        let i = app
            .scenarios
            .iter()
            .position(|s| matches!(s.initial, State::AuthoritativeDns { .. }))
            .unwrap();
        app.launch(i);
        app.set_dns_view(i, teaching::View::Assignment);
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![]);
        }
        let id = app.scenarios[i].id.clone();
        let before = app.progress.attempts[&id].state.clone();
        click(&ctx, &mut app, Target::Field(Field::Serial));
        let field_id = app
            .metrics
            .dns_fields
            .iter()
            .find(|(k, _, _)| k == "serial")
            .unwrap()
            .2;
        assert_eq!(ctx.memory(|m| m.focused()), Some(field_id));
        frame(
            &ctx,
            &mut app,
            vec![
                egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers {
                        ctrl: true,
                        command: true,
                        ..Default::default()
                    },
                },
                egui::Event::Text("bad serial".into()),
            ],
        );
        frame(&ctx, &mut app, vec![]);
        click(&ctx, &mut app, Target::Save);
        assert_eq!(app.ptr_draft(i).serial, "bad serial");
        assert!(
            app.ptr_draft(i)
                .notice
                .as_ref()
                .unwrap()
                .starts_with("Cannot save:")
        );
        assert_eq!(app.progress.attempts[&id].state, before);
        assert!(app.metrics.dns_notice.is_some());
        assert!(!progress::current_solved(
            &app.progress.attempts[&id],
            &app.scenarios[i]
        ));
    }
}
