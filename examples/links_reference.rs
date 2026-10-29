//! Fixed pure in-process inode observations, never host path or command execution.
#[path = "support/automation_digest.rs"]
mod digest;
#[allow(dead_code)] // This narrow probe exercises only the documented comparison API.
#[path = "../src/links.rs"]
mod links;
use links::{Kind, State};
use serde_json::{Value, json};
fn fresh() -> State {
    State::new(&["/", "/a", "/b", "/real"], "/").unwrap()
}
fn same(s: &State, a: &str, b: &str) -> bool {
    s.stat(a).unwrap().inode == s.stat(b).unwrap().inode
}
fn observation(id: &str, fields: Value) -> Value {
    json!({"id":id,"observations":fields})
}
fn main() {
    assert_eq!(
        std::env::args_os().count(),
        1,
        "This fixed probe has no arguments/selectors"
    );
    assert_eq!(links::VERSION, 1);
    let mut rows = Vec::new();
    {
        let mut s = fresh();
        s.create_file("item", "seed\n").unwrap();
        s.hard_link("item", "alias").unwrap();
        s.copy_file("item", "copy").unwrap();
        let identity = same(&s, "item", "alias");
        let separate = !same(&s, "item", "copy");
        let count = s.stat("item").unwrap().links;
        let copy_count = s.stat("copy").unwrap().links;
        s.write_file("alias", "updated\n").unwrap();
        rows.push(observation("shared-alias-independent-copy",json!({"same_alias":identity,"copy_distinct":separate,"alias_links":count,"copy_links":copy_count,"original":s.read_file("item").unwrap(),"alias":s.read_file("alias").unwrap(),"copy":s.read_file("copy").unwrap()})));
    }
    {
        let mut s = fresh();
        s.create_file("item", "payload\n").unwrap();
        s.hard_link("item", "alias").unwrap();
        let before = s.stat("item").unwrap().inode;
        s.unlink("item").unwrap();
        let alias = s.stat("alias").unwrap();
        let content = s.read_file("alias").unwrap();
        let removed = format!("{:?}", s.read_file("item").unwrap_err().kind);
        s.unlink("alias").unwrap();
        rows.push(observation("unlink-one-then-last-name",json!({"alias_identity_preserved":before==alias.inode,"remaining_links":alias.links,"content":content,"removed_name":removed,"last_name":format!("{:?}",s.read_file("alias").unwrap_err().kind)})));
    }
    {
        let mut s = fresh();
        s.create_file("a/data", "alpha\n").unwrap();
        s.create_file("b/data", "beta\n").unwrap();
        s.symbolic_link("data", "a/ref").unwrap();
        let before = s.stat("a/ref").unwrap();
        let text = s.read_file("a/ref").unwrap();
        s.move_entry("a/ref", "b/ref").unwrap();
        let after = s.stat("b/ref").unwrap();
        rows.push(observation("move-relative-link-changes-parent",json!({"before":text,"after":s.read_file("b/ref").unwrap(),"target":s.read_link("b/ref").unwrap(),"same_link_identity":before.inode==after.inode,"kind_symbolic":after.kind==Kind::SymbolicLink,"old_name":format!("{:?}",s.stat("a/ref").unwrap_err().kind)})));
    }
    {
        let mut s = fresh();
        s.symbolic_link("later", "latest").unwrap();
        let link = s.stat("latest").unwrap();
        let missing = format!("{:?}", s.read_file("latest").unwrap_err().kind);
        s.create_file("later", "arrived\n").unwrap();
        let arrived = s.read_file("latest").unwrap();
        s.unlink("later").unwrap();
        rows.push(observation("dangling-link-arrival-retirement",json!({"kind_symbolic":link.kind==Kind::SymbolicLink,"target":s.read_link("latest").unwrap(),"target_bytes":link.size_bytes,"before":missing,"arrived":arrived,"after":format!("{:?}",s.read_file("latest").unwrap_err().kind)})));
    }
    {
        let mut s = fresh();
        s.create_file("real/data", "nested\n").unwrap();
        s.symbolic_link("real", "shortcut").unwrap();
        let via = s.stat_follow("shortcut/data").unwrap();
        let direct = s.stat("real/data").unwrap();
        rows.push(observation("internal-directory-link-prefix",json!({"same_target":via.inode==direct.inode,"content":s.read_file("shortcut/data").unwrap(),"raw_target":s.read_link("shortcut").unwrap(),"default_symbolic":s.stat("shortcut").unwrap().kind==Kind::SymbolicLink,"follow_directory":s.stat_follow("shortcut").unwrap().kind==Kind::Directory,"trailing_slash_directory":s.stat("shortcut/").unwrap().kind==Kind::Directory})));
    }
    {
        let mut s = fresh();
        s.create_file("original", "first\n").unwrap();
        s.symbolic_link("original", "view").unwrap();
        s.copy_file("view", "copy").unwrap();
        let separate = !same(&s, "original", "copy");
        s.write_file("original", "second\n").unwrap();
        rows.push(observation("copy-follows-source-with-independent-data",json!({"copy_distinct":separate,"copy_regular":s.stat("copy").unwrap().kind==Kind::Regular,"source_symbolic":s.stat("view").unwrap().kind==Kind::SymbolicLink,"view":s.read_file("view").unwrap(),"copy":s.read_file("copy").unwrap(),"copy_links":s.stat("copy").unwrap().links})));
    }
    {
        let mut s = fresh();
        s.create_file("item", "protected\n").unwrap();
        s.create_file("existing", "existing\n").unwrap();
        let before = s.clone();
        let exists = format!("{:?}", s.hard_link("item", "existing").unwrap_err().kind);
        let atomic = s == before;
        rows.push(observation("fixed-file-errors-preserve-content",json!({"existing_destination":exists,"file_component":format!("{:?}",s.read_file("item/child").unwrap_err().kind),"file_trailing_slash":format!("{:?}",s.stat_follow("item/").unwrap_err().kind),"missing":format!("{:?}",s.read_file("absent").unwrap_err().kind),"source":s.read_file("item").unwrap(),"existing":s.read_file("existing").unwrap(),"atomic":atomic})));
    }
    {
        let mut s = fresh();
        s.create_file("-odd name", "é\n").unwrap();
        s.hard_link("-odd name", "alias name").unwrap();
        s.symbolic_link("-odd name", "view name").unwrap();
        rows.push(observation("utf8-data-and-literal-names",json!({"same_alias":same(&s,"-odd name","alias name"),"data_bytes":s.stat("-odd name").unwrap().size_bytes,"target_bytes":s.stat("view name").unwrap().size_bytes,"target":s.read_link("view name").unwrap(),"content":s.read_file("view name").unwrap(),"links":s.stat("alias name").unwrap().links})));
    }
    let mut only = fresh();
    only.create_file("item", "safe\n").unwrap();
    only.symbolic_link("/outside", "absolute").unwrap();
    only.symbolic_link("cycle-b", "cycle-a").unwrap();
    only.symbolic_link("cycle-a", "cycle-b").unwrap();
    let vm_only = json!({"absolute_target_missing":format!("{:?}",only.read_file("absolute").unwrap_err().kind),"root_parent_escape":format!("{:?}",only.read_file("/../../outside").unwrap_err().kind),"hard_directory":format!("{:?}",only.hard_link("real","bad-directory-alias").unwrap_err().kind),"hard_symbolic":format!("{:?}",only.hard_link("absolute","bad-link-alias").unwrap_err().kind),"cycle":format!("{:?}",only.read_file("cycle-a").unwrap_err().kind),"oversized_target":format!("{:?}",only.symbolic_link(&"x".repeat(1025),"too-large").unwrap_err().kind)});
    let fixture: Value = serde_json::from_slice(include_bytes!(
        "../tests/fixtures/links-fixed-reference.json"
    ))
    .unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), rows.len());
    assert!(fixture["actual_comparison"].is_null());
    println!(
        "{}",
        json!({"schema_version":1,"core_version":links::VERSION,"compiled_inputs":[{"path":"src/links.rs","sha256":digest::hash(include_bytes!("../src/links.rs"))},{"path":"examples/links_reference.rs","sha256":digest::hash(include_bytes!("links_reference.rs"))},{"path":"examples/support/automation_digest.rs","sha256":digest::hash(include_bytes!("support/automation_digest.rs"))},{"path":"tests/fixtures/links-fixed-reference.json","sha256":digest::hash(include_bytes!("../tests/fixtures/links-fixed-reference.json"))},{"path":"Cargo.toml","sha256":digest::hash(include_bytes!("../Cargo.toml"))},{"path":"Cargo.lock","sha256":digest::hash(include_bytes!("../Cargo.lock"))}],"cases":rows,"vm_only":vm_only})
    );
}
