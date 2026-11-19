//! Real named-map evaluation in the bounded VM; no host subprocesses.
use dlms_learning_lab::shell::{self, Case, Vm};
use std::collections::BTreeMap;
fn case() -> Case {
    Case {
        name: "named map fixture".into(),
        args: vec![],
        stdin: String::new(),
        env: BTreeMap::new(),
        files: BTreeMap::new(),
        stdout: String::new(),
        stderr: String::new(),
        status: 0,
        after: BTreeMap::new(),
    }
}
fn expect(source: &str, stdout: &str) {
    let r = shell::run(source, &case());
    assert_eq!(r.error, None, "{source}: {r:?}");
    assert_eq!(
        (r.stdout.as_str(), r.stderr.as_str(), r.status),
        (stdout, "", 0),
        "{source}"
    );
}
#[test]
fn declare_store_replace_count_and_missing_are_real_state() {
    expect(
        r#"declare -A sites
sites[oak]='two words'
sites[elm]=''
printf '<%s>:<%s>:%s\n' "${sites[oak]}" "${sites[absent]}" "${#sites[@]}"
sites[oak]=updated
declare -A sites
printf '%s:%s:%s\n' "${sites[oak]}" "${sites[elm]}" "${#sites[@]}"
"#,
        "<two words>:<>:2\nupdated::2\n",
    );
}
#[test]
fn decimal_looking_keys_are_strings_and_plain_access_uses_zero() {
    expect(
        r#"declare -A map
map[01]=leading
map[1]=one
map[0]=zero
printf '%s:%s:%s:%s\n' "${map[01]}" "${map[1]}" "$map" "${#map[@]}"
map=replaced
printf '%s:%s:%s\n' "${map[0]}" "${map[01]}" "${#map[@]}"
"#,
        "leading:one:zero:3\nreplaced:leading:3\n",
    );
}
#[test]
fn dynamic_quoted_keys_and_values_are_data_not_code_or_patterns() {
    let mut c = case();
    c.env.insert("KEY".into(), "space $name ] ; ' quote".into());
    c.env.insert("VALUE".into(), "a  b * %s\nlast".into());
    let r = shell::run(
        r#"declare -A map
map["$KEY"]="$VALUE"
printf '<%s>:%s\n' "${map[$KEY]}" "${#map[@]}"
"#,
        &c,
    );
    assert_eq!(r.error, None, "{r:?}");
    assert_eq!(r.stdout, "<a  b * %s\nlast>:1\n");
    assert!(r.files.is_empty());
    expect(
        r#"declare -A map
map['two words']='literal $VALUE'
printf '%s\n' "${map['two words']}"
"#,
        "literal $VALUE\n",
    );
}
#[test]
fn ordinary_read_and_loop_targets_update_key_zero_only() {
    let mut c = case();
    c.stdin = "from input\n".into();
    let r = shell::run(
        r#"declare -A map
map[kept]=retain
read -r map
printf '%s:%s\n' "$map" "${map[kept]}"
for map in first last; do :; done
printf '%s:%s:%s\n' "${map[0]}" "${map[kept]}" "${#map[@]}"
"#,
        &c,
    );
    assert_eq!(r.error, None, "{r:?}");
    assert_eq!(r.stdout, "from input:retain\nlast:retain:2\n");
}
#[test]
fn global_map_changes_survive_calls_but_not_isolated_capture_or_pipeline() {
    expect(
        r#"declare -A map
map[oak]=outer
change() { map[oak]=child; }
change
printf '%s\n' "${map[oak]}"
value=$(map[oak]=captured; printf '%s' "${map[oak]}")
printf '%s:%s\n' "$value" "${map[oak]}"
map[oak]=pipeline | cat
printf '%s\n' "${map[oak]}"
"#,
        "child\ncaptured:child\nchild\n",
    );
}
#[test]
fn accumulated_updates_replace_duplicates_without_losing_empty_values() {
    let mut c = case();
    c.stdin = "oak|first\nelm|quiet\noak|revised\nempty|\n".into();
    let r = shell::run(
        r#"declare -A map
while IFS= read -r line; do
 key=$(printf '%s\n' "$line" | cut -d '|' -f 1)
 value=$(printf '%s\n' "$line" | cut -d '|' -f 2)
 if [ -n "$key" ]; then map["$key"]="$value"; fi
done
for key in oak elm empty missing; do printf '%s=<%s>\n' "$key" "${map[$key]}"; done
printf 'count=%s\n' "${#map[@]}"
"#,
        &c,
    );
    assert_eq!(r.error, None, "{r:?}");
    assert_eq!(
        r.stdout,
        "oak=<revised>\nelm=<quiet>\nempty=<>\nmissing=<>\ncount=3\n"
    );
}
#[test]
fn unsupported_forms_are_explicit_and_never_canned_success() {
    for source in [
        "declare -a map",
        "name=map; declare -A \"$name\"",
        "x=scalar; declare -A x",
        "x=(indexed); declare -A x",
        "f() { declare -A map; }; f",
        "declare -A map; f() { local map; }; f",
        "declare -A map; map=(a b)",
        "declare -A map; printf '%s' \"${map[@]}\"",
        "declare -A map; printf '%s' \"${!map[@]}\"",
        "declare -A map; map[oak]=$(printf hi)",
        "declare -A map; map[oak]=value > marker",
        "declare -A map; export map",
        "export SITE=oak; declare -A map",
        "set -x; declare -A map",
        "declare -A map; set -x",
        "declare -A map; map['@']=value",
        "declare -A map; key='*'; map[\"$key\"]=value",
        "declare -A map; map[\"pre$key\"]=value",
    ] {
        let r = shell::run(source, &case());
        assert!(
            r.error
                .as_ref()
                .is_some_and(|s| s.to_lowercase().contains("unsupported")),
            "{source}: {r:?}"
        );
        assert!(r.files.is_empty(), "{source}: {r:?}");
    }
}
#[test]
fn empty_or_long_keys_explain_the_problem_and_do_not_write_files() {
    for key in [String::new(), "x".repeat(129)] {
        let mut c = case();
        c.env.insert("KEY".into(), key.clone());
        let r = shell::run("declare -A map; map[\"$KEY\"]=value; printf wrong", &c);
        let e = r.error.unwrap();
        assert!(
            e.contains(if key.is_empty() {
                "cannot be empty"
            } else {
                "128 UTF-8 bytes"
            }),
            "{e}"
        );
        assert!(r.stdout.is_empty());
        assert!(r.files.is_empty());
    }
}
#[test]
fn entry_and_memory_limits_are_bounded_and_stop_remains_responsive() {
    let mut c = case();
    c.args = (0..64).map(|n| format!("key{n}")).collect();
    let r = shell::run(
        "declare -A map; for key in \"$@\"; do map[\"$key\"]=value; second=extra$key; map[\"$second\"]=value; done; map[overflow]=value",
        &c,
    );
    assert!(r.error.as_ref().is_some_and(|s| s.contains("128")), "{r:?}");
    let mut c = case();
    c.env.insert("VALUE".into(), "x".repeat(5000));
    c.args = (0..64).map(|n| format!("key{n}")).collect();
    let r = shell::run(
        "declare -A map; for key in \"$@\"; do map[\"$key\"]=\"$VALUE\"; done",
        &c,
    );
    assert!(
        r.error.as_ref().is_some_and(|s| s.contains("memory")),
        "{r:?}"
    );
    let mut vm = Vm::new(
        "declare -A map; while true; do map[oak]=updated; done",
        &case(),
    )
    .unwrap();
    vm.tick(20);
    assert!(!vm.done());
    vm.stop();
    assert!(vm.done());
    assert!(
        vm.outcome
            .error
            .as_ref()
            .is_some_and(|s| s.contains("Stopped"))
    );
}
#[test]
fn legacy_bare_indexed_subscript_rejection_still_precedes_prior_effects() {
    for source in [
        "printf wrong; printf '%s' ${list[index]}",
        "printf wrong; printf '%s' ${list[01]}",
        "printf wrong; printf '%s' ${list[1+1]}",
    ] {
        let r = shell::run(source, &case());
        assert!(
            r.error.as_ref().is_some_and(|s| s.contains("Unsupported")),
            "{r:?}"
        );
        assert!(r.stdout.is_empty());
    }
}

#[test]
fn excluded_map_replacement_refuses_before_rhs_captures_or_redirects() {
    for source in [
        "declare -A map; map=($(printf changed > marker)); printf wrong",
        "declare -A map; map=(\"$(printf changed > marker)\"); printf wrong",
        "declare -A map; map[oak]=$(printf changed > marker); printf wrong",
    ] {
        let r = shell::run(source, &case());
        assert!(
            r.error.as_ref().is_some_and(|s| s.contains("Unsupported")),
            "{source}: {r:?}"
        );
        assert!(r.files.is_empty(), "{source}: {r:?}");
        assert!(r.stdout.is_empty(), "{source}: {r:?}");
    }
}
#[test]
fn excluded_special_key_variables_never_masquerade_as_missing_input() {
    for source in [
        "declare -A map; map[\"$IFS\"]=value",
        "declare -A map; printf '%s' \"${map[$IFS]}\"",
    ] {
        let r = shell::run(source, &case());
        assert!(
            r.error
                .as_ref()
                .is_some_and(|s| s.contains("Unsupported custom IFS")),
            "{source}: {r:?}"
        );
        assert!(r.files.is_empty());
    }
}

#[test]
fn expanded_key_metacharacters_are_never_reparsed_or_used_as_host_paths() {
    for key in [
        "$(printf injected)",
        "../../outside",
        "x]; printf injected; #",
        "`printf injected`",
        "space\nline",
    ] {
        let mut c = case();
        c.env.insert("KEY".into(), key.into());
        let r = shell::run(
            "declare -A map; map[\"$KEY\"]=retained; printf '%s:%s\\n' \"${map[$KEY]}\" \"${#map[@]}\"",
            &c,
        );
        assert_eq!(r.error, None, "{key:?}: {r:?}");
        assert_eq!(r.stdout, "retained:1\n");
        assert!(r.files.is_empty());
    }
}
