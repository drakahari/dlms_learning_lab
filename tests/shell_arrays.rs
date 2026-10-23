use dlms_learning_lab::shell::{self, Case, Vm};
use std::collections::BTreeMap;

fn case() -> Case {
    Case {
        name: "bounded indexed arrays".into(),
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
fn expect(c: &Case, source: &str, output: &str) {
    let r = shell::run(source, c);
    assert_eq!(r.error, None, "{source}: {r:?}");
    assert_eq!(r.status, 0, "{source}: {r:?}");
    assert_eq!(r.stdout, output, "{source}");
}
#[test]
fn dense_arrays_distinguish_empty_unset_scalar_and_one_empty_element() {
    expect(
        &case(),
        r#"
printf '%s\n' "${#missing[@]}"
empty=()
one=('')
scalar=''
list=('two words' '' '*')
printf '%s:%s:%s:%s\n' "${#empty[@]}" "${#one[@]}" "${#scalar[@]}" "${#list[@]}"
printf '<%s>\n' "${list[@]}"
printf '<%s>:<%s>:<%s>\n' "${list[0]}" "${list[2]}" "${list[4294967295]}"
"#,
        "0\n0:1:1:3\n<two words>\n<>\n<*>\n<two words>:<*>:<>\n",
    );
}
#[test]
fn compound_replacement_expands_against_prior_binding_and_preserves_quote_provenance() {
    let mut c = case();
    c.files.insert("/work/a.log".into(), "a\n".into());
    c.files.insert("/work/two words.log".into(), "b\n".into());
    expect(
        &c,
        r#"
list=(before tail)
list=("${list[1]}" "${list[0]}" "${list[@]}")
printf '<%s>\n' "${list[@]}"
pattern='*.log'
list=($pattern '*.log' "*.log")
printf '<%s>\n' "${list[@]}"
"#,
        "<tail>\n<before>\n<before>\n<tail>\n<a.log>\n<two words.log>\n<*.log>\n<*.log>\n",
    );
}
#[test]
fn assignment_read_and_loop_target_replace_only_element_zero() {
    let mut c = case();
    c.stdin = "from input\n".into();
    expect(
        &c,
        r#"
list=(first retained)
list=replaced
printf '%s:%s\n' "$list" "${list[1]}"
read -r list
printf '%s:%s\n' "$list" "${list[1]}"
for list in one two; do :; done
printf '%s:%s\n' "$list" "${list[1]}"
list=()
list=new
printf '%s:%s\n' "${#list[@]}" "${list[0]}"
"#,
        "replaced:retained\nfrom input:retained\ntwo:retained\n1:new\n",
    );
}
#[test]
fn function_locals_retain_array_type_dynamic_scope_and_restore_whole_values() {
    expect(
        &case(),
        r#"
list=(outer retained)
change() { list=child; }
work() {
  local list
  printf '%s\n' "${#list[@]}"
  list=(inside second)
  local list
  change
  printf '%s:%s\n' "$list" "${list[1]}"
}
work
printf '%s:%s\n' "$list" "${list[1]}"
work() { local list=local; printf '%s:%s\n' "${#list[@]}" "$list"; }
work
printf '%s:%s\n' "$list" "${list[1]}"
"#,
        "0\nchild:second\nouter:retained\n1:local\nouter:retained\n",
    );
}
#[test]
fn arrays_forward_arguments_loop_without_splitting_and_survive_pipeline_copies() {
    expect(
        &case(),
        r#"
list=('two words' '' '*')
show() { for value in "$@"; do printf '<%s>\n' "$value"; done; }
show "${list[@]}"
list=changed | sort
printf '<%s>\n' "${list[@]}"
"#,
        "<two words>\n<>\n<*>\n<two words>\n<>\n<*>\n",
    );
}
#[test]
fn scalar_access_is_used_by_arithmetic_and_locale_selection() {
    let mut c = case();
    c.files.insert("/work/a.txt".into(), String::new());
    expect(
        &c,
        "number=(4 ignored); printf '%s\\n' \"$((number + 2))\"; LC_ALL=(C ignored); printf '%s\\n' *.txt",
        "6\na.txt\n",
    );
}
#[test]
fn empty_array_expansion_produces_no_arguments_and_multiline_assignment_works() {
    expect(
        &case(),
        r#"
list=(
  first
  'second value'
)
printf '%s\n' "${#list[@]}"
list=()
show() { printf '%s\n' "$#"; }
show "${list[@]}"
show "${missing[@]}"
scalar=text
show "${scalar[@]}"
"#,
        "2\n0\n0\n1\n",
    );
}
#[test]
fn unsupported_array_forms_and_reserved_base_names_fail_honestly() {
    for source in [
        "list[0]=wrong",
        "list=([1]=wrong)",
        "list+=(wrong)",
        "printf '%s' \"${list[-1]}\"",
        "printf '%s' \"${list[01]}\"",
        "printf '%s' \"${list[1+1]}\"",
        "printf '%s' \"${list[index]}\"",
        "printf '%s' \"${list[4294967296]}\"",
        "printf '%s' ${list[@]}",
        "printf '%s' \"prefix${list[@]}\"",
        "printf '%s' \"${list[@]}suffix\"",
        "printf '%s' \"${list[@]}${list[@]}\"",
        "printf '%s' \"${list[*]}\"",
        "printf '%s' \"${list[@]:1}\"",
        "printf '%s' \"${!list[@]}\"",
        "printf '%s' \"${#list[0]}\"",
        "read -a list",
        "unset list",
        "declare -a list",
        "list=(one) printf wrong",
        "list=(one) | sort",
        "list=(one) > wrong",
        "IFS=(wrong)",
        "RANDOM=(wrong)",
        "printf '%s' \"${IFS[0]}\"",
        "printf '%s' \"${RANDOM[0]}\"",
    ] {
        let r = shell::run(source, &case());
        assert!(
            r.error
                .as_ref()
                .is_some_and(|e| e.to_lowercase().contains("unsupported")),
            "{source}: {r:?}"
        );
        assert!(r.files.is_empty(), "{source}");
        assert!(!r.stdout.contains("wrong"), "{source}");
    }
}
#[test]
fn syntax_errors_are_not_silent_array_success_and_function_syntax_still_works() {
    for source in [
        "list=(one",
        "list=(one; two)",
        "list=((nested))",
        "list= (one two)",
        "list=\t(one two)",
    ] {
        assert!(shell::run(source, &case()).error.is_some(), "{source}");
    }
    expect(&case(), "list() { printf okay; }; list", "okay");
}
#[test]
fn array_element_limits_include_expansion_and_stop_remains_cooperative() {
    let words = vec!["x"; 128].join(" ");
    expect(
        &case(),
        &format!("list=({words}); printf '%s' \"${{#list[@]}}\""),
        "128",
    );
    for source in [
        format!("list=({words} x)"),
        format!("list=({words}); list=(\"${{list[@]}}\" x)"),
    ] {
        let r = shell::run(&source, &case());
        assert!(r.error.as_ref().is_some_and(|e| e.contains("128")), "{r:?}");
    }
    let mut vm = Vm::new(
        "list=(one two); while true; do list=(\"${list[@]}\"); done",
        &case(),
    )
    .unwrap();
    vm.tick(12);
    assert!(!vm.done());
    vm.stop();
    assert!(vm.done());
    assert!(vm.outcome.error.unwrap().contains("Stopped"));
}
#[test]
fn retained_arrays_local_shadows_and_pipeline_snapshots_share_memory_budget() {
    let mut c = case();
    c.env.insert("large".into(), "x".repeat(60_000));
    for source in [
        "list=(\"$large\" \"$large\" \"$large\" \"$large\")",
        "list=(\"$large\"); f() { local list; list=(\"$large\"); f; }; f",
        "list=(\"$large\" \"$large\"); printf x | sort",
    ] {
        let r = shell::run(source, &c);
        assert!(
            r.error
                .as_ref()
                .is_some_and(|e| e.to_lowercase().contains("limit")),
            "{source}: {r:?}"
        );
    }
}

#[test]
fn scalar_element_and_count_expansions_compose_inside_quoted_words() {
    expect(
        &case(),
        r#"a=(one two); x=prefix; printf '%s\n' "${a[0]}-${a[1]}" "${x}${a[0]}" "${#a[@]}:${#a[@]}""#,
        "one-two\nprefixone\n2:2\n",
    );
}

#[test]
fn escaped_newline_preserves_compound_assignment_adjacency() {
    expect(
        &case(),
        "list=\\\n(one two); printf '%s' \"${list[1]}\"",
        "two",
    );
}

#[test]
fn repeated_pipeline_snapshots_charge_shared_work_before_cloning() {
    let mut c = case();
    c.env.insert("large".into(), "x".repeat(60_000));
    let source = format!(
        "list=(\"$large\"); while true; do {}; done",
        vec![":"; 16].join(" | ")
    );
    let r = shell::run(&source, &c);
    assert!(
        r.error
            .as_ref()
            .is_some_and(|e| e.contains("Expansion work limit")),
        "{r:?}"
    );
    for slice in [1, 16, 64] {
        let mut vm = Vm::new(&source, &c).unwrap();
        for _ in 0..2000 {
            vm.tick(slice);
            if vm.done() {
                break;
            }
        }
        assert!(vm.done());
        assert_eq!(vm.outcome, r);
    }
}
