//! Fixed native acceptance route. Uses real controls and editor events; no solved
//! State, draft, grading record or cursor history is installed by this driver.
use super::*;
const ID: &str = "ccna-private-office-plan";
const BASE: usize = 72000;
const SIBLING: &str = "ccna-ipv4-network-route";
#[derive(Clone, Debug)]
enum Step {
    SiblingLibrary,
    SiblingFavorite,
    SiblingStart,
    SiblingBack,
    Library,
    Start,
    Shot(&'static str),
    Example,
    Fill,
    Check,
    ExamplePass,
    ExampleFlags(Vec<bool>),
    Assignment,
    Addresses,
    Device,
    Choose(&'static str),
    Focus,
    Type(String),
    Flags(Vec<bool>),
    Invalidated,
    Malformed,
    Warm,
    Back,
    Resume,
    Challenge,
    Learn,
    Reset,
    Cancel,
    Confirm,
    Fresh,
    Settings(f32, bool, bool),
    Settled,
    RevealWorkspace,
    Done,
}
fn configure(steps: &mut Vec<Step>, a: &str, b: &str) {
    for (device, address) in [("Alder", a), ("Birch", b)] {
        steps.extend([
            Step::Device,
            Step::Choose(device),
            Step::Focus,
            Step::Type(address.into()),
        ]);
    }
}
fn plan() -> Vec<Step> {
    let mut p = vec![
        Step::SiblingLibrary,
        Step::SiblingFavorite,
        Step::SiblingStart,
        Step::Assignment,
        Step::Addresses,
        Step::Focus,
        Step::Type("separate sibling draft".into()),
        Step::SiblingBack,
        Step::Library,
        Step::Start,
        Step::Shot("introduction"),
        Step::Example,
        Step::Shot("separate-example"),
        Step::Fill,
        Step::Check,
        Step::ExamplePass,
        Step::Shot("example-verified-ungraded"),
        Step::Device,
        Step::Choose("Willow"),
        Step::Focus,
        Step::Type("172.22.4.34/28".into()),
        Step::Check,
        Step::ExampleFlags(vec![true, true, false, true]),
        Step::Shot("example-change-mismatched-mask"),
        Step::Focus,
        Step::Type("172.22.4.34/29".into()),
        Step::Check,
        Step::ExamplePass,
        Step::Shot("example-repaired-ungraded"),
        Step::Assignment,
        Step::Shot("full-assignment"),
        Step::Check,
        Step::Flags(vec![false, false, true, true]),
        Step::Shot("documentation-space-retry"),
    ];
    let mut first = Vec::new();
    configure(&mut first, "10.20.3.1/27", "10.20.3.2/26");
    p.extend(first[..4].iter().cloned());
    p.extend([
        Step::Check,
        Step::Flags(vec![true, false, false, false]),
        Step::Shot("only-one-computer-configured"),
    ]);
    p.extend(first[4..].iter().cloned());
    p.extend([
        Step::Check,
        Step::Flags(vec![true, true, false, true]),
        Step::Shot("traffic-passes-masks-disagree"),
    ]);
    p.extend([
        Step::Focus,
        Step::Type("10.20.3.2/27".into()),
        Step::Invalidated,
        Step::Check,
        Step::Flags(vec![true; 4]),
        Step::Shot("reference-verified"),
        Step::Focus,
        Step::Type("unfinished /".into()),
        Step::Invalidated,
        Step::Warm,
        Step::Back,
        Step::Resume,
        Step::Warm,
        Step::Shot("unfinished-draft-resumed"),
        Step::Check,
        Step::Malformed,
        Step::Shot("malformed-address-feedback"),
    ]);
    for (a, b, label) in [
        (
            "172.20.12.17/26",
            "172.20.12.62/26",
            "alternate-private-pool",
        ),
        (
            "192.168.72.5/27",
            "192.168.72.28/27",
            "alternate-subnet-plan",
        ),
    ] {
        configure(&mut p, a, b);
        p.extend([Step::Check, Step::Flags(vec![true; 4]), Step::Shot(label)]);
    }
    p.extend([
        Step::Challenge,
        Step::Shot("challenge-independent"),
        Step::Learn,
    ]);
    for scale in [1., 1.15, 1.25, 1.5, 2.] {
        for light in [false, true] {
            for wide in [false, true] {
                p.extend([
                    Step::Settings(scale, light, wide),
                    Step::Settled,
                    Step::Shot("matrix-assignment"),
                ]);
                if scale == 2. {
                    p.extend([Step::RevealWorkspace, Step::Shot("matrix-200-workspace")]);
                }
            }
        }
    }
    p.extend([
        Step::Reset,
        Step::Cancel,
        Step::Reset,
        Step::Confirm,
        Step::Fresh,
        Step::Shot("selected-attempt-reset"),
        Step::Done,
    ]);
    assert!(p.len() < 500);
    p
}
fn full(p: &Progress) -> serde_json::Value {
    let mut v = serde_json::to_value(p).unwrap();
    v["migration_receipts"] = serde_json::to_value(&p.migration_receipts).unwrap();
    v
}
fn without_practice(p: &Progress) -> serde_json::Value {
    let mut v = full(p);
    v["attempts"][ID]["workspace"]
        .as_object_mut()
        .unwrap()
        .remove("routing_v4_learning");
    v
}
pub(in crate::ui) fn reset_expected(
    before: &serde_json::Value,
    s: &pack::Scenario,
) -> serde_json::Value {
    let visited = before["attempts"][&s.id]["visited"].as_u64().unwrap();
    let mut fresh = progress::Attempt::new(s, visited);
    model::prepare(&mut fresh);
    dlms_learning_lab::ipv4_routing_teaching::prepare(
        &mut fresh,
        s.ipv4_routing_teaching.as_ref().unwrap(),
        true,
    );
    if let engine::Goal::Ipv4Routing { requirements } = &s.goal
        && let Some(name) = dlms_learning_lab::ipv4_routing_teaching::preferred_device(requirements)
    {
        fresh
            .workspace
            .as_mut()
            .unwrap()
            .routing_v4_draft
            .as_mut()
            .unwrap()
            .selected_device = name;
    }
    let mut result = before.clone();
    result["attempts"][&s.id] = serde_json::to_value(fresh).unwrap();
    result
}
pub(in crate::ui) fn reset_preserved(
    before: &serde_json::Value,
    after: &serde_json::Value,
    s: &pack::Scenario,
) -> bool {
    &reset_expected(before, s) == after
}
#[cfg(test)]
pub(in crate::ui) fn route_len() -> usize {
    plan().len()
}
impl LabApp {
    fn private_office_card_start(&mut self, ctx: &egui::Context, id: &str) -> bool {
        let phase = ctx
            .data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage))))
            .unwrap_or(0);
        let target = if phase >= 2 {
            None
        } else {
            assert!(self.selected.is_none());
            assert_eq!(
                self.metrics.card_actions.len(),
                1,
                "One real filtered catalog card"
            );
            Some(self.metrics.card_actions[0].0)
        };
        if !self.click_visible_smoke_control(ctx, target) {
            return false;
        }
        assert_eq!(self.scenarios[self.selected.unwrap()].id, id);
        true
    }
    pub(in crate::ui) fn private_office_native(
        &mut self,
        ctx: &egui::Context,
        n: usize,
    ) -> (bool, Option<String>) {
        let budget = ctx.data_mut(|d| {
            let key = egui::Id::new(("private-office-step-budget", n));
            let (count, began) = d
                .get_temp::<(usize, Instant)>(key)
                .unwrap_or((0, Instant::now()));
            d.insert_temp(key, (count + 1, began));
            (count + 1, began)
        });
        assert!(
            budget.0 <= 300 && budget.1.elapsed().as_secs() < 15,
            "Bounded native step {n} failed to settle: polls {}, elapsed {:?}",
            budget.0,
            budget.1.elapsed()
        );
        let steps = plan();
        let index = n - BASE;
        let step = steps.get(index).expect("Bounded native route");
        if budget.0 == 1 {
            eprintln!(
                "private-office dispatch step{index}: {step:?}; root={}, discard={}, selected={:?}, view={:?}, inputs={}, device={:?}, assignment={:?}",
                ctx.cumulative_frame_nr(),
                ctx.will_discard(),
                self.selected.map(|i| self.scenarios[i].id.as_str()),
                self.selected.map(|i| self.routing_v4_view(i)),
                self.metrics.inputs.len(),
                self.metrics.routing_device,
                self.metrics.assignment_action
            );
        }
        let i = self
            .scenarios
            .iter()
            .position(|s| s.id == ID)
            .expect("Normal bundled project");
        let view = |app: &Self| app.routing_v4_view(i);
        match step {
            Step::SiblingLibrary => {
                assert!(self.progress.attempts.is_empty());
                let source = self.scenarios.iter().find(|s| s.id == SIBLING).unwrap();
                self.filter = Filter {
                    track: source.track.clone(),
                    unit: source.unit.clone(),
                    search: source.title.clone(),
                    ..Default::default()
                };
            }
            Step::SiblingFavorite => {
                assert_eq!(self.metrics.card_actions.len(), 1);
                if !self.click_visible_smoke_control(ctx, Some(self.metrics.card_actions[0].1)) {
                    return (false, None);
                }
                assert!(self.progress.favorites.contains(SIBLING));
            }
            Step::SiblingStart => {
                if !self.private_office_card_start(ctx, SIBLING) {
                    return (false, None);
                }
            }
            Step::SiblingBack => {
                let a = &self.progress.attempts[SIBLING];
                let text = serde_json::to_string(
                    a.workspace
                        .as_ref()
                        .unwrap()
                        .routing_v4_draft
                        .as_ref()
                        .unwrap(),
                )
                .unwrap();
                assert!(text.contains("separate sibling draft"));
                assert!(!progress::current_solved(
                    a,
                    self.scenarios.iter().find(|s| s.id == SIBLING).unwrap()
                ));
                if !self.click_visible_smoke_control(ctx, self.metrics.back) {
                    return (false, None);
                }
                assert!(self.selected.is_none());
            }
            Step::Library => {
                assert!(self.selected.is_none());
                self.filter = Filter {
                    track: "CCNA".into(),
                    unit: "private-office-planning".into(),
                    search: self.scenarios[i].title.clone(),
                    ..Default::default()
                };
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
            }
            Step::Start => {
                if !self.private_office_card_start(ctx, ID) {
                    return (false, None);
                }
                assert_eq!(self.selected, Some(i));
                assert_eq!(view(self), teaching::View::Introduction);
            }
            Step::Shot(label) => {
                let size = ctx.content_rect().size() * ctx.pixels_per_point();
                eprintln!(
                    "private-office native step{index} {label}: actual_pixels={size:?}, text={}, theme={}, fullscreen={:?}",
                    self.scale,
                    self.theme_id,
                    ctx.input(|x| x.viewport().fullscreen)
                );
                return (true, Some(format!("private-office-{index:03}-{label}")));
            }
            Step::Example => {
                let key = egui::Id::new("private-office-before-practice-full");
                if ctx.data(|d| d.get_temp::<serde_json::Value>(key)).is_none() {
                    let before = without_practice(&self.progress);
                    ctx.data_mut(|d| d.insert_temp(key, before));
                }
                if !self.click_visible_smoke_control(ctx, self.metrics.example_action) {
                    return (false, None);
                }
            }
            Step::Fill => {
                if !self.click_visible_smoke_control(ctx, self.metrics.routing_load_example) {
                    return (false, None);
                }
            }
            Step::Check => {
                if !self.click_visible_smoke_control(ctx, self.metrics.check) {
                    return (false, None);
                }
            }
            Step::ExamplePass => {
                let expected_profile = ctx
                    .data(|d| {
                        d.get_temp::<serde_json::Value>(egui::Id::new(
                            "private-office-before-practice-full",
                        ))
                    })
                    .unwrap();
                assert_eq!(
                    without_practice(&self.progress),
                    expected_profile,
                    "Practice preserves whole Progress and genuine receipts except its own separate learning state"
                );
                let a = &self.progress.attempts[ID];
                let lesson = self.scenarios[i].ipv4_routing_teaching.as_ref().unwrap();
                let l = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .routing_v4_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(
                    l.checks(lesson).iter().map(|x| x.0).collect::<Vec<_>>(),
                    vec![true; 4]
                );
                assert_eq!(a.state, self.scenarios[i].initial);
                assert!(!progress::current_solved(a, &self.scenarios[i]));
            }
            Step::ExampleFlags(expected) => {
                let expected_profile = ctx
                    .data(|d| {
                        d.get_temp::<serde_json::Value>(egui::Id::new(
                            "private-office-before-practice-full",
                        ))
                    })
                    .unwrap();
                assert_eq!(
                    without_practice(&self.progress),
                    expected_profile,
                    "Practice preserves whole Progress and genuine receipts except its own separate learning state"
                );
                let a = &self.progress.attempts[ID];
                let l = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .routing_v4_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(
                    l.checks(self.scenarios[i].ipv4_routing_teaching.as_ref().unwrap())
                        .iter()
                        .map(|r| r.0)
                        .collect::<Vec<_>>(),
                    *expected
                );
                assert_eq!(
                    a.state, self.scenarios[i].initial,
                    "Real practice edit/test never alters assignment state"
                );
                assert!(!progress::current_solved(a, &self.scenarios[i]));
            }
            Step::Assignment => {
                if !self.click_visible_smoke_control(ctx, self.metrics.assignment_action) {
                    return (false, None);
                }
            }
            Step::Addresses => {
                if !self.click_visible_smoke_control(ctx, self.metrics.routing_address_tab) {
                    return (false, None);
                }
                let selected = self.selected.unwrap();
                let draft = self.progress.attempts[&self.scenarios[selected].id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .routing_v4_draft
                    .as_ref()
                    .unwrap();
                assert!(
                    !draft.show_routes,
                    "Real Addresses & interfaces tab selected"
                );
            }
            Step::Device => {
                if !self.click_visible_smoke_control(ctx, self.metrics.routing_device) {
                    return (false, None);
                }
            }
            Step::Choose(name) => {
                let phase = ctx
                    .data(|d| {
                        d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage)))
                    })
                    .unwrap_or(0);
                let rect = if phase >= 2 {
                    None
                } else {
                    Some(
                        self.metrics
                            .routing_device_choices
                            .iter()
                            .find(|(s, _)| s == name)
                            .expect("Real open dropdown choice before dispatch")
                            .1,
                    )
                };
                if !self.click_smoke_control(ctx, rect) {
                    return (false, None);
                }
                let workspace = self.progress.attempts[ID].workspace.as_ref().unwrap();
                let selected = if view(self) == teaching::View::Example {
                    &workspace
                        .routing_v4_learning
                        .as_ref()
                        .unwrap()
                        .draft
                        .selected_device
                } else {
                    &workspace.routing_v4_draft.as_ref().unwrap().selected_device
                };
                assert_eq!(
                    selected, name,
                    "Native choice actually changed selected device"
                );
            }
            Step::Focus => {
                if !self.click_visible_smoke_control(ctx, self.metrics.inputs.first().copied()) {
                    return (false, None);
                }
            }
            Step::Type(text) => {
                assert_eq!(
                    ctx.memory(|m| m.focused()),
                    self.metrics.focus,
                    "Actual address-field focus before typing"
                );
                self.pending_events.push(egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers {
                        ctrl: true,
                        command: true,
                        ..Default::default()
                    },
                });
                self.pending_events.push(egui::Event::Text(text.clone()));
            }
            Step::Flags(expected) => {
                let a = &self.progress.attempts[ID];
                assert!(self.checked);
                assert_eq!(
                    engine::checks(&a.state, &self.scenarios[i].goal)
                        .iter()
                        .map(|r| r.0)
                        .collect::<Vec<_>>(),
                    *expected
                );
                let solved = expected.iter().all(|x| *x);
                assert_eq!(progress::current_solved(a, &self.scenarios[i]), solved);
                assert_eq!(self.metrics.completion, solved);
            }
            Step::Invalidated => {
                assert!(!self.checked);
                assert!(!self.metrics.completion);
                assert!(!progress::current_solved(
                    &self.progress.attempts[ID],
                    &self.scenarios[i]
                ));
            }
            Step::Malformed => {
                assert!(!self.checked);
                assert!(!self.metrics.completion);
                assert!(
                    self.notice
                        .as_ref()
                        .is_some_and(|s| s.contains("Malformed IPv4 address")),
                    "Native parser feedback must explain the malformed address"
                );
                assert!(!progress::current_solved(
                    &self.progress.attempts[ID],
                    &self.scenarios[i]
                ));
            }
            Step::Warm => {
                let before = full(&self.progress);
                let portable = serde_json::to_value(&self.progress).unwrap();
                self.save();
                assert!(self.storage_error.is_none());
                let raw = std::fs::read(&self.path).unwrap();
                let mut raw_value: serde_json::Value = serde_json::from_slice(&raw).unwrap();
                let receipts = raw_value
                    .as_object_mut()
                    .unwrap()
                    .remove("migration_receipts")
                    .unwrap();
                assert_eq!(
                    raw_value, portable,
                    "Every portable field survives actual save"
                );
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(serde_json::to_value(&loaded).unwrap(), portable);
                assert_eq!(
                    serde_json::to_value(&loaded.migration_receipts).unwrap(),
                    receipts,
                    "Decoded genuine saved receipts, never a synthesized caller field"
                );
                assert_eq!(
                    std::fs::read(&self.path).unwrap(),
                    raw,
                    "Readback does not rewrite progress"
                );
                assert_eq!(full(&self.progress), before, "No loaded Progress installed");
            }
            Step::Back => {
                let key = egui::Id::new("private-office-before-back");
                if ctx.data(|d| d.get_temp::<serde_json::Value>(key)).is_none() {
                    let before = full(&self.progress);
                    ctx.data_mut(|d| d.insert_temp(key, before));
                }
                if !self.click_visible_smoke_control(ctx, self.metrics.back) {
                    return (false, None);
                }
                assert!(self.selected.is_none());
            }
            Step::Resume => {
                if !self.private_office_card_start(ctx, ID) {
                    return (false, None);
                }
                let mut expected = ctx
                    .data(|d| {
                        d.get_temp::<serde_json::Value>(egui::Id::new("private-office-before-back"))
                    })
                    .unwrap();
                let next = expected["clock"].as_u64().unwrap().saturating_add(1);
                expected["clock"] = next.into();
                expected["attempts"][ID]["visited"] = next.into();
                assert_eq!(
                    full(&self.progress),
                    expected,
                    "Real Continue retains entire profile except documented recency"
                );
                assert_eq!(view(self), teaching::View::Assignment);
            }
            Step::Challenge => {
                if !self.click_visible_smoke_control(ctx, self.metrics.mode_challenge) {
                    return (false, None);
                }
                assert!(self.progress.attempts[ID].challenge);
                assert_eq!(view(self), teaching::View::Assignment);
                assert!(!self.help_open);
            }
            Step::Learn => {
                if !self.click_visible_smoke_control(ctx, self.metrics.mode_learn) {
                    return (false, None);
                }
            }
            Step::Settings(scale, light, wide) => {
                self.scale = *scale;
                self.theme_id = if *light { "dlms-light" } else { "dlms-dark" }.into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(*wide));
                if !wide {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(960., 840.)));
                }
                self.lesson_scroll_reset = true;
            }
            Step::Settled => {
                let Step::Settings(scale, light, wide) = steps[index - 1] else {
                    panic!()
                };
                let size = ctx.content_rect().size() * ctx.pixels_per_point();
                let ppp = ctx.pixels_per_point();
                let zoom = ctx.zoom_factor();
                let began = ctx.data_mut(|d| {
                    let k = egui::Id::new(("private-office-settle-start", n));
                    let value = d.get_temp::<Instant>(k).unwrap_or_else(Instant::now);
                    d.insert_temp(k, value);
                    value
                });
                assert!(
                    began.elapsed().as_secs() < 15,
                    "Actual geometry/zoom did not settle"
                );
                if !ppp.is_finite()
                    || ppp <= 0.
                    || !zoom.is_finite()
                    || (zoom - scale).abs() > 0.001
                    || ctx.input(|x| x.viewport().fullscreen) != Some(wide)
                {
                    ctx.request_repaint();
                    return (false, None);
                }
                if !self.case_lesson_geometry(ctx) {
                    return (false, None);
                }
                assert!(
                    size.y > 400. && size.x > 400.,
                    "Observed usable native dimensions"
                );
                let key = egui::Id::new(("private-office-paired-narrow", scale.to_bits(), light));
                if wide {
                    let narrow = ctx
                        .data(|d| d.get_temp::<f32>(key))
                        .expect("Actually observed paired narrow window first");
                    if size.x <= narrow * 1.25 {
                        ctx.request_repaint();
                        return (false, None);
                    }
                } else {
                    if size.x > 1200. {
                        ctx.request_repaint();
                        return (false, None);
                    }
                    ctx.data_mut(|d| d.insert_temp(key, size.x));
                }
                assert_eq!(view(self), teaching::View::Assignment);
            }
            Step::RevealWorkspace => {
                assert_eq!(self.metrics.inputs.len(), 1);
                let field = self.metrics.inputs[0];
                let check = self.metrics.check.expect("Actual Check configuration");
                let clip = self
                    .metrics
                    .main_viewport
                    .expect("Actual page clip")
                    .intersect(ctx.content_rect());
                let union = field.union(check);
                assert!(
                    union.height() <= clip.height(),
                    "Field and Check must fit together without shrinking text"
                );
                if !self.reveal_smoke_control(ctx, union) {
                    return (false, None);
                }
                assert!(
                    smoke_control_visible(field, clip, ctx.pixels_per_point()),
                    "Address field fully inside actual clip after real wheel navigation"
                );
                assert!(
                    smoke_control_visible(check, clip, ctx.pixels_per_point()),
                    "Check fully inside actual clip after real wheel navigation"
                );
                assert!(field.height() >= ctx.style().spacing.interact_size.y);
            }
            Step::Reset => {
                assert!(
                    self.progress.attempts.contains_key(SIBLING),
                    "Actual unrelated edited lab makes retention nonvacuous"
                );
                assert!(self.progress.favorites.contains(SIBLING));
                let key = egui::Id::new(("private-office-reset-before", n));
                if ctx.data(|d| d.get_temp::<serde_json::Value>(key)).is_none() {
                    let v = full(&self.progress);
                    ctx.data_mut(|d| d.insert_temp(key, v));
                }
                if !self.click_visible_smoke_control(ctx, self.metrics.reset) {
                    return (false, None);
                }
                assert!(self.reset_pending);
            }
            Step::Cancel => {
                if !self.click_visible_smoke_control(ctx, self.metrics.reset_cancel) {
                    return (false, None);
                }
                assert!(!self.reset_pending);
                let key = egui::Id::new(("private-office-reset-before", n - 1));
                let before = ctx.data(|d| d.get_temp::<serde_json::Value>(key)).unwrap();
                assert_eq!(full(&self.progress), before);
            }
            Step::Confirm => {
                if !self.click_visible_smoke_control(ctx, self.metrics.confirm_reset) {
                    return (false, None);
                }
                assert!(!self.reset_pending);
            }
            Step::Fresh => {
                let a = &self.progress.attempts[ID];
                assert_eq!(a.state, self.scenarios[i].initial);
                assert!(!progress::current_solved(a, &self.scenarios[i]));
                assert!(!self.metrics.completion);
                let key = egui::Id::new(("private-office-reset-before", n - 2));
                let before = ctx.data(|d| d.get_temp::<serde_json::Value>(key)).unwrap();
                let after = full(&self.progress);
                assert!(
                    reset_preserved(&before, &after, &self.scenarios[i]),
                    "Entire selected reset law, all unrelated progress and receipts intact"
                );
            }
            Step::Done => {
                println!(
                    "PASS normal private-office native: real filtered Start, Introduction/separate example/check/no credit/full assignment, specific mask failure despite traffic, reference/two alternate pools, invalidated edit, complete saved progress/receipt readback without installing it, Challenge, actual five-size/two-theme/two-width assignment matrix, resetCancel/Confirm. Not cold-process, novice, screen-reader or other-OS acceptance."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        (true, None)
    }
}
