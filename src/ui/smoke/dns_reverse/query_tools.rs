//! Fixed current-catalog extension of the existing native DNS editor workflow.
use super::*;
pub(super) const IDS: [&str; 3] = [
    "lpic2-dns-dualstack",
    "lpic2-dns-aliases",
    "lpic2-dns-secondary",
];
fn query(p: &mut Vec<Step>, input: String, expected: &str) {
    field(p, Field::Command, input);
    p.extend([
        Step::Click(Target::Lookup),
        Step::CommandOutput(expected.into()),
    ]);
}
pub(super) fn plan(scenarios: &[Scenario]) -> Vec<Step> {
    let mut p = vec![];
    for (n, id) in IDS.iter().enumerate() {
        let s = scenarios.iter().find(|s| s.id == *id).unwrap();
        let lesson = s.dns_teaching.as_ref().unwrap();
        assert!(lesson.lookup.is_some());
        let Goal::AuthoritativeDns { requirements } = &s.goal else {
            panic!()
        };
        let good = vec![true; requirements.len()];
        let good_example = vec![true; lesson.outcomes.len()];
        let shot = |name: &str| format!("dns-queries-{n}-{name}");
        p.extend([
            Step::Library,
            Step::Filter(n),
            Step::Wait,
            Step::Start,
            Step::Wait,
            Step::Shot(shot("introduction"), 0),
            Step::Shot(shot("introduction-worked"), 5),
            Step::Click(Target::Try),
            Step::Wait,
            Step::Click(Target::Commands),
        ]);
        let zone = &lesson.fixture.primary;
        match n {
            0 => {
                query(
                    &mut p,
                    format!("host -r -t AAAA web.{} 192.0.2.53", zone.origin),
                    "has no AAAA record",
                );
                query(
                    &mut p,
                    format!("host -r -t AAAA files.{} 192.0.2.53", zone.origin),
                    "NXDOMAIN",
                );
            }
            1 => {
                query(
                    &mut p,
                    format!(
                        "dig @192.0.2.53 portal.{} CNAME +norecurse +short",
                        zone.origin
                    ),
                    "status.",
                );
                query(
                    &mut p,
                    format!("host -r -t A portal.{} 192.0.2.53", zone.origin),
                    "SERVFAIL",
                );
            }
            2 => {
                query(
                    &mut p,
                    format!("dig @192.0.2.53 web.{} A +norecurse +short", zone.origin),
                    "203.0.113.180",
                );
                query(
                    &mut p,
                    format!("dig @192.0.2.54 web.{} A +norecurse +short", zone.origin),
                    "Secondary",
                );
            }
            _ => unreachable!(),
        }
        p.extend([
            Step::Shot(shot("practice-observation"), 2),
            Step::BeforeFill,
            Step::Click(Target::Fill),
            Step::FillOnly,
        ]);
        publish(&mut p, n == 2);
        check(&mut p, true, good_example, true, shot("practice-pass"));
        p.extend([
            Step::Click(Target::Assignment),
            Step::Wait,
            Step::Shot(shot("assignment"), 0),
            Step::Click(Target::Commands),
        ]);
        let State::AuthoritativeDns { server } = &s.initial else {
            panic!()
        };
        query(
            &mut p,
            format!("host -r -t NS {} 192.0.2.53", server.primary.origin),
            "name server",
        );
        query(
            &mut p,
            format!("host -r -t TXT {} 192.0.2.53", server.primary.origin),
            "Lookup not evaluated",
        );
        p.push(Step::Shot(shot("unsupported-feedback"), 2));
        edit(&mut p, &reference(s, false));
        publish(&mut p, n == 2);
        check(&mut p, false, good.clone(), true, shot("repaired"));
        let refzone = reference(s, false);
        let name = refzone
            .records
            .iter()
            .find(|r| matches!(r.data, Data::A { .. }) && !r.owner.starts_with("ns."))
            .unwrap()
            .owner
            .clone();
        query(
            &mut p,
            format!("host -r -t A {name} 192.0.2.53"),
            "has address",
        );
        p.extend([Step::Rows(false, good.clone(), true), Step::SaveReadback]);
        if n == 2 {
            query(
                &mut p,
                format!("dig @192.0.2.54 {name} A +norecurse +short"),
                "Secondary",
            );
        }
        p.extend([
            Step::Theme(true, 1.5, true),
            Step::Wait,
            Step::Geometry(1.5, true),
            Step::Shot(shot("light150-command"), 4),
            Step::Theme(false, 2., true),
            Step::Wait,
            Step::Geometry(2., true),
            Step::Shot(shot("dark200-command"), 4),
            Step::Shot(shot("dark200-output"), 2),
            Step::SaveReadback,
        ]);
    }
    p.push(Step::Finish);
    assert!(p.len() < END - START);
    p
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authoritative_native_plan_uses_current_optin_lessons_and_actual_controls() {
        let labs = pack::current().unwrap();
        let p = plan(&labs);
        assert_eq!(p.last(), Some(&Step::Finish));
        for n in 0..3 {
            assert!(p.contains(&Step::Filter(n)));
        }
        assert!(
            p.iter()
                .any(|s| matches!(s,Step::CommandOutput(t)if t=="NXDOMAIN"))
        );
        assert!(
            p.iter()
                .any(|s| matches!(s,Step::CommandOutput(t)if t=="SERVFAIL"))
        );
        assert!(
            p.iter()
                .any(|s| matches!(s,Step::Text(Field::Command,t)if t.contains("192.0.2.54")))
        );
        assert_eq!(
            p.iter().filter(|s| matches!(s, Step::SaveReadback)).count(),
            6
        );
    }
}
