# Native application identity and Fedora startup investigation (M34)

Specified 2026-10-22 before the scoped identity change. Preserve the native toolkit, window size, fonts/themes, simulation and all custom progress/preference paths. No dependency, compositor setting, desktop-theme change or activation/security workaround.

The prior isolated default-Wayland trace stalled after its first surface frame request and a keyboard-focus departure. That is evidence of an intermittent rendering/startup problem, not an established cause. XWayland walkthroughs have passed. A fresh 15-second default-Wayland baseline at service-teaching source 77e341a now advances through multiple real native screenshots and frame callbacks. The timeout intentionally bounds a startup probe; it is not a full-flow acceptance failure or proof that the old issue disappeared.

Both the historical and current traces omit a Wayland `set_app_id`. The pinned egui-winit 0.33.3 only supplies it when ViewportBuilder.app_id is explicit. Set `org.dlms.LearningLab`, matching the existing packaged `org.dlms.LearningLab.desktop`. This follows the pinned upstream xdg-shell set_app_id protocol recommendation for grouping/desktop integration; it is an application identity fix, not a presumed frame-callback fix. Original source: wayland-protocols 0.32.13/stable/xdg-shell/xdg-shell.xml; pinned API: egui 0.33.3 ViewportBuilder::with_app_id and egui-winit 0.33.3 window-attribute builder.

The application's progress/preferences use their existing explicit directories APIs. eframe’s optional persistence feature is disabled. Setting native window identity must not relocate or invent learner work. Verify emitted protocol identity, real Fedora flows in default Wayland and XWayland, save/resume, visible resizing/scale and actual candidate launch outside the checkout. Keep intermittent startup, real Windows/macOS desktop and assistive-technology acceptance honest; do not infer causation from a single passing run.


## Observed results

The corrected application's own protocol trace emits `xdg_toplevel.set_app_id("org.dlms.LearningLab")`. A complete default-Wayland service teaching flow and a subsequent separate default-Wayland subnet/VLSM flow both pass, including current grading, separate drafts, unsaved/stale invalidation, reset/resume and light/dark 150% resizing. Captures were reviewed at 1920×1080 and 946×929. The pre-change 15-second startup probe also rendered normally, so these observations do **not** establish that application identity caused or cured the old intermittent stall. They establish current default-Wayland operation for those flows and correct desktop identity.

Build, formatting and Clippy pass; independent read-only review confirmed the exact packaged ID, Wayland-only native attribute application and absence of eframe persistence. The existing 310-test backend/UI baseline is unchanged; it was not rerun merely to test a window identifier. Actual native flows are the relevant added evidence. Previously verified XWayland remains an application-local fallback. No compositor, activation token, security setting or user theme was altered.

The original default-Wayland anomaly remains monitored until its conditions can be reproduced and distinguished from focus/occlusion behavior. Windows/macOS desktop, screen-reader/physical-input and complete curriculum acceptance remain open. Do not describe this identity correction as a conclusive startup fix.

M36 subsequently passed both full alignment capture matrices and the broader normal-app default-Wayland flow. In M37 a default-Wayland addressing run again produced no first capture before interruption. This confirms the earlier passing runs did not establish a permanent cure. M37 used the application-local `env -u WAYLAND_DISPLAY` XWayland fallback for native teaching verification. No compositor/theme/activation/security setting was changed. Separately, the resize/scroll test harness needed settled frames/hover before delivering a click batch; corrected harness timing is not evidence of a desktop-policy or production pointer fix.

## M42 investigation: repeated launches and observational diagnostics (2026-10-23)

No speculative startup fix was applied. The historical trace's unanswered callback belongs to Mesa's EGL surface queue after its first buffer commit; it is not evidence that winit's separate Requested-frame redraw gate caused the problem. A keyboard-focus departure does not prove occlusion or compositor fault. The pinned wayland-client0.31.15 wayland.xml permits withholding callbacks for completely obscured/off-screen surfaces. Successful current runs update while unfocused too. A main-thread stack at the failure is needed to distinguish EGL swap waiting, event dispatch, application rendering or storage. The online protocol reference fetch timed out; the pinned original protocol XML and eframe0.33.3/glutin0.32.3/winit0.30.13 sources were inspected directly.

The development binary passed three initial and twelve further direct-Wayland DNS-layout flows, then forty first-render/readback probes. Opt-in DLMS_STARTUP_TRACE=1 reports fixed lifecycle markers/times and focus/minimized state for the first four frames; no learner input, profile paths, scripts or content are logged. Normal rendering/backend/vsync/window configuration and real progress/preferences paths are unchanged. --ui-smoke with DLMS_SMOKE_STAGE=startup verifies the library, screenshot readback and subsequent event-loop progress before closing. Smoke profiles now use fresh random temporary directories retained by a guard, explicit0700 Unix permissions and automatic cleanup, separate from real learner data and other tests.

Use the guarded helper: python3 tools/dev.py tool native_startup.py --app target/debug/dlms-learning-lab --count 20 --output .build-artifacts/UNIQUE-NEW-DIAGNOSTIC-DIRECTORY. It uses fresh private output, exclusive file creation, capped2MB logs, bounded child lifetimes and optional own-child backtraces with GDB init/auto-load/debuginfod disabled. It respects debugger permission failures and never changes ptrace/compositor/focus/security settings. --no-protocol --no-lifecycle measures without logging perturbation. Always choose a fresh project-disk output under the mandatory memory-safe workflow; do not invoke the helper directly with RAM-backed temporary defaults. Generated evidence stays local. A failure stops the series with partial results and the original evidence retained. A diagnosis is not inferred from timeout alone.

Review exposed predictable/overwritable harness paths and incidental GDB initialization; these were corrected rather than described as a learner-execution escape. Tests cover private distinct profile lifetime/permissions, capped output, timeout/interruption cleanup and occupied/symlink log protection. The normal learner profile branch stays untouched. A fresh actual Wayland alignment matrix also passes100/150/200% at2560×1440 and1268×632, wide grid and stacked VLAN controls, empty/populated/focused IPv4 fields, errors, scrolling and light/dark. Representative renders were reviewed independently; shared native field-geometry and all70-assignment200% regressions pass. No novice, reader or actual Windows/macOS acceptance is implied.

Current scoped results are recorded in IMPLEMENTATION_LEDGER.md. The intermittent failure is **unreproduced and its cause unresolved**, not fixed by these diagnostics or proved impossible by passing launches. Continue independent simulator/content work while retaining the evidence needed if it recurs. Do not force another backend or alter desktop/security settings as a presumed cure.

Final M42 scoped evidence: fifteen unmodified development DNS-layout launches, forty instrumented startup probes, twenty early driver probes, ten hardened driver probes and twenty quiet driver launches all passed (105 development launches across the stated different scopes). The **actual packaged46791cc Fedora binary** separately passed twelve direct-Wayland DNS-layout launches outside the checkout: one confirmed the native app ID in the protocol, eleven disabled protocol/lifecycle logging; each complete flow took6.47–6.92seconds. No failing process was observed from which to obtain a causal stack. The final tool treats reaching its deadline as failure even if the child later succeeds while sampled. Sixty-eight current UI tests (including private temporary-profile permissions/lifetime) and five diagnostic boundary tests pass; formatter/Clippy pass. Layout captures contain no remaining material alignment issue. These results improve current reliability evidence and future observability, not causal closure of the historical stall.

## M44: distinct display-connection failure (2026-10-23)

After one successful new-array introduction capture, the next default native launch exited immediately with winit WaylandError(Connection(NoCompositor)), before app initialization/rendering. The selected wayland-1 socket still exists, but a bounded Unix-socket connect returns ECONNREFUSED. The application-local XWayland fallback exits XOpenDisplayFailed; X0 also refuses connections. Other X socket entries deny access and were not bypassed. Neither Xvfb nor Weston is installed; no new display server, compositor/theme/security change or permission bypass was attempted. A subsequent selected-socket recheck still refuses. Logs:/tmp/dlms-array-native-second.log and /tmp/dlms-array-native-xwayland.log.

This failure has an observed immediate cause: no accepting server on the session-selected display socket. It is **different from the historical unanswered-frame-callback stall** and does not establish its cause or a cure. Prior105 scoped development and12 packaged Wayland passes remain dated/scoped evidence, not a guarantee. New array all-four rendering/large-text/typed retry/resume/reset and installed-current runtime acceptance remain pending while displays are unavailable. Pure egui geometry/keyboard tests pass but cannot substitute for those native flows. Continue independent simulator/content/packaging work.

M45 follow-up: the selected Wayland socket later accepted connections without any application/system-setting change. The exact installed346ffa0 candidate rendered all4 array examples/assignments with reference repairs and saved reload, then exposed a smoke-harness target error for long scripts: full TextEdit content rectangle versus clipped visible viewport. A new regression proves visible-editor typing clears current success; corrected dev native full flow and separate array Stop/repair/resume pass. Independent render review accepts desktop1920×1080 and narrow946×929 light150/dark200 scrolling. This harness correction is not a startup fix. Retest the next clean installed candidate and repeated quiet startups separately; historical unanswered-frame cause remains unresolved.

M46 installed follow-through: clean017e875 candidate passes all3 native array modes outside checkout and10/10 additional quiet default-Wayland startup/readback launches. No lifecycle/protocol logging or backend override in quiet runs. Recorded0.965–0.966s includes test pacing, not a performance benchmark. Full array flow, Stop/repair/resume and unit-library scaling are actual installed evidence. Candidate identity/checksums in INSTALLATION.md; historical unanswered-frame cause remains unresolved. No compositor/theme/security setting was changed.

M47/M48 exact installede2b0200 follow-through: all4 authored process flows and the full72-capture shared IPv4/VLAN alignment matrix pass outside checkout, then ten further quiet default-Wayland startup/readback launches pass10/10. The driver records0.965–0.966s including pacing, not a startup speed benchmark. Representative actual fields/values/focus/columns/scrolled light/dark100/150/200% captures were reviewed. This improves current installed reliability evidence, not causal closure of the historical callback stall. Exact identity/hashes/scope in INSTALLATION.md. No backend/compositor/theme/security change.


## Latest scoped installed launches

M56f4d7bc0/evaluator7 and M57 7ffbfff/90labs each pass10/10 quiet default-Wayland launch/readbacks on Fedora44, using their actual isolated installed executables. Latest evidence `.build-artifacts/installations/m57-97r6dbbw/quiet-startup/results.json`; final installed all-four command-capture lesson flow also passes. Harness pacing is not a launch-speed benchmark. These observations neither reproduce nor explain/cure the original intermittent unanswered callback; that cause remains unresolved. CI/headless tests and these synthetic events do not establish other-OS desktop, novice, physical-device or screen-reader acceptance.


M68 clean29096be Fedora candidate separately passes all four installed OSPF workflows,72IPv4/VLAN alignment captures and10/10quiet default-Wayland launch/readbacks. Actual launcher, hashes and scopes are in INSTALLATION.md; evidence m68-installed-{ospf,alignment,quiet}-r1.log, installed prefix verified-29096be81982-2eflmz82. Root inspected representative actual light/dark100/150/200 renders. No failing unanswered-callback process was observed, so that historical cause is still unresolved; no backend/compositor/theme/security change or cure is claimed. Synthetic controls/readbacks are not Windows/macOS desktop, novice, physical-device or screen-reader acceptance.


M77 installed109/26 source5a968c7 passes all-three case-dispatch flows,72 shared
IPv4/VLAN alignment captures and10/10 quiet default-Wayland launch/readbacks.
Actual installed prefix verified-5a968c735523-mzx5b592, exact hashes/scopes in
INSTALLATION.md. No historical unanswered-frame failure was observed, so its
cause remains unresolved. A separate first M77 teaching capture deadlocked in
new screenshot-harness geometry by reading zoom inside a context-data lock;
that recursive read was moved outside the lock and actual native checks pass.
It is not evidence about the historical Wayland startup stall. No compositor,
backend, theme, security or other desktop setting changed.


## M93 live stalled-process capture (2026-10-25)

Actual installed97b90923eb7c native seed flow stalls before controller diagnostics.
A bounded same-user GDB attachment (init/autoload/debuginfod disabled) captures
main thread inside Mesa26.2.5 eglSwapBuffers/dri2_wl_swap_buffers_with_damage,
loader_wayland_dispatch, wl_display_dispatch_queue and ppoll. The verified own
synthetic child is then closed via pidfd/SIGTERM, reaped normally by its guard;
no other app/agent or system setting is touched. Evidence stays ignored/private.

An independent existing15s startup/protocol probe reproduces the same wait:
creation/application-ready and update0/1 finish, callback53 requested on surface38
and buffer committed, but no callback done before deadline. Keyboard enters then
leaves. Main stack again shows EGL/Wayland dispatch. This establishes an observed
blocking location for the historical pattern; it does not establish why the
callback was absent or blame focus, occlusion, compositor, driver or application.
It is distinct from earlier capture-only Context reentry deadlocks.

The same ELF passes one process-local XWayland startup/readback after only
omitting WAYLAND_DISPLAY for that child. No desktop/backend global setting is
changed. Full teaching verification continues on that supported native path;
default Wayland reliability/acceptance remains failed/pending. Earlier deb55's
10/10 passes remain true for their scope, not proof that later stalls cannot occur.


## M98 bounded same-binary diagnostic (not a normal launch change)

Linux `--ui-smoke` can request no vsync with the exact process-local environment
value `DLMS_SMOKE_NO_VSYNC=1`. It still uses its isolated synthetic profile.
Normal launches always retain pinned eframe0.33.3's vsync=true, even with that
variable present. Other values/platforms retain the default. A pure options
regression verifies this boundary without changing process/global environment.
No driver, compositor, desktop theme, service or model setting is changed.
This is an investigation hook, not an accepted learner workaround or cure.
Compare default versus requested no-vsync startup probes on the same identified
candidate and preserve both results/stacks. A swap-interval request does not
prove the driver honored it or explain an absent compositor callback. Native
acceptance and underlying Wayland cause remain separate.


## M99 Linux Wayland application policy

Same installedc799 ELF: default interval1 startup times out0/1 with tracing;
requested interval0 succeeds3/3. Quiet interval0 succeeds10/10, then quiet
interval1 fails0/1 after warm-up, with the same owned EGL/Wayland wait. This
supports an interval-dependent stall on this host, not its exact driver cause.
The full original-three terminal teaching flow passes on native Wayland with
that diagnostic request,43captures and clean exit. No host setting changes.

Independent pinned-source review confirms eframe0.33.3 requests Wait(1) versus
DontWait, glutin0.32.3 maps them to EGLSwapInterval1/0, and swaps still occur.
The event loop remains Wait/WaitUntil unless a redraw is requested, so disabling
the interval does not itself create an idle polling loop. Active redraw bursts
can run faster and interval-setting remains best effort, not driver confirmation.

The application now requests interval0 when Linux's pinnedwinit0.30.13 selection
rule chooses Wayland: nonempty Unicode WAYLAND_DISPLAY or WAYLAND_SOCKET. It
has no forced-backend hook; XDG_SESSION_TYPE alone is not used. X11, Windows and
macOS keep their existing defaults. Empty/nonUnicode signals and cross-platform
options have pure regression coverage. This is a pre-loop backend-selection
policy, not inspection of an already-created native handle. No toolkit/dependency,
desktop theme, compositor/GPU/service/swap or global environment change.

Three options tests, all-targetClippy and fmt pass. Actual production-policy
candidate startup, normal-private-profile reopening and Run/Stop are next;
no normal-policy runtime acceptance is inferred from diagnostic runs alone.


Production-policy11811b7 installed follow-through:119/29 upgrade/lifecycle passes,
then defaultWayland (diagnostic override absent) passes1/1 protocol/lifecycle and
10/10quiet startup/readbacks. A strict two-process private NORMAL-profile reopen
passes9captures through existing XDG-isolated developer acceptance; it does not
use the smoke profile. Exact manifest/session receipt remains under
`.build-artifacts/tmp/native-reopen-m99-installed-r1`. This verifies the scoped
policy on Fedora44, not every Wayland driver or Windows/macOS runtime. Script
Run/Stop/repair and quantitative idle observation remain separate gates.
