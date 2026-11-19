# ATA SMART diagnosis and monitoring in the simulator

This LPIC-2 unit practices identifying a supplied disk, separating its
overall SMART headline from individual self-test results, and selecting
the intended drives in a small monitoring configuration. The two lab
workspaces use different virtual input for practice and grading.

## Diagnostic console

- One literal `smartctl` request followed by a supplied `/dev/sdX` or
  `/dev/disk/by-id/ata-…` selector. The aliases resolve against the
  virtual inventory. The console never opens a device or file.
- `-i`, `-H` and `-c` show a condensed device identity, supplied
  health result and represented running-test state. A passing overall
  status can coexist with a logged read error.
- `-t short` starts one non-captive short test for that disk with a
  fixture duration. The user advances the virtual clock by a minute at a
  time. On completion the model appends the supplied result to the per-disk
  history. The simulator does not change the existing health headline.
- `-l selftest` reports the newest virtual result first, with power-on
  age and the first-error LBA for a read failure. The supported commands
  return 0, bit 3 for failing overall health, or bit 7 for a retained
  failed self-test entry. Those are the represented command/result
  combinations. Exit status 128 in a read-only result can reflect an
  older failed entry. Unsupported syntax has a separate diagnostic.
- The exercise uses short ATA tests and a capped log. It does not
  represent extended/self-test supersession rules, drive safety
  predictions, vendor-specific attributes, device repair, ATA command
  transport, SCSI, NVMe, or hardware time.

The maintenance ticket compares the learner's record to the latest
completed test on the intended disk. A pending test, an older LBA, and
unintended test changes on another disk have different feedback.

## Virtual monitoring rules

The saved subset accepts one literal explicit ATA device selector per
line followed by `-H`, `-l selftest` or both, in either order. It supports
separated whitespace, comment lines and LF/CRLF endings. Unsupported
directives receive a diagnostic. Save keeps the previous configuration
if syntax validation fails.

The two previews resolve the saved selectors against the supplied
current/maintenance layouts and display the selected disk serial and
enabled checks. The monitor evaluates identity and intent. It does not
start a smartd process, examine sysfs/udev, poll devices, execute a
schedule, send email/alerts or access the host. No scan, default profiles,
device type probing, continuation lines, reattachment or error recovery
are included.

## Sources

Checked November 17, 2026. Current LPIC-2 v4.5 objective 203.2 includes
monitoring SMART devices. The lab subset belongs to its T7 entry.
Other storage tools and filesystems remain separately tracked.
[LPI current objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/)

The model uses the relevant ATA command, test-completion and log terms
from the upstream manual.
[smartctl manual source](https://github.com/smartmontools/smartmontools/blob/main/src/smartctl.8.in) ·
[Published exit-status reference](https://manpages.debian.org/trixie/smartmontools/smartctl.8.en.html#EXIT_STATUS)

The modeled explicit monitoring options come from the published project
examples and implementation. The app previews configuration resolution
only. All fixture data, screens and simulator code are original.
[smartd.conf manual mirror](https://github.com/mirror/smartmontools/blob/master/smartd.conf.5.in) ·
[Upstream smartd source reference](https://www.smartmontools.org/static/doxygen/smartd_8cpp_source.html)
