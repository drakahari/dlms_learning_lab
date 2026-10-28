//! Authored ACL content regressions. Fixed virtual packets/files only.
//! The same validation works before and after ordinary bundle registration.
use dlms_learning_lab::{
    acl_editor as editor, acl_routing as routing, acl_teaching,
    acl_workspace::{self as model, Draft},
    course,
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching::View,
    workspace,
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

const UNIT: &str = "ipv4-access-policy";
const IDS: [&str; 4] = [
    "ccna-acl-source-policy",
    "ccna-acl-service-order",
    "ccna-acl-direction-repair",
    "ccna-acl-policy-handover",
];
// Exact original 90 IDs captured from the normal source packs on 2026-10-23.
const BASELINE_IDS: [&str; 90] = [
    "ccna-ipv4",
    "ccna-vlsm",
    "ccna-gateway",
    "lpic1-permissions",
    "lpic1-traversal",
    "lpic1-logs",
    "lpic2-service",
    "lpic2-dns",
    "lpic2-storage",
    "shell-handover",
    "shell-arguments",
    "shell-input",
    "shell-validation",
    "shell-report",
    "shell-loop",
    "shell-capstone",
    "shell-streams",
    "shell-pipelines",
    "shell-eof",
    "shell-batch-audit",
    "shell-functions",
    "shell-return",
    "shell-shift",
    "shell-bundle",
    "lpic1-file-navigation",
    "lpic1-file-names",
    "lpic1-copy-move",
    "lpic1-file-cleanup",
    "lpic1-creation-mask",
    "lpic1-file-handover",
    "ccna-ipv6-neighbor",
    "ccna-ipv6-workstation",
    "ccna-ipv6-scope",
    "ccna-ipv6-specific-route",
    "ccna-ipv6-backup",
    "ccna-ipv6-handover",
    "shell-local-heading",
    "shell-scope-review",
    "shell-scope-capstone",
    "shell-cut-inventory",
    "shell-sort-manifest",
    "shell-uniq-counts",
    "shell-text-capstone",
    "shell-number-records",
    "shell-order-total",
    "shell-pack-items",
    "shell-count-capstone",
    "lpic1-team-brief",
    "lpic1-drop-directory",
    "lpic1-update-existing",
    "lpic1-retire-handover",
    "ccna-vlan-access",
    "ccna-vlan-trunk",
    "ccna-vlan-native",
    "ccna-vlan-handover",
    "shell-file-manifest",
    "shell-collect-errors",
    "shell-file-batch-capstone",
    "ccna-ipv4-network-route",
    "ccna-ipv4-return-route",
    "ccna-ipv4-specific-route",
    "ccna-ipv4-host-exception",
    "ccna-ipv4-floating-route",
    "ccna-ipv4-branch-handover",
    "lpic2-dns-dualstack",
    "lpic2-dns-aliases",
    "lpic2-dns-mail",
    "lpic2-dns-rrsets",
    "lpic2-dns-secondary",
    "lpic2-dns-handover",
    "shell-array-intake",
    "shell-array-priority",
    "shell-array-package",
    "shell-array-handover",
    "lpic1-process-pause",
    "lpic1-process-pending",
    "lpic1-process-priority",
    "lpic1-process-handover",
    "ccna-lldp-receive",
    "ccna-lldp-inventory",
    "ccna-lldp-transmit-policy",
    "ccna-lldp-handover",
    "lpic2-mount-identity",
    "lpic2-mount-policy",
    "lpic2-mount-manual",
    "lpic2-mount-handover",
    "shell-capture-preview",
    "shell-capture-evidence",
    "shell-capture-decision",
    "shell-capture-handover",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Review {
    version: u32,
    scope: String,
    plans: Vec<Plan>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    id: String,
    reference: editor::Drafts,
    alternatives: Vec<Method>,
    negatives: Vec<Method>,
    omissions: Vec<String>,
    protected_route_attack: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Method {
    name: String,
    documents: editor::Drafts,
}
fn labs() -> Vec<Scenario> {
    pack::load(include_str!("../scenarios/acl-policy-v1.json")).unwrap()
}
fn plans() -> Vec<Plan> {
    let r: Review = serde_json::from_str(include_str!("fixtures/acl-content-review.json")).unwrap();
    assert_eq!(r.version, 1);
    assert!(!r.scope.trim().is_empty());
    r.plans
}
fn catalog() -> Vec<Scenario> {
    let mut all: Vec<_> = pack::bundled()
        .unwrap()
        .into_iter()
        .filter(|s| !IDS.contains(&s.id.as_str()))
        .collect();
    all.extend(labs());
    // The catalog is several independently bounded packs; growing content
    // must not turn this fixture into one oversized artificial pack.
    let before = serde_json::to_value(&all).unwrap();
    let packs = all
        .chunks(16)
        .map(|rows| serde_json::json!({"schema_version":1,"scenarios":rows}).to_string())
        .collect::<Vec<_>>();
    assert!(packs.iter().all(|text| text.len() <= pack::MAX_PACK_BYTES));
    let loaded = pack::load_catalog(&packs.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        before,
        "Chunking retains every typed definition and original fixture order"
    );
    loaded
}
fn network(s: &State) -> &routing::Network {
    let State::Ipv4Acl { network } = s else {
        panic!("ACL state required")
    };
    network
}
fn network_mut(s: &mut State) -> &mut routing::Network {
    let State::Ipv4Acl { network } = s else {
        panic!("ACL state required")
    };
    network
}
fn definition(s: &Scenario) -> &editor::Definition {
    let Goal::Ipv4Acl { definition } = &s.goal else {
        panic!("ACL definition required")
    };
    definition
}
fn reference(s: &Scenario) -> editor::Drafts {
    assert_eq!(s.solution.len(), 1);
    let action = s.solution[0]
        .strip_prefix("acl-author ")
        .expect("pack-only adapter");
    let model::Action::ApplyDocuments { documents } = serde_json::from_str(action).unwrap();
    documents
}
fn full_documents(n: &routing::Network, updates: &editor::Drafts) -> editor::Drafts {
    let mut all = Draft::new(n).unwrap().documents;
    all.routers.extend(updates.routers.clone());
    all
}
fn prepared(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    acl_teaching::prepare(&mut a, s.acl_teaching.as_ref().unwrap(), true).unwrap();
    a
}
fn solved(s: &Scenario) -> Attempt {
    let mut a = prepared(s);
    let docs = full_documents(network(&a.state), &reference(s));
    let w = a.workspace.as_mut().unwrap();
    let d = w.acl_draft.as_mut().unwrap();
    d.documents = docs;
    d.apply(network_mut(&mut a.state)).unwrap();
    w.checked = true;
    assert!(progress::current_solved(&a, s));
    a
}
fn practice(a: &mut Attempt) -> &mut acl_teaching::Learning {
    a.workspace.as_mut().unwrap().acl_learning.as_mut().unwrap()
}
fn fresh_report(s: &Scenario, documents: &editor::Drafts) -> (routing::Network, editor::Report) {
    let n = model::apply_documents(network(&s.initial), documents).unwrap();
    let report = editor::grade(definition(s), &n, &editor::Drafts::default()).unwrap();
    (n, report)
}
fn assert_meaningful_failure(
    s: &Scenario,
    n: &routing::Network,
    report: &editor::Report,
    context: &str,
) {
    assert!(!report.complete, "{} / {context}", s.id);
    assert_eq!(report.cases.len(), definition(s).cases.len());
    assert!(
        report.cases.iter().any(|c| c.passed),
        "{} / {context}: no meaningful partial outcome",
        s.id
    );
    assert!(
        report.cases.iter().any(|c| !c.passed),
        "{} / {context}",
        s.id
    );
    let feedback = model::checks(n, definition(s), &editor::Drafts::default());
    assert_eq!(feedback.len(), report.cases.len());
    for ((passed, message), case) in feedback.iter().zip(&report.cases) {
        assert_eq!(*passed, case.passed);
        if !*passed {
            assert!(message.starts_with(&case.label));
            assert!(
                message.len() > case.label.len() + 10,
                "missing diagnostic context: {message}"
            );
            assert!(!message.contains("Outcome::"));
        }
    }
}

#[test]
fn proposed_pack_prerequisites_and_unit_map_are_valid_before_optional_bundle_registration() {
    let staged = labs();
    assert_eq!(
        staged.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    assert_eq!(
        staged
            .iter()
            .map(|s| definition(s).cases.len())
            .collect::<Vec<_>>(),
        [5, 7, 6, 13]
    );
    let mut previous = BTreeSet::new();
    for (index, s) in staged.iter().enumerate() {
        let expected: Vec<String> = if index == 0 {
            Vec::new()
        } else {
            vec![IDS[index - 1].into()]
        };
        assert_eq!(s.prerequisites, expected);
        assert_eq!(s.track, "CCNA");
        assert_eq!(s.unit, UNIT);
        assert!(s.prerequisites.iter().all(|id| previous.contains(id)));
        previous.insert(s.id.clone());
        let notes = s.simplifications.to_lowercase();
        for limit in ["listed", "one-way", "independent", "session", "no host"] {
            assert!(notes.contains(limit), "{}: missing {limit} limit", s.id);
        }
        assert_eq!(network(&s.initial), &definition(s).initial);
        definition(s).validate().unwrap();
        assert!(!engine::solved(&s.initial, &s.goal));
    }
    let bundled = pack::bundled().unwrap();
    for id in BASELINE_IDS {
        assert!(
            bundled.iter().any(|s| s.id == id),
            "prior scenario missing: {id}"
        );
    }
    let mut units = course::bundled(&bundled).unwrap();
    let proposed = course::Unit {
        id: UNIT.into(),
        track: "CCNA".into(),
        title: "IPv4 access policies without collateral blocking".into(),
        summary: "Source, service, order and direction with preserved traffic.".into(),
        prerequisites: vec!["ipv4-static-foundations".into()],
        labs: IDS.iter().map(|id| (*id).into()).collect(),
    };
    assert!(
        proposed
            .prerequisites
            .iter()
            .all(|id| units.iter().any(|u| &u.id == id))
    );
    if bundled.iter().any(|s| IDS.contains(&s.id.as_str())) {
        assert!(IDS.iter().all(|id| bundled.iter().any(|s| s.id == *id)));
        let actual = units
            .iter()
            .find(|u| u.id == UNIT)
            .expect("registered labs need their actual unit");
        assert_eq!(actual.labs, proposed.labs);
        assert_eq!(actual.prerequisites, proposed.prerequisites);
    }
    units.retain(|u| u.id != UNIT);
    units.push(proposed);
    course::load(
        &serde_json::json!({"version":1,"units":units}).to_string(),
        &catalog(),
    )
    .unwrap();
    let routers: Vec<_> = staged
        .iter()
        .map(|s| {
            network(&s.initial)
                .topology
                .devices
                .values()
                .filter(|d| d.forwarding)
                .count()
        })
        .collect();
    assert_eq!(routers, [1, 1, 2, 2]);
}

#[test]
fn reference_and_semantic_alternatives_pass_every_declared_current_packet_case() {
    let review = plans();
    assert_eq!(review.len(), 4);
    for (s, p) in labs().iter().zip(&review) {
        assert_eq!(s.id, p.id);
        assert_eq!(reference(s), p.reference);
        assert!(!p.alternatives.is_empty());
        let mut state = s.initial.clone();
        for action in &s.solution {
            engine::execute(&mut state, action).unwrap();
        }
        assert!(engine::solved(&state, &s.goal));
        for (name, docs) in std::iter::once(("reference", &p.reference)).chain(
            p.alternatives
                .iter()
                .map(|a| (a.name.as_str(), &a.documents)),
        ) {
            let (n, report) = fresh_report(s, docs);
            assert!(report.complete, "{} / {name}", s.id);
            assert_eq!(report.cases.len(), definition(s).cases.len());
            assert!(report.cases.iter().all(|c| c.passed));
            for (task, result) in definition(s).cases.iter().zip(&report.cases) {
                assert_eq!(task.id, result.id);
                match &task.expectation {
                    editor::Expectation::Allowed {} => assert!(result.observation.delivered()),
                    editor::Expectation::Denied { expected } => {
                        let routing::Outcome::Denied { denial, .. } = &result.observation.actual
                        else {
                            panic!("negative must be a policy denial")
                        };
                        assert_eq!(denial.at, expected.at);
                    }
                }
            }
            assert!(engine::solved(&State::Ipv4Acl { network: n }, &s.goal));
        }
        // Independently ensure each negative is otherwise deliverable, rather
        // than relying on an unsupported/missing-route failure as its baseline.
        let baseline = routing::Network {
            schema_version: 1,
            topology: network(&s.initial).topology.clone(),
            host_endpoints: network(&s.initial).host_endpoints.clone(),
            devices: BTreeMap::new(),
        };
        for case in &definition(s).cases {
            assert!(
                baseline.inspect(&case.flow).unwrap().delivered(),
                "{} / {}",
                s.id,
                case.id
            );
        }
    }
}

#[test]
fn valid_wrong_policies_fail_with_partial_results_and_flow_specific_feedback() {
    for (s, p) in labs().iter().zip(plans()) {
        assert!(p.protected_route_attack.len() > 30);
        assert!(!p.omissions.is_empty());
        assert!(p.omissions.iter().all(|s| s.len() > 20));
        assert!(!p.negatives.is_empty());
        for wrong in &p.negatives {
            let (n, report) = fresh_report(s, &wrong.documents);
            assert_meaningful_failure(s, &n, &report, &wrong.name);
            assert!(!engine::solved(&State::Ipv4Acl { network: n }, &s.goal));
        }
    }
}

#[test]
fn source_port_misconception_exposes_both_missed_denials_and_collateral_blocking() {
    let s = labs().into_iter().find(|s| s.id == IDS[1]).unwrap();
    let mut wrong = reference(&s);
    let text = wrong.routers.get_mut("Edge").unwrap();
    let destination_rule = "deny tcp any host 10.20.9.30 eq 22";
    assert_eq!(text.matches(destination_rule).count(), 1);
    *text = text.replace(destination_rule, "deny tcp any eq 22 host 10.20.9.30");
    let (n, report) = fresh_report(&s, &wrong);
    assert_meaningful_failure(&s, &n, &report, "source port mistaken for destination port");
    for id in ["designer-ssh", "support-ssh"] {
        let case = report.cases.iter().find(|c| c.id == id).unwrap();
        assert!(!case.passed, "{id}: source50000 cannot match source22");
        assert!(
            case.observation.delivered(),
            "{id}: unwanted TCP22 was allowed"
        );
    }
    let collateral = report.cases.iter().find(|c| c.id == "support-alt").unwrap();
    assert!(!collateral.passed);
    assert!(matches!(
        collateral.observation.actual,
        routing::Outcome::Denied { .. }
    ));
    assert!(
        report
            .cases
            .iter()
            .any(|c| c.id == "designer-udp" && c.passed)
    );
}

#[test]
fn each_capstone_omitted_repair_and_collateral_block_is_detected() {
    let s = labs().into_iter().find(|s| s.id == IDS[3]).unwrap();
    let good = reference(&s);
    for (name, router, old, new) in [
        (
            "wrong Edge direction",
            "Edge",
            "ip access-group portal-services in",
            "ip access-group portal-services out",
        ),
        (
            "permit still shadows deny",
            "Edge",
            " 5 deny tcp",
            " 20 deny tcp",
        ),
        (
            "wrong Core source",
            "Core",
            "permit host 10.80.1.10",
            "permit host 10.80.1.20",
        ),
        (
            "overbroad Portal protocol",
            "Edge",
            "deny tcp any host 10.90.2.20 eq 22",
            "deny ip any host 10.90.2.20",
        ),
        (
            "TCP22 denied to every destination",
            "Edge",
            "deny tcp any host 10.90.2.20 eq 22",
            "deny tcp any any eq 22",
        ),
        (
            "blanket Archive denial",
            "Core",
            "permit host 10.80.1.10",
            "deny any",
        ),
    ] {
        let mut wrong = good.clone();
        let text = wrong.routers.get_mut(router).unwrap();
        assert!(text.contains(old));
        *text = text.replace(old, new);
        let (n, report) = fresh_report(&s, &wrong);
        assert_meaningful_failure(&s, &n, &report, name);
        if name == "blanket Archive denial" {
            for c in &report.cases {
                if c.id.starts_with("visitor-archive") {
                    assert!(
                        !c.passed,
                        "a matching denial cannot earn credit without required allowed traffic"
                    );
                }
            }
        }
    }
    let mut wrong = good;
    let text = wrong.routers.get_mut("Core").unwrap();
    *text = text
        .replace("interface archive", "interface transit")
        .replace(
            "ip access-group archive-users out",
            "ip access-group archive-users in",
        );
    let (n, report) = fresh_report(&s, &wrong);
    assert_meaningful_failure(&s, &n, &report, "wrong Core boundary");
}

#[test]
fn fixed_network_tampering_never_converts_path_failure_to_security_credit() {
    for s in labs() {
        let good = solved(&s);
        let expected = definition(&s)
            .cases
            .iter()
            .find_map(|c| {
                if let editor::Expectation::Denied { expected } = &c.expectation {
                    Some(expected)
                } else {
                    None
                }
            })
            .unwrap();
        for mode in ["forwarding", "port-down", "missing-route"] {
            let mut broken = good.clone();
            let n = network_mut(&mut broken.state);
            match mode {
                "forwarding" => {
                    n.topology
                        .devices
                        .get_mut(&expected.at.device)
                        .unwrap()
                        .forwarding = false
                }
                "port-down" => {
                    n.topology
                        .devices
                        .get_mut(&expected.at.device)
                        .unwrap()
                        .interfaces
                        .get_mut(&expected.at.interface)
                        .unwrap()
                        .up = false
                }
                _ => {
                    let host = n.host_endpoints.keys().next().unwrap().clone();
                    n.topology.devices.get_mut(&host).unwrap().routes.clear();
                }
            }
            assert!(
                editor::grade(definition(&s), n, &editor::Drafts::default()).is_err(),
                "{} / {mode}",
                s.id
            );
            assert!(!engine::solved(&broken.state, &s.goal));
            assert!(!progress::current_solved(&broken, &s));
        }
    }
}

#[test]
fn malformed_pending_and_applied_damage_invalidate_then_repair_current_success() {
    for s in labs() {
        let mut a = solved(&s);
        let applied = a.state.clone();
        let good = a
            .workspace
            .as_ref()
            .unwrap()
            .acl_draft
            .as_ref()
            .unwrap()
            .documents
            .clone();
        let router = reference(&s).routers.keys().next().unwrap().clone();
        a.workspace
            .as_mut()
            .unwrap()
            .acl_draft
            .as_mut()
            .unwrap()
            .documents
            .routers
            .insert(
                router,
                "ip access-list extended unfinished\n permit tcp any\n".into(),
            );
        assert!(!progress::current_solved(&a, &s));
        assert_eq!(a.state, applied);
        let w = a.workspace.as_mut().unwrap();
        assert!(
            w.acl_draft
                .as_mut()
                .unwrap()
                .apply(network_mut(&mut a.state))
                .is_err()
        );
        assert_eq!(a.state, applied);
        let w = a.workspace.as_mut().unwrap();
        w.acl_draft.as_mut().unwrap().documents = good.clone();
        w.acl_draft
            .as_mut()
            .unwrap()
            .apply(network_mut(&mut a.state))
            .unwrap();
        assert!(progress::current_solved(&a, &s));
        let w = a.workspace.as_mut().unwrap();
        for text in w.acl_draft.as_mut().unwrap().documents.routers.values_mut() {
            text.clear();
        }
        w.acl_draft
            .as_mut()
            .unwrap()
            .apply(network_mut(&mut a.state))
            .unwrap();
        assert!(!progress::current_solved(&a, &s));
        let w = a.workspace.as_mut().unwrap();
        w.acl_draft.as_mut().unwrap().documents = good;
        w.acl_draft
            .as_mut()
            .unwrap()
            .apply(network_mut(&mut a.state))
            .unwrap();
        assert!(progress::current_solved(&a, &s));
        a.challenge = true;
        assert!(progress::current_solved(&a, &s));
    }
}

#[test]
fn separate_examples_fill_apply_try_and_refill_repair_without_assignment_credit() {
    let mut example_counts = Vec::new();
    for s in labs() {
        let lesson = s.acl_teaching.as_ref().unwrap();
        lesson.validate().unwrap();
        assert!(lesson.distinct_from(network(&s.initial)));
        example_counts.push(lesson.definition.cases.len());
        let assignment_names: BTreeSet<_> = network(&s.initial).topology.devices.keys().collect();
        assert!(
            lesson
                .fixture
                .topology
                .devices
                .keys()
                .all(|n| !assignment_names.contains(n))
        );
        let mut a = prepared(&s);
        let before = a.state.clone();
        let initial_example = practice(&mut a).network.clone();
        practice(&mut a).fill_example(lesson).unwrap();
        assert_eq!(practice(&mut a).network, initial_example);
        assert!(!practice(&mut a).checked);
        assert_eq!(a.state, before);
        assert!(!progress::current_solved(&a, &s));
        assert!(
            practice(&mut a)
                .apply_check(lesson)
                .unwrap()
                .iter()
                .all(|c| c.0)
        );
        assert_eq!(a.state, before);
        assert!(!progress::current_solved(&a, &s));
        let changed = full_documents(&practice(&mut a).network, &lesson.try_change.documents);
        practice(&mut a).draft.documents = changed;
        let checks = practice(&mut a).apply_check(lesson).unwrap();
        let actual: BTreeMap<_, _> = lesson
            .definition
            .cases
            .iter()
            .zip(checks)
            .map(|(case, (passed, _))| (case.id.clone(), passed))
            .collect();
        assert_eq!(actual, lesson.try_change.expected);
        assert!(actual.values().any(|p| !*p));
        assert_eq!(a.state, before);
        let changed_network = practice(&mut a).network.clone();
        practice(&mut a).fill_example(lesson).unwrap();
        assert_eq!(practice(&mut a).network, changed_network);
        assert!(
            practice(&mut a)
                .apply_check(lesson)
                .unwrap()
                .iter()
                .all(|c| c.0)
        );
        assert_eq!(a.state, before);
        assert!(!progress::current_solved(&a, &s));
    }
    assert_eq!(example_counts, [5, 4, 5, 8]);
}

#[test]
fn all_ninety_prior_attempts_and_two_pending_drafts_resume_exactly_with_local_reset() {
    let all = catalog();
    let mut p = Progress {
        schema_version: 1,
        clock: 401,
        ..Default::default()
    };
    for (index, id) in BASELINE_IDS.iter().enumerate() {
        let s = all.iter().find(|s| s.id == *id).unwrap();
        let mut a = Attempt::new(s, index as u64 + 1);
        a.challenge = index % 2 == 0;
        a.transcript.push(format!("Synthetic prior note for {id}"));
        p.attempts.insert(s.id.clone(), a);
    }
    let prior = serde_json::to_value(&p.attempts).unwrap();
    p.favorites.insert(BASELINE_IDS[0].into());
    for s in labs() {
        let mut a = solved(&s);
        let lesson = s.acl_teaching.as_ref().unwrap();
        practice(&mut a).fill_example(lesson).unwrap();
        practice(&mut a).apply_check(lesson).unwrap();
        practice(&mut a).view = View::Example;
        practice(&mut a).checked = true;
        a.workspace
            .as_mut()
            .unwrap()
            .acl_draft
            .as_mut()
            .unwrap()
            .documents
            .routers
            .values_mut()
            .next()
            .unwrap()
            .push_str("\nip access-list extended pending\n");
        practice(&mut a)
            .draft
            .documents
            .routers
            .values_mut()
            .next()
            .unwrap()
            .push_str("\nip access-list extended separate-pending\n");
        assert!(!progress::current_solved(&a, &s));
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic-acl-content-progress.json");
    progress::save(&path, &p).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let mut loaded = progress::load(&path, &all).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(
        serde_json::to_value(&loaded.attempts).unwrap(),
        serde_json::to_value(&p.attempts).unwrap()
    );
    assert_eq!(loaded.favorites, p.favorites);
    assert_eq!(loaded.clock, 401);
    for id in BASELINE_IDS {
        assert_eq!(
            serde_json::to_value(&loaded.attempts[id]).unwrap(),
            prior[id]
        );
    }
    let selected = labs().remove(0);
    let others: BTreeMap<_, _> = loaded
        .attempts
        .iter()
        .filter(|(id, _)| id.as_str() != selected.id.as_str())
        .map(|(id, a)| (id.clone(), serde_json::to_vec(a).unwrap()))
        .collect();
    loaded
        .attempts
        .insert(selected.id.clone(), prepared(&selected));
    assert_eq!(loaded.attempts[&selected.id].state, selected.initial);
    assert!(!progress::current_solved(
        &loaded.attempts[&selected.id],
        &selected
    ));
    for (id, bytes) in others {
        assert_eq!(serde_json::to_vec(&loaded.attempts[&id]).unwrap(), bytes);
    }
    progress::save(&path, &loaded).unwrap();
    let loaded = progress::load(&path, &all).unwrap();
    assert!(loaded.favorites.contains(&selected.id));
    for id in BASELINE_IDS {
        assert_eq!(
            serde_json::to_value(&loaded.attempts[id]).unwrap(),
            prior[id]
        );
    }
}

#[test]
fn harmless_unattached_campus_list_remains_an_equivalent_finite_outcome_solution() {
    let s = labs().into_iter().find(|s| s.id == IDS[2]).unwrap();
    let mut docs = reference(&s);
    docs.routers.insert(
        "Campus".into(),
        "ip access-list standard spare-campus\n permit any\nexit\n".into(),
    );
    let (n, report) = fresh_report(&s, &docs);
    assert!(report.complete);
    assert_ne!(n.devices, network(&s.initial).devices);
    assert_eq!(n.topology, network(&s.initial).topology);
    assert_eq!(n.host_endpoints, network(&s.initial).host_endpoints);
}

#[test]
fn every_authored_packet_requirement_is_regraded_after_a_matching_applied_policy_change() {
    use dlms_learning_lab::acl;
    let mut mutations = 0;
    for s in labs() {
        let good = solved(&s);
        for case in &definition(&s).cases {
            let mut a = good.clone();
            let n = network_mut(&mut a.state);
            let (at, action) = match &case.expectation {
                editor::Expectation::Allowed {} => {
                    let observed = n.inspect(&case.flow).unwrap();
                    let routing::Outcome::Delivered { events, .. } = observed.actual else {
                        panic!("reference flow not delivered")
                    };
                    let at = events
                        .iter()
                        .find_map(|e| match e {
                            routing::Event::Boundary { at, .. } => Some(at.clone()),
                            _ => None,
                        })
                        .unwrap();
                    (at, acl::Action::Deny)
                }
                editor::Expectation::Denied { expected } => {
                    (expected.at.clone(), acl::Action::Permit)
                }
            };
            let device =
                n.devices
                    .entry(at.device.clone())
                    .or_insert_with(|| routing::DevicePolicy {
                        policy: acl::Policy {
                            schema_version: 1,
                            lists: BTreeMap::new(),
                        },
                        attachments: BTreeMap::new(),
                    });
            device.policy.lists.insert(
                "verify-change".into(),
                acl::AccessList {
                    kind: acl::Kind::Extended,
                    rules: vec![acl::Rule {
                        id: "row-1".into(),
                        action,
                        selector: acl::Selector::Extended {
                            source: acl::Wildcard::any(),
                            destination: acl::Wildcard::any(),
                            protocol: acl::Protocol::Ip {},
                        },
                    }],
                },
            );
            let binding = device.attachments.entry(at.interface).or_default();
            match at.direction {
                routing::Direction::Inbound => binding.inbound = Some("verify-change".into()),
                routing::Direction::Outbound => binding.outbound = Some("verify-change".into()),
            }
            n.validate().unwrap();
            let draft = Draft::new(n).unwrap();
            // Matching edited and applied state rules out merely detecting an unsaved document.
            let report = editor::grade(definition(&s), n, &draft.documents).unwrap();
            assert!(
                !report
                    .cases
                    .iter()
                    .find(|c| c.id == case.id)
                    .unwrap()
                    .passed,
                "{} {}",
                s.id,
                case.id
            );
            assert!(!report.complete);
            a.workspace.as_mut().unwrap().acl_draft = Some(draft);
            assert!(
                !progress::current_solved(&a, &s),
                "cached pass remained: {} {}",
                s.id,
                case.id
            );
            assert!(progress::current_solved(&good, &s));
            mutations += 1;
        }
    }
    assert_eq!(mutations, 31);
}

#[test]
fn library_search_includes_the_actual_assignment_goal_and_composes_with_advanced_filter() {
    use dlms_learning_lab::catalog::{self, Filter};
    let all = labs();
    let filter = Filter {
        track: "CCNA".into(),
        unit: UNIT.into(),
        difficulty: "Advanced".into(),
        search: "CORE/ARCHIVE".into(),
        ..Default::default()
    };
    let matching: Vec<_> = all
        .iter()
        .filter(|s| catalog::matches(s, &filter, false, false, false))
        .map(|s| s.id.as_str())
        .collect();
    assert_eq!(matching, vec!["ccna-acl-policy-handover"]);
    let filter = Filter {
        favorites: true,
        ..filter
    };
    assert!(!catalog::matches(&all[3], &filter, false, false, false));
    assert!(catalog::matches(&all[3], &filter, false, false, true));
}
