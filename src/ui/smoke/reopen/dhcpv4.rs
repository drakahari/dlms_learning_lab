//! Closed DHCP normal-startup case. All edits/exchanges use existing native input.
use super::*;
use dlms_learning_lab::{dhcpv4::State as Simulation, dhcpv4_workspace::Action as DhcpAction};
use std::collections::BTreeSet;
const LAB: &str = "ccna-dhcp-held-options";
const WORK: &str = "desk-scope";
const SAMPLE_WORK: &str = "gallery-desk-scope";
const BAD_ASSIGNMENT: &str = "unfinished-assignment-duration";
const BAD_EXAMPLE: &str = "unfinished-gallery-router";
pub(super) const CAPTURES: [&str; 12] = [
    "reopen-dhcp-write-library",
    "reopen-dhcp-write-introduction",
    "reopen-dhcp-write-working-example",
    "reopen-dhcp-write-example-draft",
    "reopen-dhcp-write-current-assignment",
    "reopen-dhcp-write-unfinished-assignment",
    "reopen-dhcp-write-saved-appearance",
    "reopen-dhcp-read-loaded-library",
    "reopen-dhcp-read-unfinished-assignment",
    "reopen-dhcp-read-unfinished-example",
    "reopen-dhcp-read-example-repaired",
    "reopen-dhcp-read-assignment-repaired",
];
fn simulation(state: &engine::State) -> &Simulation {
    let engine::State::DhcpV4 { simulation } = state else {
        panic!("canonical DHCP activity")
    };
    simulation
}
fn policy_action(s: &Scenario) -> DhcpAction {
    let a: DhcpAction = serde_json::from_str(
        s.solution[0]
            .strip_prefix("dhcpv4 ")
            .expect("typed authoring bridge only"),
    )
    .unwrap();
    assert!(matches!(a, DhcpAction::ApplyPolicy { .. }));
    a
}
impl LabApp {
    fn reopen_dhcp_private(&self, session: &Session) {
        assert!(!self.smoke && self._smoke_profile.is_none() && self.profile_lock.is_some());
        assert!(!self.read_only && !self.preferences_read_only);
        assert!(self.storage_error.is_none() && self.preferences_error.is_none());
        session.verify_profile_paths(&self.path).unwrap();
        assert_eq!(session.case, Case::DhcpV4);
    }
    fn reopen_dhcp_pending(&self, i: usize) {
        let s = &self.scenarios[i];
        assert_eq!(self.progress.attempts.len(), 1);
        assert_eq!(self.progress.favorites, BTreeSet::from([LAB.to_owned()]));
        let a = &self.progress.attempts[LAB];
        let w = a.workspace.as_ref().unwrap();
        let d = w.dhcpv4_draft.as_ref().unwrap();
        let l = w.dhcpv4_learning.as_ref().unwrap();
        let lesson = s.dhcpv4_teaching.as_ref().unwrap();
        assert_eq!(d.pools[WORK].lease_seconds, BAD_ASSIGNMENT);
        assert_eq!(l.draft.pools[SAMPLE_WORK].router, BAD_EXAMPLE);
        assert!(d.valid() && l.valid_for(lesson));
        assert!(!d.matches(simulation(&a.state)));
        assert!(engine::checks(&a.state, &s.goal).iter().all(|row| row.0));
        assert!(!progress::current_solved(a, s));
        let rows = l.checks(lesson);
        assert!(!rows[0].0);
        assert!(rows[1..].iter().all(|row| row.0));
        assert_eq!(simulation(&a.state).now(), 0);
        assert_eq!(l.simulation.now(), 32);
    }
    fn reopen_dhcp_repaired(&self, i: usize) {
        let s = &self.scenarios[i];
        let a = &self.progress.attempts[LAB];
        let w = a.workspace.as_ref().unwrap();
        let l = w.dhcpv4_learning.as_ref().unwrap();
        assert!(progress::current_solved(a, s));
        assert!(
            w.dhcpv4_draft
                .as_ref()
                .unwrap()
                .matches(simulation(&a.state))
        );
        assert!(
            l.checked
                && l.checks(s.dhcpv4_teaching.as_ref().unwrap())
                    .iter()
                    .all(|r| r.0)
        );
        assert!(self.progress.favorites.contains(LAB));
        assert_eq!(self.theme_id, THEME);
        assert_eq!(self.scale, SCALE);
    }
    pub(super) fn reopen_dhcpv4_tick(&mut self, ctx: &egui::Context, session: &Session) {
        assert!(self.smoke_stage >= 17000);
        let phase = self.smoke_stage - 17000;
        let s = self
            .scenarios
            .iter()
            .find(|s| s.id == LAB)
            .expect("canonical DHCP assignment")
            .clone();
        let i = self.scenarios.iter().position(|s| s.id == LAB).unwrap();
        let lesson = s.dhcpv4_teaching.as_ref().unwrap();
        assert_eq!(s.version, 1);
        assert_eq!(self.scenarios.len(), 171);
        self.reopen_dhcp_private(session);
        let mut shot = None;
        if phase == 0 && !self.reopen_geometry(ctx) {
            return;
        }
        match session.role {
            Role::Write => match phase {
                0 => {
                    assert!(self.progress.attempts.is_empty());
                    self.reopen_filter(LAB);
                    shot = Some("reopen-dhcp-write-library");
                }
                1 => {
                    if ctx
                        .data(|d| {
                            d.get_temp::<u8>(egui::Id::new((
                                "native-click-phases",
                                self.smoke_stage,
                            )))
                        })
                        .unwrap_or(0)
                        < 2
                    {
                        assert_eq!(self.metrics.card_actions.len(), 1);
                    }
                    if !self.click_visible_smoke_control(
                        ctx,
                        self.metrics.card_actions.first().map(|r| r.1),
                    ) {
                        return;
                    }
                }
                2 => {
                    assert!(self.progress.favorites.contains(LAB));
                    if !self.reopen_start_card(ctx, LAB) {
                        return;
                    }
                }
                3 => {
                    assert_eq!(self.selected, Some(i));
                    assert_eq!(self.dhcpv4_view(i), teaching::View::Introduction);
                    shot = Some("reopen-dhcp-write-introduction");
                }
                4 => {
                    if !self.click_visible_smoke_control(ctx, self.metrics.example_action) {
                        return;
                    }
                }
                5 => {
                    if !self.click_visible_smoke_control(ctx, self.metrics.insert) {
                        return;
                    }
                }
                6 => {
                    let a = &self.progress.attempts[LAB];
                    let l = a
                        .workspace
                        .as_ref()
                        .unwrap()
                        .dhcpv4_learning
                        .as_ref()
                        .unwrap();
                    assert_eq!(l.simulation, lesson.fixture);
                    assert!(!l.draft.matches(&l.simulation));
                    assert_eq!(a.state, s.initial);
                }
                7 => {
                    let actions: Vec<_> = lesson
                        .example_actions
                        .iter()
                        .map(|a| a.action.clone())
                        .collect();
                    if !self.dhcp_lessons_actions(ctx, i, true, &actions, true) {
                        return;
                    }
                }
                8 | 12 | 16 | 20 => {
                    if !self.click_visible_smoke_control(ctx, self.metrics.check) {
                        return;
                    }
                }
                9 => {
                    if !self.dhcp_lessons_feedback(ctx, i, true, true) {
                        return;
                    }
                    shot = Some("reopen-dhcp-write-working-example");
                }
                10 => {
                    if !self.dhcp_lessons_field(ctx, "router") {
                        return;
                    }
                }
                11 => self.dhcp_lessons_type(BAD_EXAMPLE),
                13 => {
                    let l = self.progress.attempts[LAB]
                        .workspace
                        .as_ref()
                        .unwrap()
                        .dhcpv4_learning
                        .as_ref()
                        .unwrap();
                    assert_eq!(l.draft.pools[SAMPLE_WORK].router, BAD_EXAMPLE);
                    let rows = l.checks(lesson);
                    assert!(!rows[0].0 && rows[1..].iter().all(|r| r.0));
                    if !self.dhcp_lessons_feedback(ctx, i, true, false) {
                        return;
                    }
                    shot = Some("reopen-dhcp-write-example-draft");
                }
                14 => {
                    if !self
                        .click_visible_smoke_control(ctx, self.metrics.lesson_tabs.get(2).copied())
                    {
                        return;
                    }
                }
                15 => {
                    let actions = [
                        policy_action(&s),
                        DhcpAction::Renew {
                            client: "desk-editor".into(),
                        },
                    ];
                    if !self.dhcp_lessons_actions(ctx, i, false, &actions, false) {
                        return;
                    }
                }
                17 => {
                    if !self.dhcp_lessons_feedback(ctx, i, false, true) {
                        return;
                    }
                    assert_eq!(simulation(&self.progress.attempts[LAB].state).now(), 0);
                    shot = Some("reopen-dhcp-write-current-assignment");
                }
                18 => {
                    if !self.dhcp_lessons_field(ctx, "lease-seconds") {
                        return;
                    }
                }
                19 => self.dhcp_lessons_type(BAD_ASSIGNMENT),
                21 => {
                    self.reopen_dhcp_pending(i);
                    if !self.dhcp_lessons_feedback(ctx, i, false, false) {
                        return;
                    }
                    shot = Some("reopen-dhcp-write-unfinished-assignment");
                }
                22 => {
                    if !self.reopen_popup(ctx, self.metrics.reopen_scale) {
                        return;
                    }
                }
                23 => {
                    let r = self
                        .metrics
                        .reopen_scale_items
                        .iter()
                        .find(|(v, _)| *v == SCALE)
                        .map(|(_, r)| *r);
                    if !self.reopen_popup(ctx, r) {
                        return;
                    }
                }
                24 => {
                    if !self.reopen_popup(ctx, self.metrics.reopen_theme) {
                        return;
                    }
                }
                25 => {
                    let r = self
                        .metrics
                        .reopen_theme_items
                        .iter()
                        .find(|(id, _)| id == THEME)
                        .map(|(_, r)| *r);
                    if !self.reopen_popup(ctx, r) {
                        return;
                    }
                }
                26 => {
                    if !self.reopen_geometry(ctx) {
                        return;
                    }
                }
                27 => {
                    self.reopen_dhcp_pending(i);
                    assert_eq!(self.theme_id, THEME);
                    assert_eq!(self.scale, SCALE);
                    assert!(self.preferences_error.is_none());
                    shot = Some("reopen-dhcp-write-saved-appearance");
                }
                28 => {
                    let key = egui::Id::new("reopen-dhcp-writer-finished");
                    if !ctx.data(|d| d.get_temp::<bool>(key)).unwrap_or(false) {
                        self.reopen_dhcp_pending(i);
                        self.save();
                        assert!(!self.dirty && self.storage_error.is_none());
                        let (observed, raw, prefs) = session.observed(self);
                        session
                            .write_raw("writer-progress.raw", &raw, 16_000_000)
                            .unwrap();
                        session
                            .write_raw("writer-preferences.raw", &prefs, 4096)
                            .unwrap();
                        session.write("writer-observed.json", &observed).unwrap();
                        ctx.data_mut(|d| d.insert_temp(key, true));
                        println!(
                            "PASS native reopen writer: actual DHCP exchanges and two distinct unfinished policy drafts saved with favorite and appearance"
                        );
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    return;
                }
                _ => panic!("closed DHCP writer phase{phase}"),
            },
            Role::Read => match phase {
                0 => {
                    let expected: Observed = serde_json::from_slice(
                        &bounded_file(&session.root.join("writer-observed.json"), JSON_CAP)
                            .unwrap(),
                    )
                    .unwrap();
                    assert_eq!(
                        bounded_file(&self.path, 16_000_000).unwrap(),
                        bounded_file(&session.root.join("writer-progress.raw"), 16_000_000)
                            .unwrap()
                    );
                    assert_eq!(
                        bounded_file(&self.path.with_file_name("preferences-v1.json"), 4096)
                            .unwrap(),
                        bounded_file(&session.root.join("writer-preferences.raw"), 4096).unwrap()
                    );
                    assert_eq!(
                        serde_json::to_value(&self.progress).unwrap(),
                        expected.progress
                    );
                    assert_eq!(
                        serde_json::json!({"version":1,"theme":self.theme_id,"text_scale":self.scale}),
                        expected.preferences
                    );
                    self.reopen_dhcp_pending(i);
                    let raw = bounded_file(&self.path, 16_000_000).unwrap();
                    let prefs = theme::Preferences {
                        version: 1,
                        theme: self.theme_id.clone(),
                        text_scale: self.scale,
                    };
                    let loaded =
                        observation_views(&self.path, &raw, &self.progress, &prefs).unwrap();
                    assert_eq!(loaded, expected);
                    session.write("reader-loaded.json", &loaded).unwrap();
                    shot = Some("reopen-dhcp-read-loaded-library");
                }
                1 => self.reopen_filter(LAB),
                2 => {
                    if !self.reopen_start_card(ctx, LAB) {
                        return;
                    }
                }
                3 => {
                    assert_eq!(self.selected, Some(i));
                    assert_eq!(self.dhcpv4_view(i), teaching::View::Assignment);
                    self.reopen_dhcp_pending(i);
                    shot = Some("reopen-dhcp-read-unfinished-assignment");
                }
                4 | 9 => {
                    if !self.click_visible_smoke_control(
                        ctx,
                        self.metrics
                            .lesson_tabs
                            .get(if phase == 4 { 1 } else { 2 })
                            .copied(),
                    ) {
                        return;
                    }
                }
                5 => {
                    assert_eq!(self.dhcpv4_view(i), teaching::View::Example);
                    self.reopen_dhcp_pending(i);
                    shot = Some("reopen-dhcp-read-unfinished-example");
                }
                6 => {
                    if !self.dhcp_lessons_actions(
                        ctx,
                        i,
                        true,
                        &[lesson.example_actions[0].action.clone()],
                        false,
                    ) {
                        return;
                    }
                }
                7 | 11 => {
                    if !self.click_visible_smoke_control(ctx, self.metrics.check) {
                        return;
                    }
                }
                8 => {
                    if !self.dhcp_lessons_feedback(ctx, i, true, true) {
                        return;
                    }
                    shot = Some("reopen-dhcp-read-example-repaired");
                }
                10 => {
                    if !self.dhcp_lessons_actions(ctx, i, false, &[policy_action(&s)], false) {
                        return;
                    }
                }
                12 => {
                    self.reopen_dhcp_repaired(i);
                    let expected: Observed = serde_json::from_slice(
                        &bounded_file(&session.root.join("writer-observed.json"), JSON_CAP)
                            .unwrap(),
                    )
                    .unwrap();
                    let actual = serde_json::to_value(&self.progress).unwrap();
                    for pointer in [
                        "/attempts/ccna-dhcp-held-options/state/simulation/leases",
                        "/attempts/ccna-dhcp-held-options/workspace/dhcpv4_learning/simulation/leases",
                    ] {
                        assert!(
                            actual.pointer(pointer).is_some(),
                            "existing held-grant wire field"
                        );
                        assert!(expected.progress.pointer(pointer).is_some());
                        assert_eq!(
                            actual.pointer(pointer),
                            expected.progress.pointer(pointer),
                            "policy repair must not invent a refreshed held grant"
                        );
                    }
                }
                13 => {
                    if !self.dhcp_lessons_feedback(ctx, i, false, true) {
                        return;
                    }
                    shot = Some("reopen-dhcp-read-assignment-repaired");
                }
                14 => {
                    let key = egui::Id::new("reopen-dhcp-reader-finished");
                    if !ctx.data(|d| d.get_temp::<bool>(key)).unwrap_or(false) {
                        self.reopen_dhcp_repaired(i);
                        self.save();
                        assert!(!self.dirty && self.storage_error.is_none());
                        let (observed, raw, prefs) = session.observed(self);
                        session
                            .write_raw("reader-progress.raw", &raw, 16_000_000)
                            .unwrap();
                        session
                            .write_raw("reader-preferences.raw", &prefs, 4096)
                            .unwrap();
                        session.write("reader-repaired.json", &observed).unwrap();
                        ctx.data_mut(|d| d.insert_temp(key, true));
                        println!(
                            "PASS native reopen reader: exact normal cold-loaded DHCP state/drafts/favorite/appearance and actual field repairs with current checks"
                        );
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    return;
                }
                _ => panic!("closed DHCP reader phase{phase}"),
            },
        }
        if let Some(name) = shot {
            self.capture_pending = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                name.to_owned(),
            )));
        }
        self.smoke_stage += 1;
        self.started = Instant::now();
    }
}
