//! Bounded full-width IPv6 header-predicate comparison; not a kernel/router model.
//! Fields: source/destination128-bit address, ingress/egress code, protocol,
//! source/destination port, ICMPv6 type/code. No serde u128 wire fields.
//! Path/connection semantics remain separate; this is not a kernel verifier.
use crate::linux_filter::{Class, Fault, Verdict};
pub const DIMENSIONS: usize = 9;
const MAX_REGIONS: usize = 4096;
pub(crate) const MAX_STEPS: usize = 1_000_000;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Interval {
    pub lo: u128,
    pub hi: u128,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect(pub [Interval; DIMENSIONS]);
impl Rect {
    fn valid(self) -> bool {
        self.0.iter().all(|i| i.lo <= i.hi)
    }
    pub fn contains(self, p: [u128; DIMENSIONS]) -> bool {
        self.0.iter().zip(p).all(|(i, p)| i.lo <= p && p <= i.hi)
    }
    pub(crate) fn intersect(self, b: Self, budget: &mut Budget) -> Result<Option<Self>, Fault> {
        budget.tick()?;
        let mut a = self;
        for (d, i) in a.0.iter_mut().enumerate() {
            i.lo = i.lo.max(b.0[d].lo);
            i.hi = i.hi.min(b.0[d].hi);
            if i.lo > i.hi {
                return Ok(None);
            }
        }
        Ok(Some(a))
    }
    fn subtract(self, b: Self, budget: &mut Budget) -> Result<Vec<Self>, Fault> {
        let Some(cut) = self.intersect(b, budget)? else {
            return Ok(vec![self]);
        };
        let mut rest = self;
        let mut out = vec![];
        for d in 0..DIMENSIONS {
            budget.tick()?;
            if rest.0[d].lo < cut.0[d].lo {
                let mut fragment = rest;
                fragment.0[d].hi = cut.0[d].lo - 1;
                out.push(fragment);
                rest.0[d].lo = cut.0[d].lo;
            }
            if rest.0[d].hi > cut.0[d].hi {
                let mut fragment = rest;
                fragment.0[d].lo = cut.0[d].hi + 1;
                out.push(fragment);
                rest.0[d].hi = cut.0[d].hi;
            }
        }
        debug_assert_eq!(rest, cut);
        Ok(out)
    }
    fn point(self) -> [u128; DIMENSIONS] {
        self.0.map(|i| i.lo)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleSpace {
    pub region: Option<Rect>,
    pub target: Verdict,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Counterexample {
    pub point: [u128; DIMENSIONS],
    pub actual: Verdict,
    pub expected: Verdict,
    pub rule: Option<usize>,
}
pub(crate) struct Budget {
    pub(crate) steps: usize,
}
fn limit() -> Fault {
    Fault{class:Class::Limit,message:"This policy exceeded the bounded comparison profile. Simplify its rules; no success was awarded.".into()}
}
impl Budget {
    fn tick(&mut self) -> Result<(), Fault> {
        self.steps = self.steps.checked_sub(1).ok_or_else(limit)?;
        Ok(())
    }
}
fn push(out: &mut Vec<Rect>, rect: Rect) -> Result<(), Fault> {
    if out.len() >= MAX_REGIONS {
        return Err(limit());
    }
    out.push(rect);
    Ok(())
}
fn difference_union(
    region: Rect,
    allowed: &[Rect],
    budget: &mut Budget,
) -> Result<Vec<Rect>, Fault> {
    let mut rest = vec![region];
    for a in allowed {
        let mut next = vec![];
        for r in rest {
            for piece in r.subtract(*a, budget)? {
                push(&mut next, piece)?;
            }
        }
        rest = next;
        if rest.is_empty() {
            break;
        }
    }
    Ok(rest)
}
fn check(
    region: Rect,
    target: Verdict,
    allowed: &[Rect],
    rule: Option<usize>,
    budget: &mut Budget,
) -> Result<Option<Counterexample>, Fault> {
    let point = if target == Verdict::Accept {
        difference_union(region, allowed, budget)?
            .first()
            .copied()
            .map(Rect::point)
    } else {
        let mut point = None;
        for a in allowed {
            if let Some(r) = region.intersect(*a, budget)? {
                point = Some(r.point());
                break;
            }
        }
        point
    };
    Ok(point.map(|point| Counterexample {
        point,
        actual: target,
        expected: if target == Verdict::Accept {
            Verdict::Drop
        } else {
            Verdict::Accept
        },
        rule,
    }))
}
pub(crate) fn bounded_compare(
    domain: &[Rect],
    allowed: &[Rect],
    rules: &[RuleSpace],
    policy: Verdict,
    budget: &mut Budget,
) -> Result<Option<Counterexample>, Fault> {
    if domain.is_empty()
        || domain.len() > 16
        || allowed.len() > 32
        || rules.len() > 96
        || domain.iter().chain(allowed).any(|r| !r.valid())
        || rules.iter().any(|r| r.region.is_some_and(|r| !r.valid()))
    {
        return Err(Fault {
            class: Class::Context,
            message:
                "Supply bounded valid policy domains, allowed regions and at most96ordered rules."
                    .into(),
        });
    }
    for d in domain {
        let mut rest = vec![*d];
        for (number, rule) in rules.iter().enumerate() {
            let Some(region) = rule.region else {
                continue;
            };
            let mut next = vec![];
            for r in rest {
                if let Some(matched) = r.intersect(region, budget)?
                    && let Some(c) = check(matched, rule.target, allowed, Some(number + 1), budget)?
                {
                    return Ok(Some(c));
                }
                for piece in r.subtract(region, budget)? {
                    push(&mut next, piece)?;
                }
            }
            rest = next;
            if rest.is_empty() {
                break;
            }
        }
        for region in rest {
            if let Some(c) = check(region, policy, allowed, None, budget)? {
                return Ok(Some(c));
            }
        }
    }
    Ok(None)
}
/// Exact comparison over the supplied finite-dimensional domain, with explicit complexity refusal.
pub fn compare(
    domain: &[Rect],
    allowed: &[Rect],
    rules: &[RuleSpace],
    policy: Verdict,
) -> Result<Option<Counterexample>, Fault> {
    bounded_compare(
        domain,
        allowed,
        rules,
        policy,
        &mut Budget { steps: MAX_STEPS },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rect(x: (u128, u128), y: (u128, u128)) -> Rect {
        let mut v = [Interval { lo: 0, hi: 0 }; DIMENSIONS];
        v[0] = Interval { lo: x.0, hi: x.1 };
        v[1] = Interval { lo: y.0, hi: y.1 };
        Rect(v)
    }
    fn point(x: u128, y: u128) -> [u128; DIMENSIONS] {
        let mut v = [0; DIMENSIONS];
        v[0] = x;
        v[1] = y;
        v
    }
    fn actual(rules: &[RuleSpace], policy: Verdict, p: [u128; DIMENSIONS]) -> Verdict {
        rules
            .iter()
            .find(|r| r.region.is_some_and(|r| r.contains(p)))
            .map_or(policy, |r| r.target)
    }
    #[test]
    fn rectangle_subtraction_is_disjoint_exact_and_safe_at_u128_edges() {
        for (a, b) in [
            (rect((0, 15), (0, 15)), rect((3, 8), (5, 9))),
            (rect((0, u128::MAX), (0, 0)), rect((0, u128::MAX), (0, 0))),
            (
                rect((0, u128::MAX), (0, 0)),
                rect((1, u128::MAX - 1), (0, 0)),
            ),
        ] {
            let parts = a.subtract(b, &mut Budget { steps: 100 }).unwrap();
            assert!(parts.len() <= 2 * DIMENSIONS);
            for (i, x) in parts.iter().enumerate() {
                assert!(x.valid());
                assert!(x.intersect(b, &mut Budget { steps: 1 }).unwrap().is_none());
                for y in &parts[i + 1..] {
                    assert!(x.intersect(*y, &mut Budget { steps: 1 }).unwrap().is_none());
                }
            }
            for x in [0, 1, 2, 3, 5, 8, 9, 14, 15, u128::MAX - 1, u128::MAX] {
                for y in [0, 1, 5, 9, 15] {
                    let p = point(x, y);
                    assert_eq!(
                        parts.iter().any(|r| r.contains(p)),
                        a.contains(p) && !b.contains(p)
                    );
                }
            }
        }
    }
    #[test]
    fn first_match_and_equivalent_guarded_or_split_rules_compare_as_behavior() {
        let domain = [rect((0, 15), (0, 15))];
        let allow = [rect((8, 15), (4, 7))];
        let split = vec![
            RuleSpace {
                region: Some(rect((8, 11), (4, 7))),
                target: Verdict::Accept,
            },
            RuleSpace {
                region: Some(rect((12, 15), (4, 7))),
                target: Verdict::Accept,
            },
        ];
        assert_eq!(
            compare(&domain, &allow, &split, Verdict::Drop).unwrap(),
            None
        );
        let guarded = vec![
            RuleSpace {
                region: Some(rect((0, 7), (0, 15))),
                target: Verdict::Drop,
            },
            RuleSpace {
                region: Some(rect((0, 15), (0, 3))),
                target: Verdict::Drop,
            },
            RuleSpace {
                region: Some(rect((0, 15), (8, 15))),
                target: Verdict::Drop,
            },
        ];
        assert_eq!(
            compare(&domain, &allow, &guarded, Verdict::Accept).unwrap(),
            None
        );
        let bad = [
            RuleSpace {
                region: Some(domain[0]),
                target: Verdict::Drop,
            },
            split[0].clone(),
            split[1].clone(),
        ];
        let c = compare(&domain, &allow, &bad, Verdict::Drop)
            .unwrap()
            .unwrap();
        assert_eq!(c.rule, Some(1));
        assert_eq!((c.actual, c.expected), (Verdict::Drop, Verdict::Accept));
        assert!(allow[0].contains(c.point));
        let c = compare(
            &domain,
            &allow,
            &[RuleSpace {
                region: Some(rect((8, 15), (4, 8))),
                target: Verdict::Accept,
            }],
            Verdict::Drop,
        )
        .unwrap()
        .unwrap();
        assert_eq!(c.point[1], 8);
        assert_eq!((c.actual, c.expected), (Verdict::Accept, Verdict::Drop));
    }
    #[test]
    fn deterministic_exhaustive_small_domains_confirm_every_generated_witness_and_no_false_pass() {
        let mut seed = 0x8d12_ef01u128;
        let mut n = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            seed
        };
        for _ in 0..128 {
            let mut random_rect = || {
                let a = n() % 16;
                let b = n() % 16;
                let c = n() % 16;
                let d = n() % 16;
                rect((a.min(b), a.max(b)), (c.min(d), c.max(d)))
            };
            let allowed = (0..3).map(|_| random_rect()).collect::<Vec<_>>();
            let regions = (0..8).map(|_| random_rect()).collect::<Vec<_>>();
            let rules = regions
                .into_iter()
                .map(|region| RuleSpace {
                    region: Some(region),
                    target: if n() % 2 == 0 {
                        Verdict::Accept
                    } else {
                        Verdict::Drop
                    },
                })
                .collect::<Vec<_>>();
            let policy = if n() % 2 == 0 {
                Verdict::Accept
            } else {
                Verdict::Drop
            };
            let mismatch = (0..16).any(|x| {
                (0..16).any(|y| {
                    let p = point(x, y);
                    (actual(&rules, policy, p) == Verdict::Accept)
                        != allowed.iter().any(|r| r.contains(p))
                })
            });
            let c = compare(&[rect((0, 15), (0, 15))], &allowed, &rules, policy).unwrap();
            assert_eq!(c.is_some(), mismatch);
            if let Some(c) = c {
                assert_eq!(actual(&rules, policy, c.point), c.actual);
                assert_eq!(
                    allowed.iter().any(|r| r.contains(c.point)),
                    c.expected == Verdict::Accept
                );
            }
        }
    }
    #[test]
    fn an_exhausted_work_budget_is_an_explicit_limit_not_success() {
        let domain = [rect((0, 15), (0, 15))];
        assert_eq!(
            bounded_compare(
                &domain,
                &domain,
                &[],
                Verdict::Accept,
                &mut Budget { steps: 0 }
            )
            .unwrap_err()
            .class,
            Class::Limit
        );
    }

    #[test]
    fn high_ipv6_bits_are_not_truncated_to_ipv4_and_split_prefixes_remain_equivalent() {
        let base = u128::from("fd20:30:40::".parse::<std::net::Ipv6Addr>().unwrap());
        let domain = rect((base, base + 15), (u128::MAX - 15, u128::MAX));
        let lower = rect((base, base + 7), (u128::MAX - 15, u128::MAX));
        let upper = rect((base + 8, base + 15), (u128::MAX - 15, u128::MAX));
        let mut rules = vec![RuleSpace {
            region: Some(lower),
            target: Verdict::Accept,
        }];
        let mismatch = compare(&[domain], &[domain], &rules, Verdict::Drop)
            .unwrap()
            .unwrap();
        assert!(mismatch.point[0] >= base + 8);
        assert!(mismatch.point[0] > u64::MAX as u128);
        assert!(domain.contains(mismatch.point));
        assert_eq!(actual(&rules, Verdict::Drop, mismatch.point), Verdict::Drop);
        assert_eq!(mismatch.expected, Verdict::Accept);
        rules.push(RuleSpace {
            region: Some(upper),
            target: Verdict::Accept,
        });
        assert!(
            compare(&[domain], &[domain], &rules, Verdict::Drop)
                .unwrap()
                .is_none()
        );
        let truncated = rect(
            ((base as u32) as u128, (base as u32) as u128 + 15),
            (u128::MAX - 15, u128::MAX),
        );
        let wrong = [RuleSpace {
            region: Some(truncated),
            target: Verdict::Accept,
        }];
        assert!(
            compare(&[domain], &[domain], &wrong, Verdict::Drop)
                .unwrap()
                .is_some()
        );
    }
    #[test]
    fn full_width_domain_zero_and_maximum_split_without_success_from_sample_points() {
        let domain = rect((0, u128::MAX), (0, 0));
        let midpoint = 1u128 << 127;
        let a = rect((0, midpoint - 1), (0, 0));
        let b = rect((midpoint, u128::MAX), (0, 0));
        let rules = [
            RuleSpace {
                region: Some(a),
                target: Verdict::Accept,
            },
            RuleSpace {
                region: Some(b),
                target: Verdict::Accept,
            },
        ];
        assert!(
            compare(&[domain], &[domain], &rules, Verdict::Drop)
                .unwrap()
                .is_none()
        );
        let mut sample = b;
        sample.0[0].hi = u128::MAX - 1;
        let incomplete = [
            rules[0].clone(),
            RuleSpace {
                region: Some(sample),
                target: Verdict::Accept,
            },
        ];
        let c = compare(&[domain], &[domain], &incomplete, Verdict::Drop)
            .unwrap()
            .unwrap();
        assert_eq!(c.point[0], u128::MAX);
        assert_eq!(actual(&incomplete, Verdict::Drop, c.point), Verdict::Drop);
    }

    #[test]
    fn every_header_dimension_preserves_high_coordinates_and_checked_witnesses() {
        let base = [1u128 << 120, 1u128 << 96, 1, 2, 6, 40000, 443, 128, 0];
        let domain = Rect(base.map(|n| Interval { lo: n, hi: n + 1 }));
        for axis in 0..DIMENSIONS {
            let mut allowed = domain;
            allowed.0[axis].lo += 1;
            let correct = [RuleSpace {
                region: Some(allowed),
                target: Verdict::Accept,
            }];
            assert!(
                compare(&[domain], &[allowed], &correct, Verdict::Drop)
                    .unwrap()
                    .is_none()
            );
            for bits in 0..(1usize << DIMENSIONS) {
                let p = std::array::from_fn(|d| base[d] + ((bits >> d) & 1) as u128);
                let expected = if p[axis] == base[axis] + 1 {
                    Verdict::Accept
                } else {
                    Verdict::Drop
                };
                assert_eq!(actual(&correct, Verdict::Drop, p), expected);
            }
            let shadowed = [
                RuleSpace {
                    region: Some(domain),
                    target: Verdict::Drop,
                },
                correct[0].clone(),
            ];
            let c = compare(&[domain], &[allowed], &shadowed, Verdict::Drop)
                .unwrap()
                .unwrap();
            assert_eq!(c.point[axis], base[axis] + 1);
            assert!(domain.contains(c.point));
            assert_eq!(c.expected, Verdict::Accept);
            assert_eq!(c.actual, Verdict::Drop);
            assert_eq!(c.rule, Some(1));
            assert_eq!(actual(&shadowed, Verdict::Drop, c.point), c.actual);
        }
    }
}
