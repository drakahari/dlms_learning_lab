//! Closed native address-filter workflow; uses the shared real filter controls.
use super::sieve::Op;
use super::*;
use dlms_learning_lab::sieve_source as binding;
use std::sync::OnceLock;
const IDS: [&str; 2] = [
    "lpic2-sieve-sender-domain",
    "lpic2-sieve-recipient-recovery",
];
fn active_plan() -> &'static Vec<Op> {
    static P: OnceLock<Vec<Op>> = OnceLock::new();
    P.get_or_init(|| {
        let labs = pack::load(include_str!(
            "../../../scenarios/sieve-address-lessons-v1.json"
        ))
        .unwrap();
        let mut p = vec![];
        for (n, id) in IDS.iter().enumerate() {
            let source = labs.iter().find(|s| s.id == *id).unwrap();
            let d = binding::source(source).unwrap();
            let tag = |k: &str| format!("sieve-address-{n}-{k}");
            p.extend([
                Op::Style(1., false),
                Op::Open((*id).into()),
                Op::Shot(tag("introduction"), "introduction".into()),
                Op::Shot(tag("worked"), "worked".into()),
                Op::Click("example".into()),
                Op::Shot(tag("practice"), "editor-viewport".into()),
                Op::Choose("sample".into(), d.example.cases.last().unwrap().id.clone()),
                Op::Click("run".into()),
                Op::Check(false, true),
                Op::Shot(tag("practice-failure"), "failed-case".into()),
                Op::Click("fill-example".into()),
                Op::NotRun(true),
                Op::Click("run".into()),
                Op::Check(true, true),
                Op::Shot(tag("practice-complete"), "overall".into()),
                Op::Uncredited,
                Op::Click("assignment".into()),
                Op::Shot(tag("assignment"), "editor-viewport".into()),
                Op::Choose(
                    "sample".into(),
                    d.assignment.cases.last().unwrap().id.clone(),
                ),
                Op::Click("run".into()),
                Op::Check(false, false),
                Op::Shot(tag("failure-detail"), "failed-case".into()),
                Op::Text(
                    "editor".into(),
                    "require \"fileinto\";\nif header :is".into(),
                ),
                Op::Click("run".into()),
                Op::Error(false),
                Op::Shot(tag("syntax-error"), "error".into()),
                Op::Text("editor".into(), d.assignment.reference.clone()),
                Op::RunStop,
                Op::Stopped,
                Op::Click("run".into()),
                Op::Check(true, false),
                Op::Shot(tag("complete"), "overall".into()),
                Op::Save,
                Op::Text("editor".into(), "keep;".into()),
                Op::NotRun(false),
                Op::Click("run".into()),
                Op::Check(false, false),
                Op::Shot(tag("broken-again"), "failed-case".into()),
                Op::Text("editor".into(), d.assignment.reference.clone()),
                Op::Click("run".into()),
                Op::Check(true, false),
                Op::Save,
            ]);
            for (scale, light, label) in [(1.5, true, "light150"), (2., false, "navy200")] {
                p.extend([
                    Op::Style(scale, light),
                    Op::Shot(tag(&format!("{label}-editor")), "editor-viewport".into()),
                    Op::Shot(tag(&format!("{label}-controls")), "run".into()),
                    Op::Shot(tag(&format!("{label}-results")), "overall".into()),
                ]);
            }
            p.extend([
                Op::Style(1., false),
                Op::Click("reset-header".into()),
                Op::Click("confirm-reset".into()),
                Op::Fresh,
            ]);
        }
        p
    })
}
impl LabApp {
    pub(in crate::ui) fn sieve_addresses_tick(&mut self, ctx: &egui::Context) {
        self.sieve_flow(ctx, active_plan());
    }
}
#[test]
fn address_native_plan_exercises_both_real_tasks_and_all_result_states() {
    let p = active_plan();
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
    assert_eq!(p.len(), 108);
    let names = p
        .iter()
        .filter_map(|o| {
            if let Op::Shot(n, _) = o {
                Some(n)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(names.len(), 32);
    assert_eq!(
        names
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        32
    );
    for pair in [(true, true), (false, true), (true, false), (false, false)] {
        assert!(
            p.iter()
                .filter(|o| matches!(o,Op::Check(a,b)if(*a,*b)==pair))
                .count()
                >= 2
        );
    }
    assert_eq!(p.iter().filter(|o| matches!(o, Op::RunStop)).count(), 2);
    assert_eq!(p.iter().filter(|o| matches!(o, Op::Fresh)).count(), 2);
}
