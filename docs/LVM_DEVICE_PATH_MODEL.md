# Simulated LVM application references

2026-11-07. Two authored LPIC-2lessons are integrated in the331-lab/113-unit
review catalog. Source/tasks and development native216actions/46screens pass;
final compact widgets and quality gates pass; the corrected separate package
remains pending.
Existing329scenario objects/order and112unit objects/order remain exact.

## Actual behavior

Absolute /dev/VG/LV and simple observed /dev/mapper names resolve to the supplied
virtual group/LV object. Current mapper encoding doubles component hyphens and
uses an undoubled separator between VG and LV. The application reader delegates
to existing virtual filesystem/data/availability checks. Renames change lookup,
not allocations or stored data. No actual /dev paths are read.

Canonical /dev/VG/LV is the recommended application interface. Current mapper
encoding is internal, not a stable ABI. Canonical names also change after renames;
this is not UUID persistence. The terminal still refuses dmsetup and unsupported
Linux commands honestly. Native application Save is a simulator facility, not a
Linux command. Internal simulated-lvm-reference packets are bounded native/data
operations, never executable scripts or a host shell route.

Configurations contain1–4supplied callers with immutable labels and flat virtual
filenames. Each has a saved path and a separate draft. Editing changes no reader;
Save publishes only that supplied reference. Bad bounded references remain
repairable and the actual reader explains blank, malformed, missing, unavailable
or wrong-object states. Unsaved edits clear current success. Goals verify original
object identity, actual retained data, requested reference form, names, protected
storage and unrelated references. A readable path to the wrong original object
fails even with identical file bytes. Flags and transcripts cannot award credit.

Optional default/omitted Definition/Lesson/Session fields keep old wire output
unchanged. Source binding requires complete separate typed practice and assignment,
disjoint storage, bounded authoring steps and actual reachability. Runtime imports
bind config/draft rosters and immutable caller facts to the authored source;
malformed, oversized, duplicate, extra or foreign data is refused without replacing
previous work. Explicit Save, command execution and draft editing publish validated
transactions. Example reset preserves assignment work. Selected assignment reset
uses its fresh authored initial state; old completion history is not fabricated.

## Limits

Names use the existing32-character ASCII subset and paths are at most256bytes.
No internal mapper layers, arbitrary paths, traversal, host files, symlinks, udev,
kernel dm-N nodes, mount/fstab/boot operations, UUID/LABEL references, real devices,
real dmsetup output or network activity. At most one supplied open consumer is
admitted because the retained name-context relation has not proven a reordered
multi-consumer mapping. COW/PV-capacity profiles remain separate. Config fields
are application samples, not a general operating-system configuration editor.
One16KiB runtime per activity and existing64KiB storage/512KiB session bounds apply.
No real LVM/device conformance comparison or physical-system acceptance was run.

## Sources checked2026-11-07

- [Upstream lvm(8)](https://man7.org/linux/man-pages/man8/lvm.8.html): recommended
  application names and the distinction from internal mapper encoding.
- [Upstream dmsetup(8)](https://man7.org/linux/man-pages/man8/dmsetup.8.html): current
  splitname escaping conventions; not an implemented terminal command.
- [Upstream lvrename(8)](https://man7.org/linux/man-pages/man8/lvrename.8.html) and
  [vgrename(8)](https://man7.org/linux/man-pages/man8/vgrename.8.html): name changes
  and command forms. Existing source-enabled subset limits still apply.

No shell-evaluator change, new dependency, original-profile modification or
full-objective/exam/platform/novice acceptance claim. See [lesson evidence](LVM_REFERENCE_LESSONS.md).
