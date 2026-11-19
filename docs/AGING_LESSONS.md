# Password-aging administration unit

Development346 / 122 units,2026-11-08. Latest ready review remains PAM343T224545Z
until this unit's separate native/package gate is complete. Three distinct LPIC-1
assignments follow local account administration in `linux-password-aging`:

- `lpic1-aging-rotation`: inspect records, set maximum age and advance warnings,
  retain change history, and compare before/deadline/grace-end dates.
- `lpic1-aging-grace`: repair inactivity grace without removing rotation or
  confusing Password change required with unrestricted use.
- `lpic1-aging-contract-reset`: select any finite authorized account-expiry date,
  preserve an intentional reset marker and unrelated policies, then verify both
  reset and account-expiry outcomes. This is the unit's combined handover.

These are different administration tasks, not numerical variants counted as new
labs. Each teaches fields and options before testing, annotates a separate worked
example, supplies guided command practice and retains an independent assignment.
The virtual terminal uses the new bounded aging model; no old account engine or
scenario is changed. The supplied date table labels Expected, Actual and Not
checked. Commands are parsed into typed edits; no learner text goes to a shell.
Optional worked insertion populates practice input only; Run command executes it.
Challenge starts with independent work and hides teaching/solutions until requested.

Each task grades required fields, preserved history, protected sibling records and
three to eight dated observations. Alternate short/long options, combined or
separate commands and equivalent supported dates pass. The contract task accepts
multiple dates in its stated interval; sample dates are verified against both
interval ends. Disabling all limits, faking history, clearing the reset or changing
protected records fails. Command errors do not count as policy outcomes. A broken
state cannot retain current success even if a saved checked flag remains true.
Unexecuted command drafts are separate from current record state; they are not
invented learner actions. Any command run clears the previous check until retry.

Practice and assignment have different supplied identities and independent worlds,
inputs and bounded transcripts. Only the selected attempt resets. Current source
and identity guards validate restored work; the existing progress format remains.
No existing completion is fabricated for this new unit. Tests use disk-backed
synthetic profiles for save/export/restore and retain the pre-restore safety copy.

## Sources and boundaries

See [the subset contract](AGING_SUBSET.md), specified before implementation.
Primary references checked2026-11-08: shadow4.18.0 `chage` manual, Linux-PAMv1.7.1
`check_shadow_expiry` and current LPI LPIC-1v5.0 objectives107.1/110.1. The command
lists metadata; the separately labeled date table implements the disclosed pinned
policy ordering, including equality at expiry. Manual boundary wording differs;
this is not a universal claim about all Linux login configurations.

No actual password/hash, password change, authentication, PAM call/module loading,
NSS/LDAP, account file, host command, clock or system change. Query output uses ISO
dates only. Interactive mode, short clusters, shell syntax, root/prefix, other
utilities and ordinary epoch day0 are explicitly unsupported. Lastday0 is the
force-change marker. This adds narrow Partial practice; actual password setting,
other security tools and full objective coverage remain unimplemented.

## Evidence so far

R03primitive10passed. R09all18primitive/workspace checks passed. The new integration
first failedR10 on a result-alias mismatch; R11fixed it without changing assertions.
R13preflight2and prior18passed before registration; R14retained both source checks
after new-only wording/spacing corrections. R17all27passed: primitive10,workspace8,
authored preflight2,catalog/persistence3 and retained PAM4. All343old scenario values
and121old unit values/order remain exact. Old114LPIC1presentation values match the
actual343candidate prefix (private canonical identity receipt).

R18exposed an old original-nine test helper's missing new-state exclusion; R19
retained that helper's assertions and found actual Enter submission and table
horizontal overflow failures. R20fixed Enter and retained the overflow failure
with measured760width geometry. R21all3UItests passed7.80seconds after stacking
related content vertically inside the existing aligned table cells. No per-screen
pixel offsets or text shrinking. Actual keyboard focus, query insertion without
execution, Enter/Run, partial/wrong/unsupported/retry, all3screens and five text
scales in narrow/wide light/dark are covered. Expanded populated/unchecked layout
checks and the new native route/package gate are pending.

No full-inclusive Rust, actual host chage/PAM comparison, installed/cold lifecycle,
all28-theme sweep, Windows/macOS/Omarchy runtime, physical-device/screen-reader or
independent novice acceptance claim. Source-based review and automated checks do
not replace novice comprehension testing. No dependencies, installed-app/original
progress change, publication, push or merge.

Final source gates: R39all27behavior/persistence/preservation checks and R40all10
UI/route/preview checks pass. R29development-native135actions/45PNGpasses; root
opened3after-polish originals. R25failed later preview registration, not the135
completed hands-on actions. R26private-method test compile error retained and
fixed by locating the regression in the owning module. Optional case feedback
uses the actual table instead of repeating every expected/actual paragraph below
it; field/history/protected-record mistakes remain visible. Intro numbering is
provided once by the shared renderer. Wide layouts use two columns and guidance
sits beside the command input; narrow layouts stack without shrinking text.
R34all-targetClippy and R43Python946/R45focused presentation pass. Inventory346,
strict286Partial/7Supporting/947Open passes. Exact final package/native remains
pending; no expanded acceptance claim.

Final scoped Fedora admission: R46–51 exactT234326Zpackage validates346/122,
135native actions/45PNG pass and11originals were opened. Initial/final selected
identity matches777/191/323/4/70. Private r51receipt records hashes and limitations.
This supersedes preceding pending-package notes. No broader acceptance claim.
