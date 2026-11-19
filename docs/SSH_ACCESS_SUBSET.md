# SSH trust and key administration

Existing-roadmap batch, 2026-11-13. Three authored tasks are integrated in source;
the separate445 R1candidate is now verified with the limits below. No claim of complete 110.3/212.3
coverage, cryptographic verification, or real OpenSSH runtime conformance.

## Learning progression

LPIC-1 → **SSH: verify the server identity** follows Linux host networking.
Its task repairs one stale known-host record using the supplied verified server
public key while retaining another host. A server identity is distinct from an
account's authorized user key. LPIC-2 → **SSH: keys and safe policy handover**
follows this context and existing access-policy practice. Its first task repairs
an authorized-key file, ownership and traversal safety. Its capstone combines
server trust, retiring an old user key, saved versus active daemon configuration,
and preserving rescue administration while excluding root and an unlisted user.
Prerequisites advise rather than lock experienced learners out.

Every task has an explicit introduction, different worked/practice values and a
full independent assignment. Native editors change virtual documents; permission
controls explicitly act as simulated administrator actions. They are not shell
commands or a claim that ordinary users may change another user's ownership.
Fill worked edits only populates separate practice drafts. Save, Apply saved
policy and Check access are distinct actions. Saving invalid daemon text does
not replace active policy; applying unsupported or malformed policy fails and
retains the previous active configuration. Editing any assignment draft clears
current success; checking unsaved changes cannot earn completion.

## Deliberately bounded behavior

- Fixed supplied host, accounts, public client identities and virtual home paths.
  Possession of the source-listed identity is an exercise assumption, never proof
  of a private key, signature, encryption, live connection or successful login.
- Plain Ed25519 public records: canonical Base64, algorithm and wire lengths are
  validated. Curve validity and cryptographic operations are not implemented.
  No private material, key generation or purported SHA256 fingerprints appear.
- Literal lower-case hostnames and canonical IPv4 known-host names; matching
  trusted server keys, unknown hosts and changed keys are separate outcomes.
  No hashed/wildcard/bracketed-port hosts, markers, IPv6 names or accept-new policy.
- Plain authorized-key membership plus ordinary-user virtual file read/traversal
  and StrictModes safety for home, .ssh and the supplied authorized-key paths.
  Safe alternate ownership and modes pass; exactly 0600/0700 is not required.
  Root key-file evaluation is explicitly outside this filesystem model. Root
  denial must be achieved by supported policy, not an invented unreadable key.
- Supported sshd_config directives: PubkeyAuthentication, PasswordAuthentication,
  KbdInteractiveAuthentication, StrictModes, PermitRootLogin (yes/no/
  prohibit-password/without-password), literal AllowUsers, AuthenticationMethods
  (any/publickey), and standard relative AuthorizedKeysFile paths or none.
  Ordinary first values win; repeated AllowUsers appends. Arguments are
  case-sensitive, keywords are not. Unsupported Match, Include, PAM, patterns,
  arbitrary paths and key options diagnose rather than silently approximate.
- Policy method availability is not password verification. A failure at the wrong
  stage cannot satisfy a policy/key-specific negative objective. Unsupported
  behavior produces Not evaluated and never passing credit.
- No SSH executable, socket, host filesystem or network call exists in these
  modules. Native clipboard copy exports only supplied public fixture data.
  The `ssh-lab` JSON prefix is an authoring/test dispatcher, not a Linux command
  taught to learners. Only bounded typed events reach the source-bound controller.

Key/config text is limited to 16 KiB, key lines to 1 KiB, public records to 64,
known-host aliases to eight, policy directives to 128, literal allowed users to
32, source client identities to 16, editable documents to finite supplied paths,
and authoring events to 32 KiB. General pack/progress limits also apply. These
finite lookups do not run scripts or loops. Protected users, nodes, metadata and
file contents are compared against the source inventory on restore and save.

## Authoring and preservation

`ssh_identity`, `ssh_policy` and `ssh_key_access` contain the bounded models;
`ssh_access` composes trust/policy/path decisions. `ssh_workspace` binds source
requirements to independent and practice sessions and atomic virtual saves.
`src/ui/ssh_page.rs` renders accepted native teaching/editor components.

The three source lessons live in `scenarios/ssh-access-lessons-v1.json` and are
bundled with the exact 69 preceding continuation scenarios in
`boot-entry-lessons-v37.json`. This follows existing consolidation so the
128-file bundled limit, external 100-file limit and byte/scenario budgets stay
unchanged. Old v36 and all historical registry helpers remain intact. Regression
checks compare every old 442 scenario and 174 unit value against the new catalog.
Progress schema remains unchanged; new sessions carry their own version and lab
identity. No old completion, answers or mastery are fabricated.

## Primary references, checked 2026-11-13

- [OpenBSD sshd_config manual](https://man.openbsd.org/sshd_config): rolling
  official manual, directive precedence, method policy, paths and StrictModes.
- [OpenBSD sshd manual](https://man.openbsd.org/sshd.8): public-key file formats
  and access requirements. Exact safe modes depend on ownership and traversal.
- [OpenBSD ssh_config manual](https://man.openbsd.org/ssh_config): strict host
  checking. This exercise assumes strict, noninteractive checking; it does not
  implement every client option or a particular Fedora OpenSSH version.
- [RFC 8709 §4](https://www.rfc-editor.org/rfc/rfc8709.html#section-4): Ed25519
  public-key wire format, not an encryption algorithm.
- [RFC 8032 §7.1](https://www.rfc-editor.org/rfc/rfc8032.html#section-7.1): four
  published PUBLIC KEY test-vector byte strings used as fixture identities.
  Private fields, implementation code and algorithms are not copied or executed.

Current checks and native/package evidence are recorded in CHECKPOINT.md. Public
manual research is not a real sshd comparison. Windows/macOS runtime, physical
keyboard/device, screen-reader and independent novice acceptance remain pending.

## Development evidence

Focused models/workspace/content42 plus retained client diagnostics7 pass; every
old442 scenario and174 unit value is exact. Four actual-widget tests plus a fixed
native-plan test pass. One-job all-target Clippy (warnings denied) and inclusive
Python1235 pass. Strict coverage accepts445 labs,176 units and no unregistered
drafts. Teaching inventory440updated/5already-meets follows individual content and
rendered review, not shared-renderer inheritance. No full inclusive Rust runtime
rerun in this batch. Final gates will be recorded in CHECKPOINT.md.

Actual native R40:196 actions/26 captures, separate examples, assignment failure,
repair, stale-success invalidation, failed atomic Save, retry, per-lab reset and
real private progress save/readback. R34's193actions/23captures additionally retain
the pre-correction feedback. All3Introduction/practice/Assignment originals were
inspected; R40complete worked tables and corrected practice/neutral failure copy
were viewed separately. Source-only scene/layout tests use five scales, light/dark
and760/1600widths. Native originals are1920x1080/1902x929:150%light and200%dark
permission fields/actions are readable and aligned, but a narrower window request
is not accepted as actual native narrow-size verification. Some anchors appear
only after vertical scrolling; that is not a claim everything fits one viewport.

Representative local, ignored originals:
- [Trust Introduction](../.build-artifacts/native-runs/run-q_tdqsyu/screenshots/workspaces/ssh-0-introduction.png)
- [Permissions guided practice](../.build-artifacts/native-runs/run-q_tdqsyu/screenshots/workspaces/ssh-1-practice.png)
- [Policy worked example](../.build-artifacts/native-runs/run-q_tdqsyu/screenshots/workspaces/ssh-2-worked.png)
- [Independent handover](../.build-artifacts/native-runs/run-q_tdqsyu/screenshots/workspaces/ssh-2-assignment.png)
- [Specific failed cases](../.build-artifacts/native-runs/run-q_tdqsyu/screenshots/workspaces/ssh-2-needs-attention.png)

These are technical native reviews, not independent novice, physical keyboard,
assistive-device or screen-reader acceptance. No actual OpenSSH executable was
used for comparison; only primary documented behavior and the bounded model
were checked. At this development checkpoint, installed checks were pending; the subsequent packaged checks are recorded below.

Final source readback before packaging: real registered light/dark assertions and
actual content bounds pass in R45; normal current-catalog UI4 passes in R47. The
earlier layout fixture used fallback theme IDs, so its two-theme claim was
corrected and rerun rather than accepted. Two old capture openers had stale
current-size constants; only those assertions advanced to445/176. Final inclusive
Python1235 R50 and all-target Clippy R49 pass. R48 was interrupted by daemon
restart and never counted as a pass. Recovery retained its exact marker/log only
after acquiring the shared OS lock and verifying no recorded child/compiler.
Ordinary source/native gates: minimum available12.307 GiB, full PSI avg10max0.18%,
OOMdelta0; low swap headroom remains. No system settings or other processes changed.

## Packaged445 R1 review

See CHECKPOINT.md for the exact protected launch, binary/archive/BUILD_INFO hashes
and source gates. Private upgrade from442, uninstall/reinstall and exact synthetic
progress retention pass. Installed SSH196-action/26-capture route passes. Actual
packaged worked-policy table, corrected practice copy, named failure cases and
200% permission controls were individually viewed. Actual NFS writer/reader both
fully exit0, restored unfinished and repaired originals were inspected, and all43
manifest files rehash. Final frozen readback/retention passes for2,855selected
files/108,500,393bytes. Preserve the immutable snapshot; its scope is selected
inputs, not the full compiler closure or an independent seal. Bundled documents
precede this record. No original app/profile or public release changed.

Installed originals (local, ignored):
- [Worked policy](../.build-artifacts/native-runs/run-jt5jzz2c/screenshots/workspaces/ssh-2-worked.png)
- [Guided permissions](../.build-artifacts/native-runs/run-jt5jzz2c/screenshots/workspaces/ssh-1-practice.png)
- [Named failed cases](../.build-artifacts/native-runs/run-jt5jzz2c/screenshots/workspaces/ssh-2-needs-attention.png)
- [Dark200 controls](../.build-artifacts/native-runs/run-jt5jzz2c/screenshots/workspaces/ssh-1-dark200-permissions.png)

No SSH-specific two-process GUI reopen, native narrow sizing, physical keyboard,
assistive-device, screen-reader, independent novice, real OpenSSH or Windows/macOS
acceptance is implied. NFS cold reopening and SSH actual disk readback are separate
checks. Full current exam objectives/curriculum remain incomplete.
