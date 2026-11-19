# Bounded restricted-deletion model

Integration milestone2026-11-05. Two source-bound lessons are registered in the
current292/94catalog, after the existing access-policy unit. Core, workspace and
source/progress tests passed. Actual native dev flow R12passes82actions/11images,
and seven native-widget checks pass.
Final feedback wording and actual packaged verification pass in292/94T033211Z/bd2b8824.
104.5.K2 receives only narrow Partial practice; existing backup t/T metadata is a
different capability. See DIRECTORY_POLICY_LESSONS.md for final scoped evidence.

## Model and isolation

`filesystem::directory_policy::CoreWorld` wraps the unchanged ordinary `Tree` and
a small set of sticky directories. Existing Node/Tree fields, ordinary0777 admission,
source-free handlers and old activity semantics are unchanged. Only the opt-in model
projects01000 into effective directory modes and t/T listings. Files, identities,
paths and every operation remain in process; no learner input reaches a shell,
subprocess, host filesystem, user account, service or network.

Ordinary directory write/search and ancestor search are checked first. In a sticky
directory, removing a regular name also requires the actor to own that entry or its
directory. A regular-file move checks both source removal and replacement of an
existing destination; an unused destination has no existing entry owner to check.
Directory-owner status does not supply missing ordinary permissions. Failed actions
publish no state, clock, content or sticky changes.

`touch`/`cp` use existing real simulated creation and content-write operations.
Sticky does not prevent writing existing writable file contents. Fixed participant
probes execute supported operations on discarded bounded copies; they distinguish
Permission, Missing and Operation failures. Unsupported syntax, invalid source,
unknown actors and resource failures stay errors, never evidence of permission denial.
The lesson wrapper binds source/assignment/practice and replays bounded mutation
history from its held source before accepting progress. Structural CoreWorld
validation alone does not authorize imported progress.

## Disclosed command subset

- Existing ordinary `pwd`, `cd`, `id`, `cat`, `ls -a/-l/-d/--` observations.
- `chmod`: ordinary explicit-class rwx and000–777; directory-only `+t`, `-t`,
  numeric1000–1777 with optional leading zero. Ordinary numeric modes clear sticky.
  For symbolic `=`, selecting o/a clears unmentioned sticky; u/g without o leave
  it. Ordinary +/-rwx preserve it. Compound special-bit expressions, o+t/a+t,
  SUID/SGID, X, copying classes and recursive changes are honest exclusions.
- Literal `touch FILE`, `cp FILE DEST`, regular `rm [-f] FILE`, `mv FILE DEST`.
  Quoting and `--` preserve literal names; no expansion, pipelines or redirection.
- t means sticky plus other-user search; T means sticky without that search bit.
  Both restrict deletion. Dates remain the existing synthetic tick display, not
  host timestamps or complete GNU ls formatting.

No root/capability overrides, ACL/SELinux, immutable flags, links, real devices,
mount crossing, directory creation/removal/moves, interactive rm prompts, SUID/SGID,
full GNU grammar or same-path rename success is modeled. Existing Tree's disclosed
same-path refusal is retained. No privileged/cross-owner kernel acceptance follows.

Bounds:64nodes, four ordinary actors plus optional metadata root, two sticky
directories,128KiB file text,1MiB raw core decode,4096-byte commands,32KiB output
and1024-byte diagnostics. Source-bound sessions separately limit128mutations per scope,4096-byte drafts,
40transcript entries capped8192bytes each, and2MiB standalone raw session input.
Nested progress uses the existing16MiB import cap; the standalone raw cap is not
claimed as a preallocation limit for embedded serde sessions.
Original9–18evaluator baselines and all earlier reference evidence stay immutable.

## Actual focused checks

Staged compilation follows BUILD_WORKFLOW.md after a guarded low-headroom stop.
R04new20groups, R05old filesystem12 and R06old access7 pass. R07focused Clippy
passes21.60s. R08Python840/12.410s and later protocol Python852/12.232s pass.
No broad inclusive Rust suite or native/source-progress acceptance is claimed.

The separate fixed GNUchmod comparison actually observes15supported operators and
two exclusions on17fresh owned empty directories, plus one version call. Each
subject starts1770; held no-follow FDs verify identity, ownership, parent, emptiness
and mode/ctime-only changes. Original0755chmod uses closed C environment, held ELF,
child-only NNP proof, bounded CPU/address space/time/output and MAIN shared lock.
No accounts, privilege escalation, real multiuser deletion or personal paths are used.

Control58bd4b4e6ead4ec60239eed3e838649671d05ffd5641833bfb74b517ff6aff0e;
original chmod a77565b748fd1867a5a5e693b6bcc890025bc55892a714204c660a3a1d9bd460;
executed model probe168da2187808a5d895fb18168cb8eff1702abb8818d2d4a0fecf369dbe7b34fd.
Actual observation83605140904acb66eb0e05f928dab73784bd798d229c5b68b8cab8c38822bca3;
actual comparison faa26d72405645d926db019452ae5656185069be2a28ec8a5b1bd740780cd8b9
records15supported matches and2honest exclusions. Final independent model/source
readback seal is abdb7817ef5a19eb002672f747da162e54587159504b7e4b3912247cdb529f5c.
The actual669input canonical commitment is
c881e9161c688a6ba2c3157a3ac0f56300fae026e6d590b726333842b4725046.
Preserve raw attempt
`.build-artifacts/reference-runs/sticky-mode-r1`; do not rerun its control after
source advances. No numeric mode expectations are supplied in the fixture; the
probe derives actual modes from CoreWorld. Excluded GNU-valid o+t/4755 remain
unsupported in the model and are not matches or SUID coverage.

## Primary sources and remaining fidelity

Checked2026-11-05: installed GNUCoreutils9.10 Info/chmod manual (chmod.1.gz
5531bfb123f8554a0247a56862fa753eb34d39866c728a246515bfa3793174a0),
and primary Linux man-pages6.19 online
[unlink](https://man7.org/linux/man-pages/man2/unlink.2.html) and
[rename](https://man7.org/linux/man-pages/man2/rename.2.html), dated2026-02-08.
Earlier retained installed Linux6.13 pages are distinguished in the private review.
GNU live special-mode pages timed out; installed observed manual bytes ground the
original paraphrases, not a claimed live-page version. No GNU/manual source code
is copied into this Rust implementation.

A single real UID cannot empirically establish cross-owner denial or directory-owner
exceptions. That remains unverified without a separately authorized multiuser test
environment. Pure model matrices/manual agreement and same-owner chmod comparison
are distinct evidence. None implies full104.5 coverage, novice comprehension,
physical-device/screen-reader, Windows/macOS/Omarchy or installation acceptance.
