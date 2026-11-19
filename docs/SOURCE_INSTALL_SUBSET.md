## Verified409R1 package

Canonical-installed419 source actions, retainedJSON85, private406R1→409 lifecycle
and NFS two-process cold save/reopen/repair pass. Final selected-input identity
matches. Actual installed100/150dark/200Light and NFS screens inspected. No novice,
AT, physical-device or other-OS acceptance. CHECKPOINT.md records exact path/hashes/
receipt. Pending statements below describe development history; this package is now
technically verified, with owner lesson review still open.

# Supplied source installation: simulation contract

Three lessons use a bounded, in-process supplied-project model, not arbitrary C,
Autoconf or Makefile execution. No host file, subprocess or network operation is
reachable from learner input. No new dependency is required. All paths are virtual.

## Files and build behavior

The exercise supplies an uncompressed structured archive and a typed greeting
program. Unpacking creates virtual source files. Configure records prefix,
greeting and optional ASCII uppercase; generated config.status/Makefile describe
the fixed model, not executable learner scripts. Uppercase affects the whole
line, including the name; disabling it preserves name case.

The fixed dependency graph is `check → all` and `install → all`. Reconfiguration
stales the old build/check. Building creates a program object with compiled
settings; check tests that current build. Installed copies keep their own settings
until replaced. Clean removes build artifacts, not installed copies. Program
output is derived for different names, --version and --prefix, not canned success.

DESTDIR prepends a staging root without changing the compiled prefix. Install
copies the actual source README and sets program mode 0755/README mode 0644.
Omitting DESTDIR can overwrite a supplied virtual live file; there is no false
protection. That unrelated live package is an opaque preservation fixture;
executing its existing contents is not modeled. Atomic model transitions do not
promise atomic or safe real installations. All virtual paths are accessible;
full Unix privileges are outside this model.

## Literal command subset

- `pwd`, `cd PATH`, `mkdir [-p] PATH`, `ls [PATH]`, `cat FILE`.
- `tar -tf ARCHIVE`, `tar -xf ARCHIVE -C DIRECTORY`, and documented verbose/long
  equivalents. The destination must exist. The archive is uncompressed data.
- `./configure` with the supplied --prefix, --with-greeting,
  --enable-uppercase/--disable-uppercase options. Greeting options are specific
  to this package, not universal Autoconf options.
- `make [all|check|install|clean] [DESTDIR=PATH] [-C DIRECTORY]` with supported
  equivalent argument positions; bare make selects all.
- Explicit virtual program paths with one name, --version or --prefix.

Single/double quoting preserves literal words/spaces. Shell operators, expansion,
pipelines, unknown commands/options/targets, compression, arbitrary recipes and
source editing refuse honestly. Tar -ft/-fx are not guessed as -tf/-xf synonyms.
Development-only source-action authoring references are not Linux commands or
learner input; native Check actions grade current state directly.

Bounds: 1024 command bytes, 24 literal ASCII words, 128-character paths,
32 archive entries/32 KiB source text, 192 files, 384 directories, 32 KiB output,
96 successful commands per draft. Paths normalize only in the virtual namespace.
Invalid transitions preserve state. Finite operations cannot execute shell loops.
Compression, patch, uname, standalone install, package registration, arbitrary
compiler behavior and full timestamp/build graphs remain unsupported.

## Grading and progress

Configuration, current self-check, placement/modes, four behavioral cases,
documentation bytes and preserved live files grade separately. Passing version/
prefix cases cannot hide wrong settings, stale checks or damaged live files.
Supported alternative paths, option spellings/order and defaults can pass.
Mutation invalidates success; read-only queries preserve correct current state.

Version1 separate practice/assignment drafts persist source-bound command history
and unfinished input. Replay derives world/output; imported success/output is not
trusted. Restore rejects foreign tasks, forged results, erased intentional starter
history, unknown fields and oversized data. Global progress schema1 is unchanged.

## Evidence and references

Core12/workspace11/bindings4/authored8 and seven UI/capture regressions passed
before final packaging. Development native R3 completed419 actions; all three
Introduction/worked/practice/assignment screens and representative100/150dark/
200Light narrow views were inspected. See SOURCE_INSTALL_LESSONS.md/CHECKPOINT.md.

The explicitly ignored Linux development test compares fixed trusted C/Makefile
fixtures against GNUmake4.4.1/GCC16.2.1:30 comparisons passed after tightening.
Empty environment, private disk fixtures, make jobs1 and bounded children apply.
It never accepts learner drafts or becomes the app engine. Retained manifest:
`.build-artifacts/tmp/source-install-reference-108-1794413997859424657/MANIFEST.json`.

Sources checked2026-11-11: [LPIC-2 v4.5.0 objective206.1](https://www.lpi.org/our-certifications/exam-201-202-objectives/),
[GNU make DESTDIR](https://www.gnu.org/software/make/manual/html_node/DESTDIR.html),
[Automake staged installs](https://www.gnu.org/software/automake/manual/html_node/Staged-Installs.html),
[Autoconf installation names](https://www.gnu.org/software/autoconf/manual/autoconf-2.68/html_node/Installation-Names.html).
Primary GNU search excerpts supported prefix/staging semantics; direct requests
timed out. These narrow references/fixed comparisons do not establish general
Autoconf/compiler fidelity or full objective, novice or platform acceptance.
