//! Widget-level checks for evidence selection and facility access controls.
use super::*;
use dlms_learning_lab::security_investigation::session::Session;

const CHANGE: &str = "ccna-management-change-evidence";
const FACILITY: &str = "ccna-facility-access-plan";

fn open(id: &str) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    app.smoke = false;
    let index = app.scenarios.iter().position(|s| s.id == id).unwrap();
    app.launch(index);
    settle(&ctx, &mut app);
    (ctx, app, index)
}
fn session<'a>(app: &'a LabApp, id: &str) -> &'a Session {
    let State::SecurityInvestigation { simulation } = &app.progress.attempts[id].state else {
        panic!()
    };
    simulation
}
fn paint(ctx: &egui::Context, app: &mut LabApp, events: Vec<egui::Event>) {
    frame(ctx, app, 1500., 10000., events);
}
fn settle(ctx: &egui::Context, app: &mut LabApp) {
    for _ in 0..3 {
        paint(ctx, app, vec![]);
    }
}
fn target(ctx: &egui::Context, app: &LabApp, name: &str) -> (egui::Rect, egui::Rect) {
    let (_, r, clip) = app
        .metrics
        .security_investigation
        .controls
        .iter()
        .find(|r| r.0 == name)
        .unwrap_or_else(|| panic!("Missing {name}"))
        .clone();
    (r, clip.intersect(ctx.content_rect()))
}
fn hit(ctx: &egui::Context, app: &mut LabApp, name: &str) {
    let (r, visible) = target(ctx, app, name);
    assert!(
        smoke::smoke_control_visible(r, visible, ctx.pixels_per_point()),
        "The entire {name} control must have a visible hit target: {r:?} in {visible:?}"
    );
    for pressed in [true, false] {
        paint(
            ctx,
            app,
            vec![
                egui::Event::PointerMoved(r.center()),
                egui::Event::PointerButton {
                    pos: r.center(),
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
    settle(ctx, app);
}
fn input(ctx: &egui::Context, app: &mut LabApp, field: &str, value: &str) {
    hit(ctx, app, field);
    let select = egui::Modifiers {
        command: true,
        ..egui::Modifiers::CTRL
    };
    paint(
        ctx,
        app,
        vec![
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: select,
            },
            egui::Event::Paste(value.into()),
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers: select,
            },
        ],
    );
    settle(ctx, app);
}

fn result(app: &LabApp) -> Option<(usize, usize)> {
    app.metrics
        .security_investigation
        .result
        .map(|(count, total, _)| (count, total))
}

fn select(ctx: &egui::Context, app: &mut LabApp, chooser: &str, option: &str) {
    hit(ctx, app, chooser);
    for _ in 0..20 {
        let (r, visible) = target(ctx, app, option);
        if smoke::smoke_control_visible(r, visible, ctx.pixels_per_point()) {
            hit(ctx, app, option);
            return;
        }
        let wheel = egui::pos2(r.center().x, visible.center().y);
        assert_eq!(
            ctx.layer_id_at(wheel).map(|layer| layer.order),
            Some(egui::Order::Foreground),
            "The scroll must hit the open {chooser} menu"
        );
        paint(
            ctx,
            app,
            vec![
                egui::Event::PointerMoved(wheel),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., (visible.center().y - r.center().y).clamp(-200., 200.)),
                    modifiers: Default::default(),
                },
            ],
        );
        settle(ctx, app);
    }
    panic!("The actual popup item {option} did not scroll fully into view");
}

#[test]
fn change_workspace_requires_evidence_and_applied_admission_policy() {
    let (ctx, mut app, _) = open(CHANGE);
    hit(&ctx, &mut app, "assignment");
    hit(&ctx, &mut app, "check");
    assert_eq!(result(&app), Some((3, 9)));
    select(&ctx, &mut app, "approval", "approval-2");
    select(&ctx, &mut app, "applied-record", "applied-record-4");
    select(&ctx, &mut app, "observation", "observation-6");
    select(&ctx, &mut app, "signin", "signin-2");
    hit(&ctx, &mut app, "check");
    assert_eq!(result(&app), Some((7, 9)));
    let data: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/security_investigation_tasks_v1.json"
    ))
    .unwrap();
    let text = data[CHANGE]["answers"]["assignment"]["contents"]
        .as_str()
        .unwrap();
    input(&ctx, &mut app, "policy", text);
    hit(&ctx, &mut app, "apply-policy");
    hit(&ctx, &mut app, "check");
    assert!(app.metrics.completion);
    select(&ctx, &mut app, "signin", "signin-1");
    assert!(!app.metrics.completion);
    hit(&ctx, &mut app, "check");
    assert_eq!(result(&app), Some((8, 9)));
}
#[test]
fn facility_practice_uses_a_separate_profile_and_requires_legitimate_access() {
    let (ctx, mut app, _) = open(FACILITY);
    let before = session(&app, FACILITY).assignment.clone();
    hit(&ctx, &mut app, "example");
    select(&ctx, &mut app, "technician", "technician-1");
    select(&ctx, &mut app, "visitor", "visitor-3");
    hit(&ctx, &mut app, "record-denials");
    select(&ctx, &mut app, "report-desk", "report-desk-2");
    hit(&ctx, &mut app, "check");
    assert_eq!(result(&app), Some((7, 7)));
    assert!(!app.metrics.completion);
    assert_eq!(session(&app, FACILITY).assignment, before);
    select(&ctx, &mut app, "visitor", "visitor-0");
    hit(&ctx, &mut app, "check");
    assert_eq!(result(&app), Some((6, 7)));
}

#[test]
fn narrow_large_text_wraps_actual_trial_feedback_inside_its_value_column() {
    let (ctx, mut app, _) = open(FACILITY);
    hit(&ctx, &mut app, "assignment");
    app.scale = 2.;
    ctx.set_zoom_factor(2.);
    app.lesson_scroll_reset = true;
    for _ in 0..4 {
        frame(&ctx, &mut app, 946., 6000., vec![]);
    }
    let rows: Vec<_> = app
        .metrics
        .security_investigation
        .controls
        .iter()
        .filter(|(id, _, _)| id.starts_with("facility-trial-text-"))
        .collect();
    assert_eq!(rows.len(), 7);
    let viewport = app
        .metrics
        .main_viewport
        .unwrap()
        .intersect(ctx.content_rect());
    let font = egui::TextStyle::Body.resolve(&ctx.style());
    let (line, tolerance) = (
        ctx.fonts_mut(|fonts| fonts.row_height(&font)),
        1. / ctx.pixels_per_point(),
    );
    for (id, rect, clip) in rows {
        let allocation = clip.intersect(viewport);
        assert!(
            rect.left() >= allocation.left() - tolerance,
            "{id} {rect:?}"
        );
        assert!(
            rect.right() <= allocation.right() + tolerance,
            "{id} {rect:?}"
        );
        assert!(
            rect.height() >= 2. * line - tolerance,
            "Full wrapped feedback for {id}"
        );
    }
}

#[test]
fn credential_controls_verify_both_valid_proofs_and_clear_old_success() {
    let id = "ccna-credential-policy-repair";
    let (ctx, mut app, _) = open(id);
    hit(&ctx, &mut app, "assignment");
    hit(&ctx, &mut app, "check");
    assert_eq!(result(&app), Some((4, 10)));
    input(&ctx, &mut app, "minimum-length", "17");
    paint(&ctx, &mut app, vec![key(egui::Key::Enter, false)]);
    settle(&ctx, &mut app);
    assert!(matches!(
        session(&app, id).assignment.work.current,
        dlms_learning_lab::security_investigation::workspace::Scope::Credentials { policy }
            if policy.minimum_length == 17
    ));
    hit(&ctx, &mut app, "blocklist");
    hit(&ctx, &mut app, "replace");
    select(&ctx, &mut app, "factor", "factor-2");
    hit(&ctx, &mut app, "check");
    assert_eq!(result(&app), Some((10, 10)));
    assert!(app.metrics.completion);
    hit(&ctx, &mut app, "blocklist");
    assert!(!app.metrics.completion);
    hit(&ctx, &mut app, "check");
    assert_eq!(result(&app), Some((9, 10)));
}

#[test]
fn vpn_controls_use_only_the_applied_profile_ranges() {
    let id = "ccna-ipsec-design-review";
    let (ctx, mut app, _) = open(id);
    let before = session(&app, id).assignment.clone();
    hit(&ctx, &mut app, "example");
    select(&ctx, &mut app, "vpn-0-left", "vpn-0-left-0");
    input(&ctx, &mut app, "vpn-0-left-prefix", "10.40.10.0/24");
    input(&ctx, &mut app, "vpn-1-right-prefix", "10.40.20.0/27");
    hit(&ctx, &mut app, "check");
    assert_eq!(result(&app), Some((3, 9)));
    hit(&ctx, &mut app, "apply-vpn");
    hit(&ctx, &mut app, "check");
    assert_eq!(result(&app), Some((9, 9)));
    assert!(!app.metrics.completion);
    assert_eq!(session(&app, id).assignment, before);
    input(&ctx, &mut app, "vpn-1-right-prefix", "10.40.20.9/27");
    assert!(!session(&app, id).example.work.checked);
    hit(&ctx, &mut app, "apply-vpn");
    assert!(
        app.notice
            .as_ref()
            .is_some_and(|text| text.contains("aligned network prefix"))
    );
    hit(&ctx, &mut app, "check");
    assert_eq!(result(&app), Some((8, 9)));
}

#[test]
fn each_credential_and_vpn_practice_step_precedes_its_own_control_group() {
    for (id, keys) in [
        (
            "ccna-credential-policy-repair",
            &[
                "practice-step-1",
                "minimum-length",
                "blocklist",
                "practice-step-2",
                "factor",
                "practice-step-3",
                "replace",
            ][..],
        ),
        (
            "ccna-ipsec-design-review",
            &[
                "practice-step-1",
                "vpn-0-purpose",
                "vpn-0-right-prefix",
                "practice-step-2",
                "vpn-1-purpose",
                "vpn-1-right-prefix",
                "apply-vpn",
                "practice-step-3",
                "vpn-trials",
            ][..],
        ),
    ] {
        let (ctx, mut app, _) = open(id);
        hit(&ctx, &mut app, "example");
        for pair in keys.windows(2) {
            let first = target(&ctx, &app, pair[0]).0;
            let second = target(&ctx, &app, pair[1]).0;
            assert!(
                first.bottom() <= second.top(),
                "{id}: {} must appear before {}",
                pair[0],
                pair[1]
            );
        }
        hit(&ctx, &mut app, "assignment");
        assert!(
            !app.metrics
                .security_investigation
                .controls
                .iter()
                .any(|row| row.0.starts_with("practice-step-")),
            "Assignment must not reveal worked guidance for {id}"
        );
    }
}
