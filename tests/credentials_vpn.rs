use dlms_learning_lab::security_investigation::{
    credential_task::Expected,
    credentials::{Policy, SecondFactor},
    session::Definition,
    vpn_design::{Path, Purpose, Role},
    vpn_design_task::Draft,
    workspace::{Action, Scope, Task, Work},
};

const CREDENTIALS: &str = "ccna-credential-policy-repair";
const VPN: &str = "ccna-ipsec-design-review";

fn fixture(id: &str) -> (Definition, serde_json::Value) {
    let all: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/credentials_vpn_tasks_v1.json")).unwrap();
    let task = &all[id];
    (
        serde_json::from_value(task["definition"].clone()).unwrap(),
        task["answers"].clone(),
    )
}

#[test]
fn good_lengths_accept_legitimate_proposals_and_handle_compromise() {
    let (definition, answers) = fixture(CREDENTIALS);
    definition.validate().unwrap();
    let task = &definition.assignment;
    let Task::Credentials { supplied } = task else {
        panic!();
    };
    let mut right: Policy = serde_json::from_value(answers["assignment"].clone()).unwrap();
    let counts = |policy| {
        supplied
            .outcomes(policy)
            .into_iter()
            .filter(|r| r.passed)
            .count()
    };
    assert_eq!(counts(supplied.starter), 4);
    assert_eq!(counts(right), 10);
    right.minimum_length += 1;
    assert_eq!(counts(right), 10);
    right.minimum_length = 64;
    assert_eq!(
        counts(right),
        9,
        "A policy that rejects the approved phrase is too restrictive"
    );
    right.minimum_length = 8;
    assert_eq!(
        counts(right),
        9,
        "The short unlisted proposal must still fail"
    );
    right.minimum_length = 17;
    right.replace_after_compromise = false;
    assert_eq!(supplied.trials[8].test(right).0, Expected::Allow);
    assert!(
        supplied.trials[8]
            .outcome(right)
            .detail
            .contains("replacement")
    );
}

#[test]
fn memorized_codes_public_certificates_and_unbound_biometrics_do_not_prove_possession() {
    let (definition, answers) = fixture(CREDENTIALS);
    let task = &definition.assignment;
    let Task::Credentials { supplied } = task else {
        panic!();
    };
    let valid: Policy = serde_json::from_value(answers["assignment"].clone()).unwrap();
    for row in &supplied.trials[5..8] {
        assert_eq!(row.test(valid).0, Expected::Deny);
    }
    assert!(supplied.trials[6].test(valid).1.contains("private key"));
    assert!(
        supplied.trials[7]
            .test(valid)
            .1
            .contains("bound authenticator")
    );
    let mut work = Work::new(task);
    work.apply(task, Action::Credentials { policy: valid })
        .unwrap();
    work.apply(task, Action::Check).unwrap();
    let mut wrong = valid;
    wrong.second_factor = SecondFactor::MemorizedCode;
    work.apply(task, Action::Credentials { policy: wrong })
        .unwrap();
    assert!(!work.checked);
    let report = work.outcomes(task);
    assert!(!report[3].passed && !report[4].passed && !report[5].passed);
    let mut bad = wrong;
    bad.minimum_length = 0;
    let old = work.clone();
    assert!(
        work.apply(task, Action::Credentials { policy: bad })
            .is_err()
    );
    assert_eq!(work, old);
}

#[test]
fn vpn_distinguishes_outer_endpoints_from_inner_ranges_and_matches_both_directions() {
    let (definition, answers) = fixture(VPN);
    definition.validate().unwrap();
    let task = &definition.assignment;
    let Task::VpnDesign { supplied } = task else {
        panic!();
    };
    let drafts: [Draft; 2] = serde_json::from_value(answers["assignment"].clone()).unwrap();
    let a = drafts[0].profile().unwrap();
    let b = drafts[1].profile().unwrap();
    let top = supplied.topology;
    assert!(
        matches!(top.trace(&[a,b], supplied.trials[0].traffic).unwrap(),
        Path::Protected { purpose: Purpose::SiteToSite, from, to }
            if from.role == Role::BranchGateway && from.outer_address == top.branch_gateway
            && to.role == Role::HeadquartersGateway && to.outer_address == top.headquarters_gateway)
    );
    assert!(
        matches!(top.trace(&[a,b], supplied.trials[1].traffic).unwrap(),
        Path::Protected { from, to, .. } if from.role == Role::HeadquartersGateway
        && to.role == Role::BranchGateway)
    );
    assert!(
        matches!(top.trace(&[a,b], supplied.trials[2].traffic).unwrap(),
        Path::Protected { purpose: Purpose::RemoteAccess, from, .. }
            if from.outer_address == top.remote_client
            && from.outer_address != supplied.trials[2].traffic.source)
    );
    assert_eq!(
        top.trace(&[a, b], supplied.trials[3].traffic).unwrap(),
        Path::NoMatchingProfile
    );
    assert_eq!(
        top.trace(&[a, b], supplied.trials[4].traffic).unwrap(),
        Path::Local
    );
    let mut overbroad = b;
    overbroad.right_traffic = top.headquarters_lan;
    let results = supplied.outcomes(&[a, overbroad]);
    assert!(
        !results[1].passed,
        "The complete required destination range is graded"
    );
    assert!(
        !results[5].passed,
        "The adjacent service must remain excluded"
    );
}

#[test]
fn swapped_profiles_pass_but_bad_drafts_leave_previous_applied_policies_intact() {
    let (d, answers) = fixture(VPN);
    let task = &d.example;
    let mut work = Work::new(task);
    let mut drafts: [Draft; 2] = serde_json::from_value(answers["example"].clone()).unwrap();
    drafts.swap(0, 1);
    std::mem::swap(&mut drafts[0].left_traffic, &mut drafts[0].right_traffic);
    std::mem::swap(
        &mut drafts[0].endpoints.left,
        &mut drafts[0].endpoints.right,
    );
    work.apply(
        task,
        Action::VpnDrafts {
            drafts: drafts.clone(),
        },
    )
    .unwrap();
    assert!(work.outcomes(task).iter().any(|o| !o.passed));
    work.apply(task, Action::ApplyVpn).unwrap();
    work.apply(task, Action::Check).unwrap();
    assert!(work.outcomes(task).iter().all(|o| o.passed));
    drafts[0].left_traffic = "10.40.20.17/27".into();
    work.apply(task, Action::VpnDrafts { drafts }).unwrap();
    assert!(!work.checked);
    let before = work.clone();
    assert!(
        work.apply(task, Action::ApplyVpn)
            .unwrap_err()
            .contains("aligned network prefix")
    );
    assert_eq!(work, before);
    let Scope::VpnDesign { applied, .. } = work.current else {
        panic!();
    };
    assert_eq!(applied[0].purpose, Purpose::RemoteAccess);
    assert!(work.outcomes(task).iter().any(|o| !o.passed));
}
