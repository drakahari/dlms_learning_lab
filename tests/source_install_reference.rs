//! Explicit development-only comparison. All host inputs/recipes below are fixed,
//! owned fixtures. Nothing comes from learner drafts, packs or personal files.
#![cfg(target_os = "linux")]
use dlms_learning_lab::source_install::{Configuration, Facts, Output, Target, World, command};
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
const PROGRAM: &str = r#"#include <stdio.h>
#include <string.h>
#include <ctype.h>
int main(int argc, char **argv) {
    if (argc != 2 || (argv[1][0] == '-' && strcmp(argv[1], "--version") && strcmp(argv[1], "--prefix"))) {
        fputs("Usage: greet NAME | --version | --prefix\n", stderr); return 2;
    }
    if (!strcmp(argv[1], "--version")) { puts("greet 1.4"); return 0; }
    if (!strcmp(argv[1], "--prefix")) { puts(PREFIX); return 0; }
    char text[1024];
    if (snprintf(text, sizeof(text), "%s %s\n", GREETING, argv[1]) >= (int)sizeof(text)) return 2;
    if (UPPERCASE) for (char *p=text; *p; ++p) *p = (char)toupper((unsigned char)*p);
    fputs(text, stdout); return 0;
}
"#;
const MAKEFILE: &str = r#"include config.mk
CC := /usr/bin/cc
.DEFAULT_GOAL := all
.PHONY: all check install
all: greet
greet: greet.c config.h
	$(CC) -std=c11 -include config.h greet.c -o greet
check: all
	./greet Ada > /dev/null
	./greet Rio > /dev/null
install: all
	/usr/bin/install -d "$(DESTDIR)$(prefix)/bin" "$(DESTDIR)$(prefix)/share/doc/greet"
	/usr/bin/install -m 0755 greet "$(DESTDIR)$(prefix)/bin/greet"
	/usr/bin/install -m 0644 README "$(DESTDIR)$(prefix)/share/doc/greet/README"
"#;
fn execute(root: &Path, program: &Path, args: &[&str]) -> std::process::Output {
    let mut child = Command::new(program)
        .args(args)
        .current_dir(root)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("LC_ALL", "C")
        .env("HOME", root)
        .env("TMPDIR", root.join("tmp"))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let start = Instant::now();
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if start.elapsed() > Duration::from_secs(45) {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "Owned fixed-fixture tool exceeded45s; evidence retained at {}",
                root.display()
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let out = child.wait_with_output().unwrap();
    assert!(
        out.stdout.len() + out.stderr.len() <= 1_048_576,
        "Unexpectedly large fixed-fixture output"
    );
    out
}
fn header(root: &Path, prefix: &str, greeting: &str, uppercase: bool) {
    // All callers use fixed strings below, never an external argument/file.
    assert!(["/usr/local", "/opt/tools"].contains(&prefix));
    assert!(["Welcome,", "Good day,"].contains(&greeting));
    fs::write(
        root.join("config.h"),
        format!(
            "#define PREFIX {prefix:?}\n#define GREETING {greeting:?}\n#define UPPERCASE {}\n",
            u8::from(uppercase)
        ),
    )
    .unwrap();
    fs::write(root.join("config.mk"), format!("prefix := {prefix}\n")).unwrap();
}
fn make(root: &Path, stage: &Path, target: &str) {
    assert!(stage.starts_with(root) && stage.is_absolute());
    assert!(["install", "check"].contains(&target));
    let variable = format!("DESTDIR={}", stage.display());
    let out = execute(
        root,
        Path::new("/usr/bin/make"),
        &["--jobs=1", "--no-print-directory", target, &variable],
    );
    fs::write(
        root.join(format!(
            "make-{target}-{}.log",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )),
        [out.stdout.clone(), out.stderr.clone()].concat(),
    )
    .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
fn compare(
    root: &Path,
    binary: &Path,
    world: &World,
    path: &str,
    phase: &str,
    records: &mut Vec<serde_json::Value>,
) {
    for args in [
        vec!["Mira"],
        vec!["Lee Tran"],
        vec!["--prefix"],
        vec!["--version"],
        vec!["--unsupported"],
        vec![],
    ] {
        let actual = execute(root, binary, &args);
        let actual = Output {
            stdout: String::from_utf8(actual.stdout).unwrap(),
            stderr: String::from_utf8(actual.stderr).unwrap(),
            status: actual.status.code().unwrap(),
        };
        let expected = world
            .run(
                path,
                &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            )
            .unwrap();
        assert_eq!(actual, expected, "{phase}: {args:?}");
        records.push(serde_json::json!({"phase":phase,"arguments":args,"stdout":actual.stdout,"stderr":actual.stderr,"status":actual.status}));
    }
}
#[test]
#[ignore = "Controlled GNU make/C fixture; run explicitly with MAIN tools/dev.py and --ignored"]
fn fixed_gnu_make_fixture_matches_prefix_staging_dependencies_and_stale_copies() {
    let project = Path::new(env!("CARGO_MANIFEST_DIR"));
    let base = project.join(".build-artifacts/tmp");
    assert_eq!(base.canonicalize().unwrap(), base);
    let root = base.join(format!(
        "source-install-reference-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("tmp")).unwrap();
    for tool in ["/usr/bin/make", "/usr/bin/cc", "/usr/bin/install"] {
        assert!(
            Path::new(tool).is_file(),
            "Missing preinstalled development tool: {tool}; install nothing automatically."
        );
    }
    let make_version = execute(&root, Path::new("/usr/bin/make"), &["--version"]);
    let cc_version = execute(&root, Path::new("/usr/bin/cc"), &["--version"]);
    assert!(make_version.status.success() && cc_version.status.success());
    fs::write(root.join("greet.c"), PROGRAM).unwrap();
    fs::write(root.join("Makefile"), MAKEFILE).unwrap();
    let facts: Facts =
        serde_json::from_str(include_str!("fixtures/source-install-project-v1.json")).unwrap();
    fs::write(root.join("README"), &facts.manual).unwrap();
    let stage = root.join("stage");
    let mut world = World::new(&facts).unwrap();
    command::prepare_directories(&mut world, &["/usr/src", "/stage"]).unwrap();
    world.extract(&facts, "/usr/src").unwrap();
    let mut records = Vec::new();
    header(&root, "/usr/local", "Welcome,", false);
    world
        .configure(
            &facts,
            Configuration {
                prefix: "/usr/local".into(),
                greeting: "Welcome,".into(),
                uppercase: false,
            },
        )
        .unwrap();
    make(&root, &stage, "install");
    world.make(&facts, Target::Install, Some("/stage")).unwrap();
    compare(
        &root,
        &stage.join("usr/local/bin/greet"),
        &world,
        "/stage/usr/local/bin/greet",
        "install-builds-dependency",
        &mut records,
    );
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
        fs::metadata(stage.join("usr/local/bin/greet"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o755
    );
    assert_eq!(
        fs::metadata(stage.join("usr/local/share/doc/greet/README"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o644
    );
    assert_eq!(
        fs::read_to_string(stage.join("usr/local/share/doc/greet/README")).unwrap(),
        facts.manual
    );
    // Explicit fixture aging makes the dependency relationship deterministic even
    // on coarse timestamp filesystems. Only the newly owned fixture is touched.
    fs::File::open(root.join("greet"))
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_secs(1000)))
        .unwrap();
    header(&root, "/usr/local", "Good day,", true);
    world
        .configure(
            &facts,
            Configuration {
                prefix: "/usr/local".into(),
                greeting: "Good day,".into(),
                uppercase: true,
            },
        )
        .unwrap();
    make(&root, &stage, "check");
    world.make(&facts, Target::Check, None).unwrap();
    compare(
        &root,
        &root.join("greet"),
        &world,
        "/usr/src/greet-1.4/greet",
        "rebuilt-current",
        &mut records,
    );
    compare(
        &root,
        &stage.join("usr/local/bin/greet"),
        &world,
        "/stage/usr/local/bin/greet",
        "staged-copy-stays-old",
        &mut records,
    );
    make(&root, &stage, "install");
    world.make(&facts, Target::Install, Some("/stage")).unwrap();
    compare(
        &root,
        &stage.join("usr/local/bin/greet"),
        &world,
        "/stage/usr/local/bin/greet",
        "restaged-current",
        &mut records,
    );
    fs::File::open(root.join("greet"))
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_secs(1000)))
        .unwrap();
    header(&root, "/opt/tools", "Welcome,", false);
    world
        .configure(
            &facts,
            Configuration {
                prefix: "/opt/tools".into(),
                greeting: "Welcome,".into(),
                uppercase: false,
            },
        )
        .unwrap();
    make(&root, &stage, "install");
    world.make(&facts, Target::Install, Some("/stage")).unwrap();
    compare(
        &root,
        &stage.join("opt/tools/bin/greet"),
        &world,
        "/stage/opt/tools/bin/greet",
        "prefix-is-not-destdir",
        &mut records,
    );
    let hashes = execute(
        &root,
        Path::new("/usr/bin/sha256sum"),
        &["greet.c", "Makefile", "config.h", "config.mk", "greet"],
    );
    assert!(hashes.status.success());
    let manifest = serde_json::json!({"kind":"fixed-development-reference-only","cases":records,"make_version":String::from_utf8(make_version.stdout).unwrap(),"cc_version":String::from_utf8(cc_version.stdout).unwrap(),"selected_final_fixture_hashes":String::from_utf8(hashes.stdout).unwrap(),"limitations":"Fixed supplied C program/Makefile and controlled header generation only; not general C, Makefile or Autoconf parsing. No learner data or personal files. Fedora development evidence only."});
    fs::write(
        root.join("MANIFEST.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    println!(
        "SOURCE_INSTALL_REFERENCE: {} (30 behavior comparisons)",
        root.join("MANIFEST.json").display()
    );
}
