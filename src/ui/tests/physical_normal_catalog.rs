//! Normal bundled physical unit navigation; no injected Scenario/Progress/results.
//! Setup configurations use the same typed user writers; UI actions use real events.
use super::*;
use dlms_learning_lab::{
    physical_link as core, physical_link_workspace as work, script_teaching::View,
};
const IDS: [&str; 4] = [
    "ccna-physical-media-span",
    "ccna-physical-duplex-diagnosis",
    "ccna-physical-counter-baseline",
    "ccna-physical-uplink-handover",
];
fn review() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../tests/fixtures/physical-link-author-review.json"
    ))
    .unwrap()
}
fn bring(
    ctx: &egui::Context,
    a: &mut LabApp,
    rect: impl Fn(&LabApp) -> Option<egui::Rect>,
) -> egui::Rect {
    let mut stable = None;
    for _ in 0..20 {
        let r = rect(a).expect("Actual normal navigation control");
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(r, root, ctx.pixels_per_point()) {
            if stable == Some((r, root)) {
                return r;
            }
            stable = Some((r, root));
            frame(ctx, a, 1200., 850., vec![]);
            continue;
        }
        stable = None;
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(egui::pos2(
                    root.left() + 1. / ctx.pixels_per_point(),
                    root.center().y,
                )),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(
                        0.,
                        (root.center().y - r.center().y)
                            .clamp(-root.height() * 0.75, root.height() * 0.75),
                    ),
                    modifiers: Default::default(),
                },
            ],
        );
    }
    panic!("Control was not fully visible after20 real wheel attempts")
}
fn open_unit() -> (egui::Context, LabApp) {
    let (ctx, mut a) = app();
    assert_eq!((a.scenarios.len(), a.units.len()), (176, 46));
    assert_eq!(
        a.scenarios[171..175]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        IDS
    );
    a.filter.track = "CCNA".into();
    a.filter.unit = "physical-uplink-diagnosis".into();
    draw(&ctx, &mut a);
    assert_eq!(
        a.metrics.card_actions.len(),
        4,
        "Selected unit displays its four real lab cards"
    );
    let r = bring(&ctx, &mut a, |a| {
        a.metrics.card_actions.first().map(|r| r.0)
    });
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    assert_eq!(a.scenarios[a.selected.unwrap()].id, IDS[0]);
    (ctx, a)
}
fn apply_reference(a: &mut LabApp, n: usize, field: &str) {
    let i = a.scenarios.iter().position(|s| s.id == IDS[n]).unwrap();
    let s = a.scenarios[i].clone();
    let data = review();
    let row = &data["tasks"][n];
    let configuration: core::Configuration = serde_json::from_value(if field == "reference" {
        row["references"][0].clone()
    } else {
        row[field]["configuration"].clone()
    })
    .unwrap();
    let p = a.progress.attempts.get_mut(&s.id).unwrap();
    work::set_view(p, &s, View::Assignment).unwrap();
    let mut draft = p
        .workspace
        .as_ref()
        .unwrap()
        .physical_link_draft
        .as_ref()
        .unwrap()
        .clone();
    draft.configuration = configuration;
    work::set_draft(p, &s, work::Scope::Assignment, &draft).unwrap();
    work::apply(p, &s, work::Scope::Assignment).unwrap();
    work::set_checked(p, &s, work::Scope::Assignment, true).unwrap();
}
#[test]
fn normal_unit_entry_and_real_next_action_preserve_prior_work_and_start_next_introduction() {
    let (ctx, mut a) = open_unit();
    assert!(
        a.metrics
            .physical
            .tabs
            .iter()
            .any(|(v, _)| *v == View::Introduction)
    );
    assert!(!a.metrics.physical.complete);
    assert!(a.metrics.next_unit.is_none());
    apply_reference(&mut a, 0, "reference");
    draw(&ctx, &mut a);
    assert!(a.metrics.physical.complete);
    let first = serde_json::to_value(&a.progress.attempts[IDS[0]]).unwrap();
    let r = bring(&ctx, &mut a, |a| a.metrics.next_unit);
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    assert_eq!(a.scenarios[a.selected.unwrap()].id, IDS[1]);
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[IDS[0]]).unwrap(),
        first
    );
    let current = &a.progress.attempts[IDS[1]];
    assert_eq!(
        current
            .workspace
            .as_ref()
            .unwrap()
            .physical_link_learning
            .as_ref()
            .unwrap()
            .view,
        View::Introduction
    );
    assert!(!progress::current_solved(
        current,
        &a.scenarios[a.selected.unwrap()]
    ));
}
#[test]
fn wrong_current_configuration_cannot_show_next_even_with_a_checked_marker_and_resume_keeps_draft()
{
    let (ctx, mut a) = open_unit();
    apply_reference(&mut a, 0, "wrong");
    draw(&ctx, &mut a);
    assert!(!a.metrics.physical.complete);
    assert!(a.metrics.next_unit.is_none());
    let before = serde_json::to_value(&a.progress.attempts[IDS[0]]).unwrap();
    let back = bring(&ctx, &mut a, |a| a.metrics.back);
    click(&ctx, &mut a, back);
    draw(&ctx, &mut a);
    assert!(a.selected.is_none());
    assert_eq!(a.metrics.card_actions.len(), 4);
    let resume = bring(&ctx, &mut a, |a| {
        a.metrics.card_actions.first().map(|r| r.0)
    });
    click(&ctx, &mut a, resume);
    draw(&ctx, &mut a);
    assert_eq!(a.scenarios[a.selected.unwrap()].id, IDS[0]);
    let mut retained = serde_json::to_value(&a.progress.attempts[IDS[0]]).unwrap();
    retained["visited"] = before["visited"].clone();
    assert_eq!(retained, before);
    assert!(!a.metrics.physical.complete);
    assert!(a.metrics.next_unit.is_none());
}
