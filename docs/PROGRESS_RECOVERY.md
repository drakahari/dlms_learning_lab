# Progress, backups and recovery

Your assignments, example drafts, favorites and navigation views are local. There is no account, cloud synchronization or telemetry. Settings → Your data manages your own files; simulator paths such as `/work/report` never access this feature or the host filesystem.

## Export

Choose **Settings → Your data → Export backup…**, select a folder and an unused filename, then Save backup. Documents is the initial location; you do not need to know an application profile path. The native chooser lists folders/files and accepts a filename in the selected folder. Existing files and reserved application filenames cannot be replaced. Copy the resulting JSON backup to your own safe storage. It contains your scripts and other work: do not publish it in the project repository.

The backup contains progress, both assignment/example drafts, case results and favorites. Appearance is independent and stays unchanged when you restore. Backups use a versioned envelope; old raw version-1 progress files are also accepted for recovery.

## Restore

1. Close any other copy of Learning Lab. Choose **Restore backup…** and select a backup.
2. The app checks the format, identifiers, versions, bounded state and drafts. A background worker re-runs saved script/example results inside the bounded simulator. Cancel stops checking without writing anything. Forged or stale result text cannot earn script completion merely by being present in JSON.
3. Review the saved-lab/completion/favorite summary. Nothing is replaced until **Restore checked backup** is selected. Cancel preserves the current session.
4. Before replacement, current writable session edits are saved. An exact copy of the existing progress file goes to `progress-before-restore-N.json` beside it, using an unused filename. If the app opened corrupt progress read-only and you made session edits, those edits are also exported to `session-before-restore-N.json` first. If protecting work fails, restore stops.
5. The already checked in-memory backup is atomically installed. Changing the selected file after the preview cannot change what is restored. The app returns to the library and displays the safety-copy path.

A write can be committed while final directory durability confirmation fails. The app reports that specific warning instead of falsely claiming that no replacement happened. Keep an additional backup when such a storage warning occurs. The controlled failure test covers reporting; real power-loss/filesystem durability has not been hardware tested.

## Old work and newly added requirements

Version-1 progress stays readable. Existing scenario IDs/versions are retained. Previously completed four-answer IPv4 analysis remains visible as earlier credit; a new workstation requirement is never filled in automatically. Values an older version supplied are retained for review, and an explicit check is needed before current completion. Reset affects only its selected attempt. Older binaries may not understand later optional fields; do not downgrade your only progress copy.

## Corrupt, future or oversized files

Invalid/future imports are rejected without replacing current work. Invalid normal progress opens read-only so the original remains recoverable. Symbolic links, devices and other special files are not accepted. The current progress size ceiling is 16 MB; the backup envelope allows only small additional metadata.

If the *current* progress file exceeds 16 MB, automatic restore refuses to replace it because a bounded exact safety copy cannot be made. Recovery steps:

1. Close all app instances.
2. Use your OS file manager to copy the oversized file to a safe location, then move the original out of the displayed local-progress location. Keep that copy; do not delete it.
3. Reopen the app and restore a known valid backup through Settings. The app will not truncate an oversized file on your behalf.

A corrupted file with no known backup may need manual review; do not import edited data into your only copy. The application reports unknown versions rather than silently dropping unfamiliar labs or fields.

## Multiple instances and platform status

The app takes an OS writer lock **before reading** progress. A second instance opens read-only, including preferences. Close/reopen it after the owning instance exits to load the newest data. The OS releases the lock on exit/crash; the small lock file is deliberately retained rather than deleted, avoiding inode replacement races. On storage that cannot safely lock, the app refuses writable operation. This protects cooperating app instances, not arbitrary external file editors or adversarial host filesystem races.

Fedora backend/UI/native evidence is recorded in IMPLEMENTATION_LEDGER.md. The standard-library lock and atomic-write architecture is portable, but Windows/macOS runtime behavior still needs actual platform acceptance.

## Shell evaluator upgrades

Saved script runs now identify the evaluator revision that produced them. Earlier records without that field remain readable and preserve their original source, output, file snapshots and earned credit. They are labeled **Earlier run complete · Rerun to verify**, separately from current success. Opening the assignment moves the older result to optional saved history without changing the draft. Run verifies the current draft; a failure or unsupported construct cannot reuse earlier success. A successful current Run restores current completion, while earlier history remains available. Reset explicitly clears that selected attempt, including history. Successive upgrades retain earlier runs together, including a later earned result alongside an older incomplete run. Optional Earlier saved runs shows their scripts, output and simulated files. History is bounded to32 records per attempt and the existing16MB profile limit; a full history needing another migration is rejected with the original retained, rather than silently discarding work.

Before the first upgraded write, the exact older progress bytes are copied once to `progress-v1.before-shell-vN.json` (N is the running evaluator revision) beside the main file; existing backups are never replaced. Reads alone do not rewrite progress. Save size/bounds and atomic-write protections still apply. Export/restore retains earlier history and rechecks active saved runs with the current evaluator before approval; example output has separate revision labeling and never awards assignment credit. Future incompatible record revisions are rejected with the original file retained. Downgrade compatibility after saving new metadata is not claimed.


Evaluator4 tightens shared text-expansion bounds and cumulative replacement work. A source that earned credit under3 can now reach a resource limit, even without printing large output. Its exact earlier source/results/credit remain historical; a current Run is required for current completion. Existing earlier histories stay alongside it, and the first upgraded write preserves the exact prior profile in the non-overwriting v4 backup. Reset clears only the selected attempt/history; it cannot remove that original-byte safety copy. See SHELL_EXPANSION_LIMITS.md.


Evaluator5 adds bounded * and ? filename matching and corrects unquoted variable-supplied bracket expressions that earlier ran literally. A clean revision4 probe confirmed that earlier result before migration tests were authored. Unsupported bracket patterns now ask for the supported subset or literal quoting; prior results/credit remain history, with the first upgraded write protected by the non-overwriting v5 backup. New syntax never earns prior learners unrecorded answers. See SHELL_GLOBBING.md.


The IPv4 routing integration adds only optional version1 workspace fields, omitted from earlier attempts. Assignment and ungraded practice each retain their own topology and incomplete raw configuration text. A pending different draft clears current completion even if the previously applied topology still passes; Check configuration applies all valid edits atomically. Invalid imported device/interface/cable structure or routing metadata attached to another activity is rejected before restoration. Existing original subnet, IPv6, terminal and script attempts keep their IDs/versions and data.


Evaluator6 adds the bounded Bash indexed-list subset. Earlier scalar/array results and credit remain labeled history; they do not invent new array answers. The first upgraded write preserves exact older bytes in the non-overwriting v6 backup, including Reset-first behavior. Evaluator7 adds bounded cooperative command substitution; its public execution and exact-backup migration are integrated and independently backend-reviewed, with synthetic native Fedora Run/Stop/history/export/review/cancel acceptance (SHELL_SUBSTITUTION.md). Actual Windows/macOS acceptance remains pending.

The process and LLDP units add optional saved workspace metadata without changing the version1 progress envelope or existing scenario definitions. Practice tables/topologies, assignment state, invalid editable drafts and selected teaching views stay separate. Restore validates authored process/endpoint identities and fixed attachments. Unsaved configuration, pending termination, stale or expired neighbor data cannot retain current completion.


## Revision 7 transition receipts and exact backups

On the first revision7 write over a compatible existing target, the fixed `before-shell-v7.json` slot must contain the **exact original target bytes**, including whitespace. An unused slot is created without replacement; an occupied slot, including an AlreadyExists race, is accepted only if those bounded ordinary-file bytes are identical. Different valid schema1 JSON also blocks the write. The actual on-disk target determines this boundary, even if Reset removed all runs or only an old practice result remains. A first workspace transition has an independent exact `before-workspace.json` slot.

Successful main-file writes include small optional internal `migration_receipts` (workspace boundary and bounded sorted evaluator revisions). The writer carries them only from the actual compatible on-disk target; candidate/import metadata never authorizes a skip. They survive Reset/removal/recreation of workspaces. Later writes retain old practice/history without comparing the evolving main bytes to the original fixed snapshot. Receipts mean a boundary was handled through backup, creation or explicit recovery; they do not assert that a fixed backup file currently exists. Completed slots are not recreated or reverified if an external editor later removes/replaces them. This is cooperative application provenance, not cryptographic integrity or protection from hostile filesystem edits.

Metadata is prepared only in pending main bytes. Failed pre-copy, safety-copy or main persistence leaves the original target and caller provenance unchanged; exact backup artifacts already created may remain and support retry. A directory-sync warning after replacement counts as committed and retains the new receipts. Readers/imports reject malformed or future receipts. Exports deliberately omit receipts, imports validate then strip them, and restored profiles obtain target-derived receipts through the same writer. Portable data cannot supply target migration permission.

Normal load/save still rejects corrupt or incompatible targets. The existing explicit validated-preview/confirmed-Restore recovery path can replace a bounded corrupt/future target only after its exact raw bytes are protected in a new no-clobber `progress-before-restore-N.json`. Those bytes are not classified as evaluator6 and no fixed before7 backup is claimed. Recovery does not inspect, replace or claim an unrelated fixed migration slot. The new receipt records that the creation/recovery boundary was handled. Cancelling preview performs no write; an active nested capture is cooperatively cancelled without returning a graded candidate. Invalid/oversized/special-file originals remain protected by the existing bounded reader rules.


## Bounded history and receipt sequences

The reader now bounds `script_history_more` and `migration_receipts.shell` while
reading each sequence, before retaining more than32 elements or decoding an
excess record. It never reserves memory from an unchecked sequence size hint.
Existing validation still enforces a combined maximum of32 earlier records
across `script_history` and `script_history_more`, along with each record's
revision/content bounds. A standalone history list may contain32; if the primary
history is present, only31 additional records are valid. The optional primary
record is separate, so at most32 list records plus that one record can be
decoded before the unchanged combined check rejects the profile. Receipt order,
unique nonzero revisions, current/future policy, portable omission, exact backup
provenance and the evaluator revision do not change. Missing/empty fields retain
their previous defaults and serialization. This is a reader allocation bound,
not a new migration or support for additional shell syntax.

The scoped reader verification passed80 checks:49 library checks (including a
typed-element counter proving an excess item is not decoded), six new sequence
reader checks, seven backup checks and18 existing actual-history/migration
regressions. The new checks retain all33 original revision6 records byte for byte
and exercise an empty current7 record shape without claiming an observed
solution. No earlier-history snapshot was regenerated. All-target Clippy with
warnings denied and formatting checks pass. These reader checks do not establish
new evaluator syntax, curriculum or native-platform acceptance.


## Evaluator 8 and multiple earlier revisions

Revision 8 adds bounded `case` selection. An existing revision 7 run and any
revision 6 history retain their exact source, outcomes and revision labels.
Current success requires a new Run; an earlier completion remains identified as
earlier work. No new lab requirements or script answers are filled in.

The first compatible write at the new boundary uses the exact original target
bytes in `progress-v1.before-shell-v8.json`; an occupied different file blocks
that write even if it differs only in whitespace. Reset-first and example-only
profiles still need protection. The already completed revision 7 receipt and
snapshot are separate. Portable backups cannot supply the target's migration
permission. Confirmed restore rechecks saved sources in the current simulator
and protects the exact pre-restore target; Cancel writes nothing.

The controlled revision 7 fixtures are genuine observations from that evaluator,
not revision 8 outcomes relabeled as old work. Backend checks cover history,
capacity, separate drafts/examples, exact backups, write failures, cancellation,
restore and selected reset. Native and platform evidence is recorded separately
in the implementation ledger; save/readback alone is not a cold-process restart
or power-loss test. Keep an exported backup before trying an older app version.

## Evaluator9 getopts source boundary (acceptance pending)

Startup OPTERR now defaults to1 even when a fixture supplied0; this changes
previously valid8 outcomes and requires9, independently of newly supported
getopts. No9 activation or test/native acceptance is claimed by the source patch.
Genuine6/7/8 records stay unchanged in bounded multirevision history. Opening or
saving never replays an old script or fabricates current completion. Examples
keep their own saved source/result revision; they never award assignment credit.

The first9 write must preserve the **actual target's** exact raw bytes in
`progress-v1.before-shell-v9.json`. An occupied different valid JSON or merely
different whitespace blocks replacement. Old before8 copies are not overwritten.
Only actual on-disk provenance governs repeated saves: a target carrying[7,8]
becomes[7,8,9], whereas a direct7→9 write becomes[7,9]. Imported markers cannot
skip protection. Reset-first and practice-only old results need the same backup.
Pending/failed writes do not stamp the caller; a postreplacement durability
warning means the replacement committed. Explicit confirmed raw recovery retains
its exact pre-restore safety copy; it does not invent a prior revision receipt.
Cancel writes nothing. No new progress schema or arbitrary revision override is
introduced; existing generic receipt/history code supplies this transition.
