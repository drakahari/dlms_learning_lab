use super::directory;
use dlms_learning_lab::ldap_password_workspace as w;
fn task(id: &str, domain: &str, host: &str) -> w::Task {
    let actor = format!("uid=alex,ou=People,dc={domain},dc=test");
    let other = format!("uid=sam,ou=People,dc={domain},dc=test");
    let uri = format!("ldap://{host}");
    let command = format!("ldappasswd -x -W -S -A -ZZ -H {uri} -D '{actor}'");
    let reference = w::Draft {
        command: command.clone(),
        config: "TLS_CACERT /work/training-ca.pem\nTLS_REQCERT demand\n".into(),
        bind_profile: "current".into(),
        old_profile: Some(w::Key("current".into())),
        replacement_profile: Some(w::Key("replacement".into())),
    };
    let starter = w::Draft {
        command: command.replace(" -ZZ", ""),
        config: "TLS_CACERT /work/other-ca.pem\nTLS_REQCERT never\n".into(),
        bind_profile: "replacement".into(),
        ..reference.clone()
    };
    let cases = (0..3)
        .map(|index| {
            let mut initial: dlms_learning_lab::ldap_query::Source = serde_json::from_str(
                &serde_json::to_string(&directory::fixture())
                    .unwrap()
                    .replace("dc=example", &format!("dc={domain}")),
            )
            .unwrap();
            let suffix = initial.suffix.clone();
            let root = initial.entries.iter_mut().find(|e| e.dn == suffix).unwrap();
            root.attributes
                .iter_mut()
                .find(|a| a.name.eq_ignore_ascii_case("dc"))
                .unwrap()
                .values = vec![domain.into()];
            initial
                .entries
                .iter_mut()
                .find(|e| e.dn == other)
                .unwrap()
                .attributes
                .push(directory::attr(
                    "description",
                    &[if index == 0 {
                        "preserve day shift"
                    } else {
                        "preserve another shift"
                    }],
                ));
            let old = format!("Fictional-{id}-old-{}", index + 1);
            let replacement = format!("Fictional-{id}-new-{}", index + 1);
            let unrelated = if index == 2 {
                old.clone()
            } else {
                format!("Fictional-{id}-other-{}", index + 1)
            };
            w::Case {
                id: ["required-old", "optional-old", "shared-old"][index].into(),
                label: [
                    "Current target password required",
                    "Current target password optional",
                    "Another identity shares the old value",
                ][index]
                    .into(),
                initial,
                people: vec![
                    w::Person {
                        dn: actor.clone(),
                        profile: "current".into(),
                    },
                    w::Person {
                        dn: other.clone(),
                        profile: "other".into(),
                    },
                ],
                values: vec![
                    w::Value {
                        profile: "current".into(),
                        sample_password: old,
                    },
                    w::Value {
                        profile: "replacement".into(),
                        sample_password: replacement,
                    },
                    w::Value {
                        profile: "other".into(),
                        sample_password: unrelated,
                    },
                ],
                certificate: w::Certificate {
                    names: vec![w::Name(host.into())],
                    issuer: "training-root".into(),
                    not_before: 100,
                    not_after: 200,
                    confidential: true,
                },
                supports_start_tls: true,
                sample_time: 150,
                policy: w::Policy {
                    supports_password_modify: true,
                    allow_self_change: true,
                    require_old_password: index == 0,
                },
            }
        })
        .collect();
    w::Task {
        version: 1,
        id: id.into(),
        instructions: format!(
            "Change only {actor} to the supplied replacement credential. Require verified protection, reject the previous credential and preserve unrelated identities and all directory fields."
        ),
        uri,
        actor,
        desired_profile: "replacement".into(),
        profiles: vec![
            w::Profile {
                id: "current".into(),
                label: "Lab current credential".into(),
            },
            w::Profile {
                id: "replacement".into(),
                label: "Lab replacement credential".into(),
            },
            w::Profile {
                id: "other".into(),
                label: "Unrelated identity credential".into(),
            },
        ],
        ca_inputs: vec![
            w::RootInput {
                path: "/work/training-ca.pem".into(),
                roots: vec![w::Key("training-root".into())],
            },
            w::RootInput {
                path: "/work/other-ca.pem".into(),
                roots: vec![w::Key("other-root".into())],
            },
        ],
        starter,
        reference,
        cases,
    }
}
pub fn definition() -> w::Definition {
    w::Definition {
        version: 1,
        initial: task("password-assignment", "example", "directory.example.test"),
        example: task("password-practice", "workshop", "practice.example.test"),
    }
}
