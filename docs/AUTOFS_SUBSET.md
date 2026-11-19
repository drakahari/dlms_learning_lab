# Local-file AutoFS simulation subset

Status2026-11-07: registered source-held direct/indirect activities in the337lab/
116unit review catalog. Focused/development-native checks pass; final isolated
candidate verification is pending. This is partial local-file203.3practice only.

## Intended behavior

A master row joins a managed directory to a supplied map file and explicit idle
timeout. An indirect key is one directory name under that directory; a direct
map uses `/-` in the master and full mountpoint paths as keys. Entries use
`-fstype=ext4` and a colon-prefixed local device, such as `:/dev/VG/LV`.
Only supplied virtual devices and nonnested trigger points are available.

Saving and activating are distinct. The runtime snapshots master/direct maps
only on explicit refresh. An indirect lookup consults the current saved map.
Neither changes an already mounted backing filesystem. The next lookup after
expiry mounts the then-selected original filesystem through the existing
in-process mount engine and reads its actual virtual files. File-not-found can
happen **after** a successful demand mount; that mount is retained. Unknown or
inactive devices fail without inventing data or a mount.

Idle checks advance a bounded virtual clock, never sleep or query the host.
Each mount tracks its own last access. A check expires the supplied nonbusy
backing mount when its idle interval reaches its timeout. This settled-check
model does not claim the kernel's exact expiry scheduling or busy/open-handle
behavior. It does not model the separate autofs trigger filesystem as a backing
ext4 mount. No real daemon, systemd unit or service is started.

## Closed subset and boundaries

- 1–3 provided plain map files;0–2 active master rows;1–4 supplied filesystems
  and distinct nonnested trigger mountpoints. Master/map files each ≤4096bytes,
  ≤64lines; at most8map rows (the4target/unique-key limits may reject sooner).
- Explicit master `--timeout=1` through `--timeout=3600`; no implicit daemon
  defaults. Empty master/map is valid and selects no entries. Whole-line
  comments, ordinary whitespace and `file:` prefixes are supported.
- Local ext4 entries, optional single `ro`/`rw` and `defaults`; unique options.
  Device existence and active state are checked by the mount engine, not parser
  pattern matching. Synthetic filesystem UUIDs stay tied to original virtual
  objects but UUID tags are not admitted in this map subset.
- Unsupported: NFS/network/NSS/program/LDAP/NIS/SSS maps, includes, wildcard or
  variable/key substitution, quoting/escaping, inline comments, multi-mounts,
  replicated locations, other filesystem/options, mount namespaces and host I/O.
  Malformed/unsupported maps fail explicitly; no shell or program map executes.
- Duplicate keys/reused files, nested/overlapping namespaces fail explicitly.
  Real Linux ignores later duplicate indirect master directories; this subset
  rejects that ambiguity rather than pretending to implement all precedence.
- The reused mount engine permits one live backing mount per supplied filesystem.
  Mounting that same filesystem at another provided point fails explicitly; this
  is a simulator restriction, not a Linux restriction.
- Changing root/timeout/direct-key structure while backing filesystems remain
  mounted is explicitly outside this initial model. Expire the supplied nonbusy
  mounts first. This is a simulator limit, **not a universal Linux reload rule**.
  Updating a direct entry's source/options with the same roots/keys can refresh
  while mounted, preserving the old association until expiry.
- ≤86400simulated seconds; each idle step1–3600seconds; four bounded activity
  records, ≤4096bytes per supplied file, ≤64KiB original storage. Saved model
  bounds and closed deserializers are enforced independently of text parsing.
  Invalid save/refresh/clock operations retain previous state. A successful
  demand mount preceding a missing-file error is the explicit exception above.

Existing profiles and grades are unchanged for all335previous labs. The new
source-held AutoFS Goal/session admits only replay-validated operations against
its caller's catalog. Each activity teaches master versus map, key/path
composition, saving versus refresh and lookup versus expiry before assessment,
with separate example drafts and a meaningful independent assignment. At most
64actions and32terminal rows per scope,512KiB per scope/1MiB per session; a
reset starts only that selected attempt. New requirements receive no old credit.
Older review binaries cannot interpret new AutoFS saved-work variants: retain
review backups when comparing old candidates. Original normal progress remains
separate from these review profiles. No cryptographic learner authentication or
proof of independent mastery is claimed.

## Sources and verification status

Checked2026-11-07 against upstream AutoFS manual pages:
[autofs(5)](https://man7.org/linux/man-pages/man5/autofs.5.html), especially indirect
versus direct refresh, Sun keys, local-device colon and non-NFS filesystem option;
[auto.master(5)](https://man7.org/linux/man-pages/man5/auto.master.5.html), master
format, direct marker, indirect keys and duplicate-root behavior. These are
primary author-maintained AutoFS manuals; no source text/palette is redistributed.
[LPI LPIC-2v4.5 objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/)
203.3 names AutoFS. This bounded local ext4 exercise cannot cover network maps or
all filesystem configuration objectives.

ParserR04:11, runtimeR08:16, workspaceR13:11, source/progressR20:5, authoredR23:6,
eguiR25:5 and native route/captureR27:2focused tests pass. Actual developmentR28
219native actions/30captures pass. Final presentation polish/isolated candidate
is still pending. No actual
AutoFS daemon/kernel conformance comparison, final packaged native flow, installed acceptance
or novice/screen-reader study is claimed. Development never runs real autofs,
mounts, host service commands or learner scripts as a reference.
