//! Same source and unchanged private assertions, without the full library-test binary.
//! The ordinary linked public API is checked too; no copied implementation or host tool.
use dlms_learning_lab::linux_filter;
#[path = "../src/linux_filter_regions.rs"]
mod regions;
#[test]
fn linked_production_comparator_finds_unintended_access_and_accepts_equivalent_splits() {
    use dlms_learning_lab::linux_filter::Verdict;
    use dlms_learning_lab::linux_filter_regions::{self as production, Interval, Rect, RuleSpace};
    let rectangle = |lo, hi| {
        let mut p = [Interval { lo: 0, hi: 0 }; production::DIMENSIONS];
        p[0] = Interval { lo, hi };
        Rect(p)
    };
    let domain = [rectangle(0, 15)];
    let allowed = [rectangle(8, 15)];
    let split = [
        RuleSpace {
            region: Some(rectangle(8, 11)),
            target: Verdict::Accept,
        },
        RuleSpace {
            region: Some(rectangle(12, 15)),
            target: Verdict::Accept,
        },
    ];
    assert_eq!(
        production::compare(&domain, &allowed, &split, Verdict::Drop).unwrap(),
        None
    );
    let broad = [RuleSpace {
        region: Some(rectangle(7, 15)),
        target: Verdict::Accept,
    }];
    let c = production::compare(&domain, &allowed, &broad, Verdict::Drop)
        .unwrap()
        .unwrap();
    assert_eq!(c.point[0], 7);
    assert_eq!(
        (c.actual, c.expected, c.rule),
        (Verdict::Accept, Verdict::Drop, Some(1))
    );
    let local_domain =
        domain.map(|r| regions::Rect(r.0.map(|i| regions::Interval { lo: i.lo, hi: i.hi })));
    let local_allowed =
        allowed.map(|r| regions::Rect(r.0.map(|i| regions::Interval { lo: i.lo, hi: i.hi })));
    let local_rule = regions::RuleSpace {
        region: Some(regions::Rect(
            broad[0]
                .region
                .unwrap()
                .0
                .map(|i| regions::Interval { lo: i.lo, hi: i.hi }),
        )),
        target: Verdict::Accept,
    };
    let identical_source =
        regions::compare(&local_domain, &local_allowed, &[local_rule], Verdict::Drop)
            .unwrap()
            .unwrap();
    assert_eq!(
        (
            identical_source.point,
            identical_source.actual,
            identical_source.expected,
            identical_source.rule
        ),
        (c.point, c.actual, c.expected, c.rule)
    );
}
