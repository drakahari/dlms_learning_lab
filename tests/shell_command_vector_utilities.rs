//! In-process source probes only; Root runs these under the shared memory-safe runner.
//! Expected values are manually reasoned, not obtained from the candidate evaluator.
use dlms_learning_lab::shell::{self, Case};
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    name: String,
    group: String,
    sources: Vec<String>,
    case: Case,
}

fn source(id: &str) -> &'static str {
    match id {
        "sort-vector" => {
            r###"LC_ALL=C
if test "$#" -ne 3; then printf 'Invalid sort request\n' >&2; exit 2; fi
case "$1" in asc|desc) :;; *) printf 'Invalid sort request\n' >&2; exit 2;; esac
case "$2" in all|unique) :;; *) printf 'Invalid sort request\n' >&2; exit 2;; esac
case "$3" in ''|-|.|..|*/*) printf 'Invalid source\n' >&2; exit 2;; esac
if test -f "$3"; then :; else printf 'Missing source\n' >&2; exit 1; fi
command=(sort)
if test "$1" = desc; then command=("${command[@]}" -r); fi
if test "$2" = unique; then command=("${command[@]}" -u); fi
command=("${command[@]}" -- "$3")
"${command[@]}"
"###
        }
        "sort-direct" => {
            r###"LC_ALL=C
if test "$#" -ne 3; then printf 'Invalid sort request\n' >&2; exit 2; fi
case "$1" in asc|desc) :;; *) printf 'Invalid sort request\n' >&2; exit 2;; esac
case "$2" in all|unique) :;; *) printf 'Invalid sort request\n' >&2; exit 2;; esac
case "$3" in ''|-|.|..|*/*) printf 'Invalid source\n' >&2; exit 2;; esac
if test -f "$3"; then :; else printf 'Missing source\n' >&2; exit 1; fi
case "$1:$2" in
 asc:all) sort -- "$3";;
 asc:unique) sort -u -- "$3";;
 desc:all) sort -r -- "$3";;
 desc:unique) sort -r -u -- "$3";;
esac
"###
        }
        "select-vector" => {
            r###"LC_ALL=C
if test "$#" -ne 2; then printf 'Invalid transform\n' >&2; exit 2; fi
case "$1" in
 ordered) command=(sort);;
 neighbors) command=(uniq);;
 owners) command=(cut -s -d ':' -f 3);;
 *) printf 'Unknown action\n' >&2; exit 2;;
esac
case "$2" in ''|-|.|..|*/*) printf 'Invalid source\n' >&2; exit 2;; esac
if test -f "$2"; then :; else printf 'Missing source\n' >&2; exit 1; fi
command=("${command[@]}" -- "$2")
"${command[@]}"
"###
        }
        "select-direct" => {
            r###"LC_ALL=C
if test "$#" -ne 2; then printf 'Invalid transform\n' >&2; exit 2; fi
case "$1" in ordered|neighbors|owners) :;; *) printf 'Unknown action\n' >&2; exit 2;; esac
case "$2" in ''|-|.|..|*/*) printf 'Invalid source\n' >&2; exit 2;; esac
if test -f "$2"; then :; else printf 'Missing source\n' >&2; exit 1; fi
case "$1" in
 ordered) sort -- "$2";;
 neighbors) uniq -- "$2";;
 owners) cut -s -d ':' -f 3 -- "$2";;
esac
"###
        }
        "jobs-vector" => {
            r###"LC_ALL=C
count=$#
if test "$count" -eq 0 || test "$((count % 2))" -ne 0; then
 printf 'Invalid pairs\n' >&2; exit 2
fi
number=0
failed=0
while test "$#" -gt 0; do
 number=$((number + 1))
 action=$1
 file=$2
 shift 2
 ready=yes
 command=()
 case "$action" in
  ordered) command=(sort);;
  neighbors) command=(uniq);;
  owners) command=(cut -s -d ':' -f 3);;
  *) printf 'Rejected: %s\n' "$number" >&2; ready=no; failed=1;;
 esac
 if test "$ready" = yes; then
  case "$file" in ''|-|.|..|*/*) printf 'Invalid source: %s\n' "$number" >&2; ready=no; failed=1;; esac
 fi
 if test "$ready" = yes; then
  if test -f "$file"; then :; else
   printf 'Missing: %s\n' "$number" >&2; ready=no; failed=1
  fi
 fi
 if test "$ready" = yes; then
  command=("${command[@]}" -- "$file")
  printf 'Job %s: %s\n' "$number" "$action"
  "${command[@]}"
 fi
 done
exit "$failed"
"###
        }
        "jobs-direct" => {
            r###"LC_ALL=C
count=$#
if test "$count" -eq 0 || test "$((count % 2))" -ne 0; then
 printf 'Invalid pairs\n' >&2; exit 2
fi
number=0
failed=0
while test "$#" -gt 0; do
 number=$((number + 1))
 action=$1
 file=$2
 shift 2
 ready=yes
 case "$action" in ordered|neighbors|owners) :;; *) printf 'Rejected: %s\n' "$number" >&2; ready=no; failed=1;; esac
 if test "$ready" = yes; then
  case "$file" in ''|-|.|..|*/*) printf 'Invalid source: %s\n' "$number" >&2; ready=no; failed=1;; esac
 fi
 if test "$ready" = yes; then
  if test -f "$file"; then :; else
   printf 'Missing: %s\n' "$number" >&2; ready=no; failed=1
  fi
 fi
 if test "$ready" = yes; then
  printf 'Job %s: %s\n' "$number" "$action"
  case "$action" in
   ordered) sort -- "$file";;
   neighbors) uniq -- "$file";;
   owners) cut -s -d ':' -f 3 -- "$file";;
  esac
 fi
 done
exit "$failed"
"###
        }
        "publish-vector" => {
            r###"LC_ALL=C
if test "$#" -ne 3; then printf 'Invalid publication\n' >&2; exit 2; fi
case "$1" in ordered|neighbors|owners) :;; *) printf 'Unknown action\n' >&2; exit 2;; esac
case "$2" in ''|-|.|..|*/*) printf 'Invalid source\n' >&2; exit 2;; esac
case "$3" in ''|-|.|..|*/*|keep.txt) printf 'Invalid destination\n' >&2; exit 2;; esac
if test "$2" = "$3"; then printf 'Invalid destination\n' >&2; exit 2; fi
if test -f "$2"; then :; else printf 'Missing source\n' >&2; exit 1; fi
case "$1" in
 ordered) command=(sort);;
 neighbors) command=(uniq);;
 owners) command=(cut -s -d ':' -f 3);;
esac
command=("${command[@]}" -- "$2")
"${command[@]}" > "$3"
"###
        }
        "publish-direct" => {
            r###"LC_ALL=C
if test "$#" -ne 3; then printf 'Invalid publication\n' >&2; exit 2; fi
case "$1" in ordered|neighbors|owners) :;; *) printf 'Unknown action\n' >&2; exit 2;; esac
case "$2" in ''|-|.|..|*/*) printf 'Invalid source\n' >&2; exit 2;; esac
case "$3" in ''|-|.|..|*/*|keep.txt) printf 'Invalid destination\n' >&2; exit 2;; esac
if test "$2" = "$3"; then printf 'Invalid destination\n' >&2; exit 2; fi
if test -f "$2"; then :; else printf 'Missing source\n' >&2; exit 1; fi
case "$1" in
 ordered) sort -- "$2" > "$3";;
 neighbors) uniq -- "$2" > "$3";;
 owners) cut -s -d ':' -f 3 -- "$2" > "$3";;
esac
"###
        }
        "minus-stdin-control" => {
            r###"LC_ALL=C
command=(sort -- "$1")
"${command[@]}"
"###
        }
        _ => panic!("Unlisted fixed source {id}"),
    }
}

fn verify(group: &str) {
    let fixtures: Vec<Fixture> = serde_json::from_str(FIXTURES).unwrap();
    let mut checked = 0;
    for f in fixtures.iter().filter(|f| f.group == group) {
        for id in &f.sources {
            let actual = shell::run(source(id), &f.case);
            assert_eq!(actual.error, None, "{} / {id}: {actual:?}", f.name);
            assert_eq!(actual.status, f.case.status, "{} / {id}", f.name);
            assert_eq!(actual.stdout, f.case.stdout, "{} / {id}", f.name);
            assert_eq!(actual.stderr, f.case.stderr, "{} / {id}", f.name);
            assert_eq!(actual.files, f.case.after, "{} / {id}", f.name);
            checked += 1;
        }
    }
    assert!(checked > 0, "Missing group {group}");
}

#[test]
fn optional_sort_flags_and_option_looking_paths_are_separate_words() {
    verify("sort_options");
}
#[test]
fn dynamic_sort_uniq_and_cut_heads_honor_sentinel_and_literal_operands() {
    verify("utility_heads");
}
#[test]
fn per_job_vectors_reset_and_rejections_do_not_block_later_jobs() {
    verify("job_reset");
}
#[test]
fn authored_single_minus_guards_have_an_independent_stdin_control() {
    verify("minus_guard");
}
#[test]
fn missing_unknown_or_aliased_requests_cannot_truncate_the_destination() {
    verify("no_write");
}
#[test]
fn array_and_direct_branch_alternatives_publish_the_actual_utility_result() {
    verify("publication");
}

const FIXTURES: &str = r###"[
  {
    "name": "sort asc all two words.txt",
    "group": "sort_options",
    "sources": [
      "sort-vector",
      "sort-direct"
    ],
    "case": {
      "name": "sort asc all two words.txt",
      "args": [
        "asc",
        "all",
        "two words.txt"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/two words.txt": "z\nb\nA\nb\n"
      },
      "stdout": "A\nb\nb\nz\n",
      "stderr": "",
      "status": 0,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/two words.txt": "z\nb\nA\nb\n"
      }
    }
  },
  {
    "name": "sort asc all -r",
    "group": "sort_options",
    "sources": [
      "sort-vector",
      "sort-direct"
    ],
    "case": {
      "name": "sort asc all -r",
      "args": [
        "asc",
        "all",
        "-r"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/-r": "z\nb\nA\nb\n"
      },
      "stdout": "A\nb\nb\nz\n",
      "stderr": "",
      "status": 0,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/-r": "z\nb\nA\nb\n"
      }
    }
  },
  {
    "name": "sort asc all --",
    "group": "sort_options",
    "sources": [
      "sort-vector",
      "sort-direct"
    ],
    "case": {
      "name": "sort asc all --",
      "args": [
        "asc",
        "all",
        "--"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/--": "z\nb\nA\nb\n"
      },
      "stdout": "A\nb\nb\nz\n",
      "stderr": "",
      "status": 0,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/--": "z\nb\nA\nb\n"
      }
    }
  },
  {
    "name": "sort asc unique two words.txt",
    "group": "sort_options",
    "sources": [
      "sort-vector",
      "sort-direct"
    ],
    "case": {
      "name": "sort asc unique two words.txt",
      "args": [
        "asc",
        "unique",
        "two words.txt"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/two words.txt": "z\nb\nA\nb\n"
      },
      "stdout": "A\nb\nz\n",
      "stderr": "",
      "status": 0,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/two words.txt": "z\nb\nA\nb\n"
      }
    }
  },
  {
    "name": "sort asc unique -r",
    "group": "sort_options",
    "sources": [
      "sort-vector",
      "sort-direct"
    ],
    "case": {
      "name": "sort asc unique -r",
      "args": [
        "asc",
        "unique",
        "-r"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/-r": "z\nb\nA\nb\n"
      },
      "stdout": "A\nb\nz\n",
      "stderr": "",
      "status": 0,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/-r": "z\nb\nA\nb\n"
      }
    }
  },
  {
    "name": "sort asc unique --",
    "group": "sort_options",
    "sources": [
      "sort-vector",
      "sort-direct"
    ],
    "case": {
      "name": "sort asc unique --",
      "args": [
        "asc",
        "unique",
        "--"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/--": "z\nb\nA\nb\n"
      },
      "stdout": "A\nb\nz\n",
      "stderr": "",
      "status": 0,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/--": "z\nb\nA\nb\n"
      }
    }
  },
  {
    "name": "sort desc all two words.txt",
    "group": "sort_options",
    "sources": [
      "sort-vector",
      "sort-direct"
    ],
    "case": {
      "name": "sort desc all two words.txt",
      "args": [
        "desc",
        "all",
        "two words.txt"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/two words.txt": "z\nb\nA\nb\n"
      },
      "stdout": "z\nb\nb\nA\n",
      "stderr": "",
      "status": 0,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/two words.txt": "z\nb\nA\nb\n"
      }
    }
  },
  {
    "name": "sort desc all -r",
    "group": "sort_options",
    "sources": [
      "sort-vector",
      "sort-direct"
    ],
    "case": {
      "name": "sort desc all -r",
      "args": [
        "desc",
        "all",
        "-r"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/-r": "z\nb\nA\nb\n"
      },
      "stdout": "z\nb\nb\nA\n",
      "stderr": "",
      "status": 0,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/-r": "z\nb\nA\nb\n"
      }
    }
  },
  {
    "name": "sort desc all --",
    "group": "sort_options",
    "sources": [
      "sort-vector",
      "sort-direct"
    ],
    "case": {
      "name": "sort desc all --",
      "args": [
        "desc",
        "all",
        "--"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/--": "z\nb\nA\nb\n"
      },
      "stdout": "z\nb\nb\nA\n",
      "stderr": "",
      "status": 0,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/--": "z\nb\nA\nb\n"
      }
    }
  },
  {
    "name": "sort desc unique two words.txt",
    "group": "sort_options",
    "sources": [
      "sort-vector",
      "sort-direct"
    ],
    "case": {
      "name": "sort desc unique two words.txt",
      "args": [
        "desc",
        "unique",
        "two words.txt"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/two words.txt": "z\nb\nA\nb\n"
      },
      "stdout": "z\nb\nA\n",
      "stderr": "",
      "status": 0,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/two words.txt": "z\nb\nA\nb\n"
      }
    }
  },
  {
    "name": "sort desc unique -r",
    "group": "sort_options",
    "sources": [
      "sort-vector",
      "sort-direct"
    ],
    "case": {
      "name": "sort desc unique -r",
      "args": [
        "desc",
        "unique",
        "-r"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/-r": "z\nb\nA\nb\n"
      },
      "stdout": "z\nb\nA\n",
      "stderr": "",
      "status": 0,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/-r": "z\nb\nA\nb\n"
      }
    }
  },
  {
    "name": "sort desc unique --",
    "group": "sort_options",
    "sources": [
      "sort-vector",
      "sort-direct"
    ],
    "case": {
      "name": "sort desc unique --",
      "args": [
        "desc",
        "unique",
        "--"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/--": "z\nb\nA\nb\n"
      },
      "stdout": "z\nb\nA\n",
      "stderr": "",
      "status": 0,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/--": "z\nb\nA\nb\n"
      }
    }
  },
  {
    "name": "ordered with operand -r",
    "group": "utility_heads",
    "sources": [
      "select-vector",
      "select-direct"
    ],
    "case": {
      "name": "ordered with operand -r",
      "args": [
        "ordered",
        "-r"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/-r": "z\na\na\n"
      },
      "stdout": "a\na\nz\n",
      "stderr": "",
      "status": 0,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/-r": "z\na\na\n"
      }
    }
  },
  {
    "name": "neighbors with operand -u",
    "group": "utility_heads",
    "sources": [
      "select-vector",
      "select-direct"
    ],
    "case": {
      "name": "neighbors with operand -u",
      "args": [
        "neighbors",
        "-u"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/-u": "a\na\nb\na\n"
      },
      "stdout": "a\nb\na\n",
      "stderr": "",
      "status": 0,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/-u": "a\na\nb\na\n"
      }
    }
  },
  {
    "name": "owners with operand -f",
    "group": "utility_heads",
    "sources": [
      "select-vector",
      "select-direct"
    ],
    "case": {
      "name": "owners with operand -f",
      "args": [
        "owners",
        "-f"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/-f": "notes\na:west:Alex\nb:east:\nc:north:two words\nd:x:100%\\*\n"
      },
      "stdout": "Alex\n\ntwo words\n100%\\*\n",
      "stderr": "",
      "status": 0,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/-f": "notes\na:west:Alex\nb:east:\nc:north:two words\nd:x:100%\\*\n"
      }
    }
  },
  {
    "name": "neighbors with operand +2",
    "group": "utility_heads",
    "sources": [
      "select-vector",
      "select-direct"
    ],
    "case": {
      "name": "neighbors with operand +2",
      "args": [
        "neighbors",
        "+2"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/+2": "x\nx\ny\n"
      },
      "stdout": "x\ny\n",
      "stderr": "",
      "status": 0,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/+2": "x\nx\ny\n"
      }
    }
  },
  {
    "name": "owners with operand two words.txt",
    "group": "utility_heads",
    "sources": [
      "select-vector",
      "select-direct"
    ],
    "case": {
      "name": "owners with operand two words.txt",
      "args": [
        "owners",
        "two words.txt"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/two words.txt": "note\nA:R:first\nB:R:last"
      },
      "stdout": "first\nlast\n",
      "stderr": "",
      "status": 0,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/two words.txt": "note\nA:R:first\nB:R:last"
      }
    }
  },
  {
    "name": "empty selected source",
    "group": "utility_heads",
    "sources": [
      "select-vector",
      "select-direct"
    ],
    "case": {
      "name": "empty selected source",
      "args": [
        "neighbors",
        "empty.txt"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/empty.txt": ""
      },
      "stdout": "",
      "stderr": "",
      "status": 0,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/empty.txt": ""
      }
    }
  },
  {
    "name": "fresh vector for every different utility",
    "group": "job_reset",
    "sources": [
      "jobs-vector",
      "jobs-direct"
    ],
    "case": {
      "name": "fresh vector for every different utility",
      "args": [
        "owners",
        "-f",
        "ordered",
        "-r",
        "neighbors",
        "-u",
        "ordered",
        "-r"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/-r": "z\na\na\n",
        "/work/-u": "b\nb\na\nb\n",
        "/work/-f": "note\nx:y:owner\nz:y:\n"
      },
      "stdout": "Job 1: owners\nowner\n\nJob 2: ordered\na\na\nz\nJob 3: neighbors\nb\na\nb\nJob 4: ordered\na\na\nz\n",
      "stderr": "",
      "status": 0,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/-r": "z\na\na\n",
        "/work/-u": "b\nb\na\nb\n",
        "/work/-f": "note\nx:y:owner\nz:y:\n"
      }
    }
  },
  {
    "name": "rejected middle job does not reuse prior command",
    "group": "job_reset",
    "sources": [
      "jobs-vector",
      "jobs-direct"
    ],
    "case": {
      "name": "rejected middle job does not reuse prior command",
      "args": [
        "ordered",
        "-r",
        "$(printf injected)",
        "-r",
        "neighbors",
        "-u"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/-r": "z\na\na\n",
        "/work/-u": "b\nb\na\nb\n",
        "/work/-f": "note\nx:y:owner\nz:y:\n"
      },
      "stdout": "Job 1: ordered\na\na\nz\nJob 3: neighbors\nb\na\nb\n",
      "stderr": "Rejected: 2\n",
      "status": 1,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/-r": "z\na\na\n",
        "/work/-u": "b\nb\na\nb\n",
        "/work/-f": "note\nx:y:owner\nz:y:\n"
      }
    }
  },
  {
    "name": "missing middle job then valid owner job",
    "group": "job_reset",
    "sources": [
      "jobs-vector",
      "jobs-direct"
    ],
    "case": {
      "name": "missing middle job then valid owner job",
      "args": [
        "ordered",
        "-r",
        "neighbors",
        "absent.txt",
        "owners",
        "-f"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/-r": "z\na\na\n",
        "/work/-u": "b\nb\na\nb\n",
        "/work/-f": "note\nx:y:owner\nz:y:\n"
      },
      "stdout": "Job 1: ordered\na\na\nz\nJob 3: owners\nowner\n\n",
      "stderr": "Missing: 2\n",
      "status": 1,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/-r": "z\na\na\n",
        "/work/-u": "b\nb\na\nb\n",
        "/work/-f": "note\nx:y:owner\nz:y:\n"
      }
    }
  },
  {
    "name": "odd pair count does not begin first job",
    "group": "job_reset",
    "sources": [
      "jobs-vector",
      "jobs-direct"
    ],
    "case": {
      "name": "odd pair count does not begin first job",
      "args": [
        "ordered",
        "-r",
        "neighbors"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/-r": "z\na\na\n",
        "/work/-u": "b\nb\na\nb\n",
        "/work/-f": "note\nx:y:owner\nz:y:\n"
      },
      "stdout": "",
      "stderr": "Invalid pairs\n",
      "status": 2,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/-r": "z\na\na\n",
        "/work/-u": "b\nb\na\nb\n",
        "/work/-f": "note\nx:y:owner\nz:y:\n"
      }
    }
  },
  {
    "name": "authored sort rejects single minus before reading stdin",
    "group": "minus_guard",
    "sources": [
      "sort-vector",
      "sort-direct"
    ],
    "case": {
      "name": "authored sort rejects single minus before reading stdin",
      "args": [
        "asc",
        "all",
        "-"
      ],
      "stdin": "z\na\n",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/-": "not stdin\n"
      },
      "stdout": "",
      "stderr": "Invalid source\n",
      "status": 2,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/-": "not stdin\n"
      }
    }
  },
  {
    "name": "authored selection rejects single minus before reading stdin",
    "group": "minus_guard",
    "sources": [
      "select-vector",
      "select-direct"
    ],
    "case": {
      "name": "authored selection rejects single minus before reading stdin",
      "args": [
        "neighbors",
        "-"
      ],
      "stdin": "z\na\n",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/-": "not stdin\n"
      },
      "stdout": "",
      "stderr": "Invalid source\n",
      "status": 2,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/-": "not stdin\n"
      }
    }
  },
  {
    "name": "raw utility control treats single minus as stdin even after sentinel",
    "group": "minus_guard",
    "sources": [
      "minus-stdin-control"
    ],
    "case": {
      "name": "raw utility control treats single minus as stdin even after sentinel",
      "args": [
        "-"
      ],
      "stdin": "z\na\n",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/-": "not stdin\n"
      },
      "stdout": "a\nz\n",
      "stderr": "",
      "status": 0,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/-": "not stdin\n"
      }
    }
  },
  {
    "name": "missing source leaves existing output unchanged",
    "group": "no_write",
    "sources": [
      "publish-vector",
      "publish-direct"
    ],
    "case": {
      "name": "missing source leaves existing output unchanged",
      "args": [
        "ordered",
        "absent.txt",
        "result.txt"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/source.txt": "b\na\na\n",
        "/work/result.txt": "old result\n"
      },
      "stdout": "",
      "stderr": "Missing source\n",
      "status": 1,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/source.txt": "b\na\na\n",
        "/work/result.txt": "old result\n"
      }
    }
  },
  {
    "name": "unlisted action leaves output unchanged",
    "group": "no_write",
    "sources": [
      "publish-vector",
      "publish-direct"
    ],
    "case": {
      "name": "unlisted action leaves output unchanged",
      "args": [
        "$(printf injected)",
        "source.txt",
        "result.txt"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/source.txt": "b\na\na\n",
        "/work/result.txt": "old result\n"
      },
      "stdout": "",
      "stderr": "Unknown action\n",
      "status": 2,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/source.txt": "b\na\na\n",
        "/work/result.txt": "old result\n"
      }
    }
  },
  {
    "name": "same input output is rejected before truncation",
    "group": "no_write",
    "sources": [
      "publish-vector",
      "publish-direct"
    ],
    "case": {
      "name": "same input output is rejected before truncation",
      "args": [
        "ordered",
        "source.txt",
        "source.txt"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/source.txt": "b\na\na\n",
        "/work/result.txt": "old result\n"
      },
      "stdout": "",
      "stderr": "Invalid destination\n",
      "status": 2,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/source.txt": "b\na\na\n",
        "/work/result.txt": "old result\n"
      }
    }
  },
  {
    "name": "single minus source rejected before destination open",
    "group": "no_write",
    "sources": [
      "publish-vector",
      "publish-direct"
    ],
    "case": {
      "name": "single minus source rejected before destination open",
      "args": [
        "ordered",
        "-",
        "result.txt"
      ],
      "stdin": "z\na\n",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/source.txt": "b\na\na\n",
        "/work/result.txt": "old result\n",
        "/work/-": "not stdin\n"
      },
      "stdout": "",
      "stderr": "Invalid source\n",
      "status": 2,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/source.txt": "b\na\na\n",
        "/work/result.txt": "old result\n",
        "/work/-": "not stdin\n"
      }
    }
  },
  {
    "name": "actual ordered output changes only destination",
    "group": "publication",
    "sources": [
      "publish-vector",
      "publish-direct"
    ],
    "case": {
      "name": "actual ordered output changes only destination",
      "args": [
        "ordered",
        "-r",
        "result.txt"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/source.txt": "b\na\na\n",
        "/work/result.txt": "old result\n",
        "/work/-r": "z\na\na\n"
      },
      "stdout": "",
      "stderr": "",
      "status": 0,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/source.txt": "b\na\na\n",
        "/work/result.txt": "a\na\nz\n",
        "/work/-r": "z\na\na\n"
      }
    }
  },
  {
    "name": "actual neighbors output changes only destination",
    "group": "publication",
    "sources": [
      "publish-vector",
      "publish-direct"
    ],
    "case": {
      "name": "actual neighbors output changes only destination",
      "args": [
        "neighbors",
        "-u",
        "result.txt"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/source.txt": "b\na\na\n",
        "/work/result.txt": "old result\n",
        "/work/-u": "a\na\nb\na\n"
      },
      "stdout": "",
      "stderr": "",
      "status": 0,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/source.txt": "b\na\na\n",
        "/work/result.txt": "a\nb\na\n",
        "/work/-u": "a\na\nb\na\n"
      }
    }
  },
  {
    "name": "actual owners output changes only destination",
    "group": "publication",
    "sources": [
      "publish-vector",
      "publish-direct"
    ],
    "case": {
      "name": "actual owners output changes only destination",
      "args": [
        "owners",
        "-f",
        "result.txt"
      ],
      "stdin": "",
      "env": {},
      "files": {
        "/work/keep.txt": "preserve me\n",
        "/work/source.txt": "b\na\na\n",
        "/work/result.txt": "old result\n",
        "/work/-f": "note\na:r:Alex\nb:r:Sam\n"
      },
      "stdout": "",
      "stderr": "",
      "status": 0,
      "after": {
        "/work/keep.txt": "preserve me\n",
        "/work/source.txt": "b\na\na\n",
        "/work/result.txt": "Alex\nSam\n",
        "/work/-f": "note\na:r:Alex\nb:r:Sam\n"
      }
    }
  }
]"###;
