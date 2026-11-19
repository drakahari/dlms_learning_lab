//! Pending controlled activity for engine/UI tests; not a registered lab.
use dlms_learning_lab::{
    engine::{Goal, State},
    ldap_access_workspace as w,
    pack::Scenario,
};
pub fn scenario() -> Scenario {
    let d: w::Definition =
        serde_json::from_str(include_str!("../fixtures/ldap_access_authored.json")).unwrap();
    let source:Scenario=serde_json::from_value(serde_json::json!({
  "id":d.initial.id,"version":1,"unit":"ldap-access-policy","track":"LPIC-2","difficulty":"Intermediate","minutes":35,
  "title":"Delegate contact changes safely","topic":"Ordered directory write controls","summary":"Controlled pending activity fixture, not registered curriculum.",
  "objective":d.initial.instructions,"steps":["Repair the write rules and verify allowed and denied contact changes."],"guide":[],
  "prerequisites":["lpic2-ldap-protected-password"],
  "explanation":"The first matching target rule selects the relevant fields; its first matching caller clause decides access. Restrict description and mail separately, put the specific desk/self callers before the catch-all, and deny unrelated writes. Every field in a combined Modify must be authorized before any data commits.",
  "hints":["Inspect the first rule that matches a wrongly allowed change.","The desk identity needs description permission, while self can edit their own two contact fields.","Keep Archive and Service outside the writable target branch, including same-uid entries."],
  "sources":["https://www.openldap.org/doc/admin26/access-control.html"],
  "simplifications":"Given established identities and bounded static ACL/Modify behavior only; not complete server configuration, fresh Bind/search, schema, add/delete or password authorization. No host command/files/network.",
  "solution":[format!("{}{}",w::PREFIX,serde_json::json!({"policy":d.initial.reference}))],
  "initial":State::LdapAccess{simulation:Box::new(w::Session::new(&d).unwrap())},"goal":Goal::LdapAccess{definition:Box::new(d.clone())}
 })).unwrap();
    w::source(&source).unwrap();
    source
}
