# Explicit-category whereis foundation

The bounded core now supports one source-bound application-support handover in
the290/93source catalog; see WHEREIS_LESSON.md. The verified Fedora T225518Z
candidate still predates this new lesson until separately rebuilt and checked.

The original in-process reader accepts only these literal forms with a supplied
category, directory and known dot-free ASCII basename:

```text
whereis -b -B DIR -f NAME
whereis -m -M DIR -f NAME
whereis -s -S DIR -f NAME
```

It reads immediate virtual directory names, not file contents, program code or
host search paths. Initial admitted forms are NAME for binary, NAME.1 for manual,
and NAME.c for source. A matching directory or a file with no executable/read
permission can still be named: discovery is different from which eligibility.
Reading names needs ancestor search and directory read; the scanned directory’s
own execute permission is unnecessary for this shallow name-only lookup.

Potentially significant unsupported manual/source suffixes, compression, s.
prefixes and multiple matches refuse before returning a partial answer. Binary
extra suffixes do not match. No default paths, environment search, interleaved
names/categories, globbing, source normalization, links or full utility claim.
Normal supported empty output is NAME plus colon and LF, with successful command
status; finding no path is separate from command failure. Model diagnostics are
explicit simulator explanations, not invented utility stderr.

Caller-held Context and original Tree authorize a query. Imported contexts must
match; caller roster/home and the existing operational marker cannot change.
This core alone does not authorize deployment edits or stored progress. The
separate whereis_deployment policy supplies those checks using held author data;
the earlier modes-only policy has not been broadened. No new Tree wire field or
evaluator change is included.

Bounds:16KiB raw context,4known names,3categories/one directory per category,
2KiB command,64nodes/files/modification records,256KiB aggregate file contents,
32KiB output and at most one match. The reader borrows data and clones no bodies.
Context fields have bounded deserializers; an external Tree caller still needs
its established input limits. None of this calls a host tool or filesystem.

## Sources and actual gates

Official util-linux2.41.5 source archive was read only, not installed or executed:
https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.41/util-linux-2.41.5.tar.xz
SHA256f586e35d320ff537aab3ffeca37e9ecd482ccbe013590db4429a414d8aa6a728
matches the published HTTPS checksum list; its PGP signature was not verified.
Bounded streaming inspection covered3884members/82,852,714declared bytes and read
only the exact whereis.c/manual/path header. whereis.c SHA2444328d9b22693aff30e0e0eaa0a6d0e8802daa333e8ed7da222be18bbba3cc
retains its file-specific Regents BSD notice privately. No upstream code is copied
into the original Rust implementation. The actual installed binary was subsequently observed as2.41.5 in the fixed
comparison below; its manual was not used as a substitute for that observation.

Source inspection confirms that the real utility initializes built-in system
directory metadata before parsing flags. Explicit category/directory queries
replace effective lookup paths but do not constitute a filesystem jail. The fixed
developer reference below uses a closed environment, original held ELF, private
generated fixture paths, restricted child limits, exact source/control approval
and retained actual stdout/status/order. Do not claim zero host startup I/O or
infer runtime comparison from this source reading.

R45all19new core groups and16retained command-location groups pass after normal
MAIN-runner compilation. R44was a root manifest-shape setup error; no code was
admitted or compiled by it. Those initial pure-core gates did not promote an objective. The later actual
comparison is recorded below; source-bound teaching/deployment and saved-work
gates are recorded separately in WHEREIS_LESSON.md and CHECKPOINT.md.

## Actual fixed comparison, before integration

R56 observed the original installed util-linux2.41.5 ELF (28200bytes, SHA256
49900147303e504d5b469b0602da31c3c088456e1edb8f9ed658863697185c73).
Its exact version output through held descriptor6 was `6 from util-linux 2.41.5`
plus LF; it was not relabeled as a direct-path invocation. R57compared18supported
queries exactly, including no-match status0, zero-mode files, matching directories,
ancestor-search/read restrictions, case sensitivity, shallow scans and live
add/remove changes. Six significant suffix/compression/prefix/multiple-match forms
were observed separately and honestly refused by the model; they are NOT six extra
conformance matches. Actual output ordering is retained, never sorted to pass.

Original control6befa1d6 binds663actual compiled source/data/extras and10tools.
Actual probe9a93cdc8 emitted commitment33e5ffc7; comparison445415f9 and raw
PROBE4be7f4d5 retain all statuses/stdout/stderr. Independent final readback
6226f38c7778f45495cd92b111fbcc63f0f00ca513e1edcc32a576739dccda7c
recomputed18exact/6honest results from retained bytes, not expected echoes.
All25restricted child captures have distinct child PIDs and the exact NNP proof;
48before/after snapshots match without restoring restrictive directory/file modes.
The controlled reference is a developer fixture, never a learner execution path.

R50/R51first guard runs exposed repeated directory-FD listing offsets. Their
failures remain; explicit lseek-to-zero and triple-snapshot regression yield
R52all819Python checks passing. No utility ran before that correction. The fixed
probe/source/control/actual captures remain immutable under ignored
.build-artifacts/reference-runs/whereis-r1. Later integration changes the live
source closure: do not rerun this old prepared control or overwrite its evidence.

Normal reference startup reads trusted built-in SYSTEM directory metadata before
flags, as disclosed above. The query lookup folders are explicit generated fixtures;
this is not a filesystem jail or zero-host-I/O claim. Learner execution remains
entirely in-process with no host tool, filesystem or network access.
