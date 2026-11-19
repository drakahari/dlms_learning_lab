use dlms_learning_lab::{ldap_query as q, ldap_workspace as w};
#[path = "ldap_directory.rs"]
mod directory_fixture;
fn expected(s: &q::Source, indices: &[usize]) -> w::Expected {
    w::Expected::Entries {
        entries: indices
            .iter()
            .map(|i| {
                let mut e = s.entries[*i].clone();
                e.attributes
                    .retain(|a| matches!(a.name.to_ascii_lowercase().as_str(), "uid" | "cn"));
                e
            })
            .collect(),
    }
}
fn sources(variant: bool) -> q::Source {
    let mut s = directory_fixture::fixture();
    if variant {
        for (idx, new, cn) in [
            (2, "robin", "Robin Ellis"),
            (3, "morgan", "Morgan Bell"),
            (6, "taylor", "Taylor Miles"),
        ] {
            let e = &mut s.entries[idx];
            let old = e
                .attributes
                .iter()
                .find(|a| a.name == "uid")
                .unwrap()
                .values[0]
                .clone();
            e.dn =
                e.dn.replacen(&format!("uid={old}"), &format!("uid={new}"), 1);
            e.attributes
                .iter_mut()
                .find(|a| a.name == "uid")
                .unwrap()
                .values = vec![new.into()];
            e.attributes
                .iter_mut()
                .find(|a| a.name == "cn")
                .unwrap()
                .values = vec![cn.into()];
        }
    }
    let mut extra = s.entries[2].clone();
    extra.dn = extra.dn.replacen("ou=People", "ou=Teams", 1);
    s.entries.push(extra);
    s
}
pub fn definition() -> w::Definition {
    let primary = sources(false);
    let secondary = sources(true);
    fn task(id: &str, primary: q::Source, secondary: q::Source, uri: &str) -> w::Task {
        let cases=[("primary",primary.clone(),""), ("changed-input",secondary,""),("defaults",primary.clone(),"ldapsearch -x -LLL '(objectClass=person)' uid cn"),("immediate-children",primary,"ldapsearch -x -LLL -s one '(objectClass=person)' uid cn")].into_iter().map(|(id,s,probe)|{
   let expectation=expected(&s,if id=="immediate-children"{&[2,3]}else{&[2,3,6]});w::Case{id:id.into(),label:format!("Directory case {id}"),directories:vec![w::DirectorySource{uri:uri.into(),directory:s}],probe:probe.into(),expected:expectation,reason:"Use the People base, include nested people where required and select only uid/cn; an entry with the same uid exists in Teams.".into()}
  }).collect();
        w::Task{version:1,id:id.into(),instructions:"Repair the directory defaults and return every People subtree person's uid and cn, keeping Teams out.".into(),starter_config:format!("URI {uri}\nBASE ou=Teams,dc=example,dc=test\n"),starter_query:"ldapsearch -x -LLL '(uid=alex)' uid cn".into(),reference_config:format!("URI {uri}\nBASE ou=People,dc=example,dc=test\n"),reference_query:"ldapsearch -x -LLL '(objectClass=person)' uid cn".into(),cases}
    }
    w::Definition {
        version: 1,
        initial: task(
            "lpic2-directory-test",
            primary.clone(),
            secondary.clone(),
            "ldap://directory.example.test",
        ),
        example: task(
            "directory-example",
            secondary,
            primary,
            "ldap://practice.example.test",
        ),
    }
}
