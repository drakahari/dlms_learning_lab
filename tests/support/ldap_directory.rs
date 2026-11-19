use dlms_learning_lab::ldap_query as l;
use l::{Attribute as A, EntrySource as E};
pub fn attr(name: &str, values: &[&str]) -> A {
    A {
        name: name.into(),
        values: values.iter().map(|s| s.to_string()).collect(),
    }
}
fn entry(dn: &str, attrs: Vec<A>) -> E {
    E {
        dn: dn.into(),
        attributes: attrs,
    }
}
pub fn fixture() -> l::Source {
    l::Source {
        version: 1,
        suffix: "dc=example,dc=test".into(),
        entries: vec![
            entry(
                "dc=example,dc=test",
                vec![
                    attr("dc", &["example"]),
                    attr("objectClass", &["top", "domain"]),
                ],
            ),
            entry(
                "ou=People,dc=example,dc=test",
                vec![
                    attr("ou", &["People"]),
                    attr("objectClass", &["top", "organizationalUnit"]),
                ],
            ),
            entry(
                "uid=alex,ou=People,dc=example,dc=test",
                vec![
                    attr("uid", &["alex"]),
                    attr("cn", &["Alex Rivera"]),
                    attr("sn", &["Rivera"]),
                    attr("mail", &["alex@example.test", "a.rivera@example.test"]),
                    attr(
                        "objectClass",
                        &["top", "person", "organizationalPerson", "inetOrgPerson"],
                    ),
                ],
            ),
            entry(
                "uid=sam,ou=People,dc=example,dc=test",
                vec![
                    attr("uid", &["sam"]),
                    attr("cn", &["Sam * Visitor"]),
                    attr("sn", &["Visitor"]),
                    attr(
                        "objectClass",
                        &["top", "person", "organizationalPerson", "inetOrgPerson"],
                    ),
                ],
            ),
            entry(
                "ou=Teams,dc=example,dc=test",
                vec![
                    attr("ou", &["Teams"]),
                    attr("objectClass", &["top", "organizationalUnit"]),
                ],
            ),
            entry(
                "ou=Remote,ou=People,dc=example,dc=test",
                vec![
                    attr("ou", &["Remote"]),
                    attr("objectClass", &["top", "organizationalUnit"]),
                ],
            ),
            entry(
                "uid=lee,ou=Remote,ou=People,dc=example,dc=test",
                vec![
                    attr("uid", &["lee"]),
                    attr("cn", &["Lee Patel"]),
                    attr("sn", &["Patel"]),
                    attr(
                        "objectClass",
                        &["top", "person", "organizationalPerson", "inetOrgPerson"],
                    ),
                ],
            ),
        ],
    }
}
