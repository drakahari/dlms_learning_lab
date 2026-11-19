use dlms_learning_lab::{
    ldap_schema::{EntrySource, Field},
    ldap_schema_workspace::{Case, Definition, Operation, Probe, Task},
};
fn values(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| (*s).into()).collect()
}
fn field(name: &str, vs: &[&str]) -> Field {
    Field {
        name: name.into(),
        values: values(vs),
    }
}
fn entry(class: &str, code: &str, root: &str, name: &str, badge: &str) -> EntrySource {
    EntrySource {
        dn: format!("cn={name},{root}"),
        classes: vec![class.into()],
        fields: vec![
            field("cn", &[name]),
            field("sn", &["Learner"]),
            field(code, &[badge]),
        ],
    }
}
fn probe(
    label: &str,
    reason: &str,
    operation: Operation,
    entry: EntrySource,
    accept: bool,
) -> Probe {
    Probe {
        label: label.into(),
        reason: reason.into(),
        operation,
        entry,
        accept,
    }
}
fn task(prefix: &str, branch: &str, root: &str) -> Task {
    let class = format!("{prefix}Person");
    let code = format!("{prefix}Badge");
    let count = format!("{prefix}Points");
    let header = format!(
        "version: 1\n\ndn: cn={prefix},cn=schema,cn=config\nchangetype: add\nobjectClass: olcSchemaConfig\ncn: {prefix}\n"
    );
    let reference = format!(
        "{header}olcAttributeTypes: ({branch}.1 NAME '{code}' EQUALITY caseExactMatch SYNTAX 1.3.6.1.4.1.1466.115.121.1.15 SINGLE-VALUE)\nolcAttributeTypes: ({branch}.2 NAME '{count}' EQUALITY integerMatch SYNTAX 1.3.6.1.4.1.1466.115.121.1.27)\nolcObjectClasses: ({branch}.3 NAME '{class}' SUP person STRUCTURAL MUST {code} MAY (uid $ {count}))\n"
    );
    let starter = reference
        .replace(" SINGLE-VALUE", "")
        .replace(
            "EQUALITY integerMatch SYNTAX 1.3.6.1.4.1.1466.115.121.1.27",
            "EQUALITY caseExactMatch SYNTAX 1.3.6.1.4.1.1466.115.121.1.15",
        )
        .replace(
            &format!("MUST {code} MAY (uid $ {count})"),
            &format!("MAY (uid $ {code} $ {count})"),
        );
    let fold = |s: String| {
        s.replace(" NAME", "\n  NAME")
            .replace(" MAY", "\n  MAY")
            .replace(" EQUALITY", "\n  EQUALITY")
            .replace(" SYNTAX", "\n  SYNTAX")
            .replace(" SINGLE-VALUE", "\n  SINGLE-VALUE")
            .replace(" SUP person", "\n  SUP person")
            .replace(&format!(" MUST {code}"), &format!("\n  MUST {code}"))
    };
    let reference = fold(reference);
    let starter = fold(starter);
    let mut cases = Vec::new();
    for (n, (name, badge)) in [("Mira", "LAB-A"), ("Eli", "LAB-B"), ("Noor", "LAB-C")]
        .into_iter()
        .enumerate()
    {
        let base = entry(&class, &code, root, name, badge);
        let existing = entry(&class, &code, root, &format!("Existing{n}"), "KEPT");
        let mut probes = Vec::new();
        let mut full = base.clone();
        full.classes.push("person".into());
        full.fields.push(field(&count, &["7", "9"]));
        full.fields
            .push(field("uid", &["learner-one", "learner-two"]));
        full.fields
            .push(field("description", &["Enrolled learner"]));
        probes.push(probe(
            "Accept complete data",
            "Optional points and identifiers may have multiple distinct values.",
            Operation::Add,
            full,
            true,
        ));
        probes.push(probe(
            "Accept without optional fields",
            "Only cn, sn and the badge are required.",
            Operation::Add,
            base.clone(),
            true,
        ));
        let mut p = base.clone();
        p.fields.pop();
        probes.push(probe(
            "Reject missing badge",
            "The custom class requires its badge through MUST.",
            Operation::Add,
            p,
            false,
        ));
        let mut p = base.clone();
        p.fields[2].values.push("SECOND".into());
        probes.push(probe(
            "Reject two badges",
            "The badge type is SINGLE-VALUE.",
            Operation::Add,
            p,
            false,
        ));
        let mut p = base.clone();
        p.fields.push(field(&count, &["many"]));
        probes.push(probe(
            "Reject words as points",
            "Integer syntax accepts canonical whole-number text, not words.",
            Operation::Add,
            p,
            false,
        ));
        let mut p = base.clone();
        p.fields.remove(1);
        probes.push(probe(
            "Reject missing inherited surname",
            "SUP person inherits the sn requirement.",
            Operation::Add,
            p,
            false,
        ));
        let mut p = base.clone();
        p.fields.push(field("name", &["unexpected"]));
        probes.push(probe(
            "Reject an unlisted attribute",
            "Attributes need permission through the class's inherited or local MUST/MAY sets.",
            Operation::Add,
            p,
            false,
        ));
        let mut p = existing.clone();
        p.fields[2].values = vec!["REVISED".into()];
        probes.push(probe(
            "Update existing badge",
            "A valid change must alter the actual stored entry.",
            Operation::Replace,
            p,
            true,
        ));
        let mut p = existing.clone();
        p.fields.pop();
        probes.push(probe(
            "Keep data after a bad update",
            "Removing every badge must fail without changing stored fields.",
            Operation::Replace,
            p,
            false,
        ));
        let mut p = base.clone();
        p.fields.push(field(&count, &["7", "7"]));
        probes.push(probe(
            "Reject duplicate points",
            "Equal values cannot appear twice in one LDAP attribute.",
            Operation::Add,
            p,
            false,
        ));
        let mut p = base.clone();
        p.fields.push(field(&count, &["0007"]));
        probes.push(probe(
            "Reject noncanonical integers",
            "Do not silently convert a leading-zero number into valid LDAP Integer text.",
            Operation::Add,
            p,
            false,
        ));
        cases.push(Case {
            id: format!("{prefix}-set-{n}"),
            label: format!("Sample set {}", n + 1),
            initial: vec![existing],
            probes,
        });
    }
    Task {
        id: format!("{prefix}-schema"),
        instructions: format!(
            "Extend the directory with {class}: a required single text badge, optional whole-number points and inherited person fields. Preserve existing entries during rejected updates."
        ),
        schema_name: prefix.into(),
        oid_branch: branch.into(),
        starter,
        reference,
        cases,
    }
}
pub fn fixture() -> Definition {
    Definition {
        version: 1,
        initial: task("training", "1.3.6.1.4.1.32473.220", "dc=registry,dc=test"),
        example: task("workshop", "1.3.6.1.4.1.32473.221", "dc=classroom,dc=test"),
    }
}
