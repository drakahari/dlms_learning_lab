//! Normal registered lessons driven by egui events; no native-window claim or solved-state installation.
use super::*;
use dlms_learning_lab::{
    nat::{Action, Configuration},
    nat_workspace as work,
};
const IDS: [&str; 3] = ["ccna-nat-static", "ccna-nat-pool", "ccna-nat-handover"];
fn learning<'a>(a: &'a LabApp, id: &str) -> &'a dlms_learning_lab::nat_teaching::Learning {
    a.progress.attempts[id]
        .workspace
        .as_ref()
        .unwrap()
        .nat_learning
        .as_ref()
        .unwrap()
}
fn draft<'a>(a: &'a LabApp, id: &str, example: bool) -> &'a work::Draft {
    let w = a.progress.attempts[id].workspace.as_ref().unwrap();
    if example {
        &w.nat_learning.as_ref().unwrap().draft
    } else {
        w.nat_draft.as_ref().unwrap()
    }
}
fn settled(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..60 {
        frame(ctx, a, 1200., 850., vec![]);
    }
}
fn outer(
    ctx: &egui::Context,
    a: &mut LabApp,
    target: impl Fn(&Metrics) -> egui::Rect,
) -> egui::Rect {
    for n in 0..=20 {
        let r = target(&a.metrics);
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if root.contains(r.center()) && root.expand(1. / ctx.pixels_per_point()).contains_rect(r) {
            return r;
        }
        assert!(n < 20, "Outer control not visible {r:?} {root:?}");
        let y = if r.bottom() > root.bottom() {
            root.bottom() - r.bottom()
        } else {
            root.top() - r.top()
        };
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(egui::pos2(root.left() + 1., root.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., y.clamp(-300., 300.)),
                    modifiers: Default::default(),
                },
            ],
        );
        settled(ctx, a);
    }
    unreachable!()
}
fn locate(ctx: &egui::Context, a: &mut LabApp, key: &str, field: bool) -> egui::Rect {
    let mut trace = std::collections::VecDeque::with_capacity(3);
    for n in 0..=20 {
        let (r, clip) = if field {
            let r = a
                .metrics
                .nat_fields
                .iter()
                .find(|r| r.0 == key)
                .expect("actual field");
            (r.1, r.4)
        } else {
            let r = a
                .metrics
                .nat_controls
                .iter()
                .find(|r| r.0 == key)
                .expect("actual control");
            (r.1, r.3)
        };
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let child = if key.starts_with("static:") || key.starts_with("remove:") {
            a.metrics.nat_static_viewport
        } else {
            None
        };
        let clip = child.map_or(clip, |c| c.intersect(clip));
        let v = clip.intersect(root);
        if v.contains(r.center()) && v.expand(1. / ctx.pixels_per_point()).contains_rect(r) {
            return r;
        }
        // Bring the actual, uncut child viewport into the root first. A field
        // can be inside the root while still below the child's own clip; using
        // that field for an outer correction moves the child in the wrong way.
        let child_clipped = child.is_some_and(|c| !root.contains_rect(c));
        let inner = child.is_some() && !child_clipped;
        let viewport = if inner { v } else { root };
        let target = if child_clipped { child.unwrap() } else { r };
        let y = if target.bottom() > viewport.bottom() {
            viewport.bottom() - target.bottom()
        } else if target.top() < viewport.top() {
            viewport.top() - target.top()
        } else {
            0.
        };
        let pointer = if inner {
            v.center()
        } else {
            egui::pos2(root.left() + 1., root.center().y)
        };
        if trace.len() == 3 {
            trace.pop_front();
        }
        trace.push_back(format!("try{n} field{r:?} raw_child{child:?} root{root:?} clip{clip:?} visible{v:?} target{target:?} inner{inner} pointer{pointer:?} wheel{:.2} main{:?} child_offset_content{:?} ppp{:.3}",y.clamp(-250.,250.),a.metrics.main_scroll_offset,a.metrics.nat_static_scroll,ctx.pixels_per_point()));
        assert!(n < 20, "NAT target {key} remains clipped; last3 {trace:?}");
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(pointer),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., y.clamp(-250., 250.)),
                    modifiers: Default::default(),
                },
            ],
        );
        settled(ctx, a);
    }
    unreachable!()
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = locate(ctx, a, key, false);
    click(ctx, a, r);
}
fn input(ctx: &egui::Context, a: &mut LabApp, key: &str, value: &str) {
    let r = locate(ctx, a, key, true);
    click(ctx, a, r);
    a.focus_input = false;
    let id = a.metrics.nat_fields.iter().find(|r| r.0 == key).unwrap().2;
    assert!(ctx.memory(|m| m.has_focus(id)));
    frame(
        ctx,
        a,
        1200.,
        850.,
        vec![acl_ctrl(egui::Key::A), egui::Event::Text(value.into())],
    );
    draw(ctx, a);
}
fn choose(ctx: &egui::Context, a: &mut LabApp, key: &str, value: &str) {
    tap(ctx, a, key);
    for n in 0..=20 {
        let r = a
            .metrics
            .nat_choices
            .iter()
            .find(|r| r.0 == format!("{key}:{value}"))
            .expect("actual option")
            .1;
        let clip = a.metrics.nat_popup.unwrap().intersect(ctx.content_rect());
        if clip.contains(r.center()) && clip.expand(1. / ctx.pixels_per_point()).contains_rect(r) {
            click(ctx, a, r);
            return;
        }
        assert!(n < 20, "actual popup option clipped");
        let y = if r.bottom() > clip.bottom() {
            clip.bottom() - r.bottom()
        } else {
            clip.top() - r.top()
        };
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(clip.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., y.clamp(-200., 200.)),
                    modifiers: Default::default(),
                },
            ],
        );
        settled(ctx, a);
    }
    unreachable!()
}
fn tab(ctx: &egui::Context, a: &mut LabApp, index: usize) {
    let r = outer(ctx, a, |m| m.lesson_tabs[index]);
    click(ctx, a, r);
}
fn opened(id: &str) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    assert_eq!((a.scenarios.len(), a.units.len()), (143, 36));
    assert!(a.progress.attempts.is_empty());
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.filter.search = a.scenarios[i].title.clone();
    draw(&ctx, &mut a);
    assert_eq!(a.metrics.card_actions.len(), 1);
    let r = outer(&ctx, &mut a, |m| m.card_actions[0].1);
    click(&ctx, &mut a, r);
    assert!(a.progress.favorites.contains(id));
    let r = outer(&ctx, &mut a, |m| m.card_actions[0].0);
    click(&ctx, &mut a, r);
    assert_eq!(a.selected, Some(i));
    assert_eq!(learning(&a, id).view, teaching::View::Introduction);
    (ctx, a, i)
}
fn row_key(a: &LabApp, id: &str, example: bool, inside: &str, part: &str) -> String {
    let index = draft(a, id, example)
        .statics
        .iter()
        .position(|r| r.inside == inside)
        .unwrap();
    let key = &a
        .metrics
        .nat_fields
        .iter()
        .filter(|r| r.0.starts_with("static:") && r.0.ends_with(":inside"))
        .nth(index)
        .unwrap()
        .0;
    format!("{}:{part}", key.strip_suffix(":inside").unwrap())
}
fn configure(ctx: &egui::Context, a: &mut LabApp, id: &str, example: bool, c: &Configuration) {
    let removed = draft(a, id, example)
        .statics
        .iter()
        .filter(|r| !c.statics.contains_key(&r.inside))
        .map(|r| r.inside.clone())
        .collect::<Vec<_>>();
    for inside in removed {
        let key = row_key(a, id, example, &inside, "inside");
        let token = key.split(':').nth(1).unwrap();
        let before = draft(a, id, example).statics.len();
        tap(ctx, a, &format!("remove:{token}"));
        assert_eq!(draft(a, id, example).statics.len() + 1, before);
    }
    for (inside, global) in &c.statics {
        if !draft(a, id, example)
            .statics
            .iter()
            .any(|r| r.inside == *inside)
        {
            let before = draft(a, id, example).statics.len();
            tap(ctx, a, "add-static");
            assert_eq!(draft(a, id, example).statics.len(), before + 1);
            let key = a
                .metrics
                .nat_fields
                .iter()
                .rfind(|r| r.0.starts_with("static:") && r.0.ends_with(":inside"))
                .unwrap()
                .0
                .clone();
            input(ctx, a, &key, inside);
        }
        let key = row_key(a, id, example, inside, "global");
        input(ctx, a, &key, &global.to_string());
    }
    if draft(a, id, example).dynamic_enabled != c.dynamic.is_some() {
        tap(ctx, a, "dynamic");
    }
    if let Some(r) = c.dynamic {
        for (key, value) in [
            ("network", r.source.network.to_string()),
            ("prefix", r.source.length.to_string()),
            ("first", r.pool.first.to_string()),
            ("last", r.pool.last.to_string()),
        ] {
            input(ctx, a, key, &value);
        }
    }
    tap(ctx, a, "apply");
    assert_eq!(draft(a, id, example).configuration().unwrap(), *c);
}
fn action(ctx: &egui::Context, a: &mut LabApp, id: &str, example: bool, action: &Action) {
    match action {
        Action::Configure { configuration } => configure(ctx, a, id, example, configuration),
        Action::ClearDynamic {} => tap(ctx, a, "clear"),
        Action::Outbound { inside, peer } => {
            choose(ctx, a, "inside", inside);
            choose(ctx, a, "peer", peer);
            tap(ctx, a, "outbound");
        }
        Action::Inbound { peer, global } => {
            choose(ctx, a, "peer", peer);
            input(ctx, a, "inbound-global", &global.to_string());
            tap(ctx, a, "inbound");
        }
    }
}
fn plan(ctx: &egui::Context, a: &mut LabApp, id: &str, example: bool, actions: &[Action]) {
    for step in actions {
        action(ctx, a, id, example, step);
    }
}
fn reference(a: &LabApp, i: usize) -> Vec<Action> {
    a.scenarios[i]
        .solution
        .iter()
        .map(|s| Action::from_json(s.strip_prefix(work::PREFIX).unwrap().as_bytes()).unwrap())
        .collect()
}
#[test]
fn three_normal_starts_intro_separate_practice_adverse_repair_and_no_assignment_credit() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let assignment = a.progress.attempts[id].state.clone();
        let raw = serde_json::to_value(draft(&a, id, false)).unwrap();
        tab(&ctx, &mut a, 1);
        let lesson = a.scenarios[i].nat_teaching.as_ref().unwrap().clone();
        let before = learning(&a, id).simulation.clone();
        let r = outer(&ctx, &mut a, |m| m.insert.unwrap());
        click(&ctx, &mut a, r);
        assert_eq!(learning(&a, id).simulation, before);
        for step in &lesson.example_actions {
            action(&ctx, &mut a, id, true, &step.action);
        }
        tap(&ctx, &mut a, "check");
        assert!(learning(&a, id).checks(&lesson).iter().all(|r| r.0));
        assert!(!a.metrics.completion);
        plan(&ctx, &mut a, id, true, &lesson.try_change.actions);
        tap(&ctx, &mut a, "check");
        assert_eq!(
            learning(&a, id)
                .checks(&lesson)
                .iter()
                .map(|r| r.0)
                .collect::<Vec<_>>(),
            lesson.try_change.expected
        );
        plan(&ctx, &mut a, id, true, &lesson.repair);
        tap(&ctx, &mut a, "check");
        assert!(learning(&a, id).checks(&lesson).iter().all(|r| r.0));
        assert_eq!(a.progress.attempts[id].state, assignment);
        assert_eq!(serde_json::to_value(draft(&a, id, false)).unwrap(), raw);
        tab(&ctx, &mut a, 2);
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
    }
}
#[test]
fn three_full_assignments_partial_reference_alternate_and_broken_again_use_real_events() {
    let review: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/nat-basic-review.json"
    ))
    .unwrap();
    for (id, row) in IDS.into_iter().zip(review.as_array().unwrap()) {
        let (ctx, mut a, i) = opened(id);
        tab(&ctx, &mut a, 2);
        let partial: Vec<Action> =
            serde_json::from_value(row["partial"]["actions"].clone()).unwrap();
        plan(&ctx, &mut a, id, false, &partial);
        tap(&ctx, &mut a, "check");
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        let reference = reference(&a, i);
        plan(&ctx, &mut a, id, false, &reference);
        tap(&ctx, &mut a, "check");
        assert!(progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        assert!(a.metrics.completion);
        let broken: Vec<Action> = serde_json::from_value(row["broken"]["actions"].clone()).unwrap();
        plan(&ctx, &mut a, id, false, &broken);
        tap(&ctx, &mut a, "check");
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        assert!(!a.metrics.completion);
        let alternate: Vec<Action> =
            serde_json::from_value(row["alternatives"][0]["actions"].clone()).unwrap();
        plan(&ctx, &mut a, id, false, &alternate);
        tap(&ctx, &mut a, "check");
        assert!(progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
    }
}
#[test]
fn real_continue_retains_distinct_drafts_reset_cancel_and_confirm_keep_favorite() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        tab(&ctx, &mut a, 1);
        input(&ctx, &mut a, "inbound-global", "unfinished example");
        tab(&ctx, &mut a, 2);
        input(&ctx, &mut a, "inbound-global", "different assignment");
        let saved = serde_json::to_value(&a.progress.attempts[id]).unwrap();
        let r = outer(&ctx, &mut a, |m| m.back.unwrap());
        click(&ctx, &mut a, r);
        assert!(a.selected.is_none());
        assert_eq!(a.metrics.card_actions.len(), 1);
        let r = outer(&ctx, &mut a, |m| m.card_actions[0].0);
        click(&ctx, &mut a, r);
        let mut expected = saved;
        expected["visited"] = serde_json::json!(a.progress.attempts[id].visited);
        assert!(serde_json::to_value(&a.progress.attempts[id]).unwrap() == expected);
        let before = serde_json::to_value(&a.progress.attempts[id]).unwrap();
        let r = outer(&ctx, &mut a, |m| m.reset.unwrap());
        click(&ctx, &mut a, r);
        let r = outer(&ctx, &mut a, |m| m.reset_cancel.unwrap());
        click(&ctx, &mut a, r);
        assert!(serde_json::to_value(&a.progress.attempts[id]).unwrap() == before);
        let r = outer(&ctx, &mut a, |m| m.reset.unwrap());
        click(&ctx, &mut a, r);
        let r = outer(&ctx, &mut a, |m| m.confirm_reset.unwrap());
        click(&ctx, &mut a, r);
        assert_eq!(a.progress.attempts[id].state, a.scenarios[i].initial);
        assert_eq!(learning(&a, id).view, teaching::View::Introduction);
        assert!(a.progress.favorites.contains(id));
        assert_eq!(draft(&a, id, false).inbound_global, "");
        assert_eq!(draft(&a, id, true).inbound_global, "");
    }
}
