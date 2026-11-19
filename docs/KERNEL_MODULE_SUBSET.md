# Supplied loadable-module simulator (capability under development)

No registered lessons or objective credit yet. The retained review candidate is
300/98; this later source is not in it. Implement the capability before authoring.
The runtime never opens object/config/sysfs paths, loads code, calls the kernel,
spawns programs or changes host hardware. All paths identify finite virtual data.

## Deliberate subset

A caller-held catalogue supplies at most32 loadable modules,16 PCI teaching devices,
eight hard dependencies/parameters per module, descriptions, object paths, declared
kernel releases, integer ranges/defaults and fixed external use counts. Dependency
cycles and unknown references refuse admission. A supplied candidate driver is
not necessarily loaded: lspci -k derives active versus possible driver information
from current state. This is not discovery of an actual device or automatic udev.

Runtime state contains an index subset, loaded dependency closure with effective
parameters, one8KiB/64line options file and32 bounded diagnostic events. Commands
are literal and limited to4096bytes. The supported operations are uname -r, lspci
-k, lsmod, modinfo NAME/supplied object path, modprobe NAME [integer options],
modprobe -r NAME, insmod supplied path [integer options], rmmod NAME, depmod -a,
dmesg, and reads of the single supplied config or effective parameter value.
No other filesystem, shell, object code, network or system operation exists.

modprobe resolves current indexed dependencies; insmod requires dependencies to
be loaded already and ignores saved modprobe options. A repeated modprobe load is
a no-op; changed saved options affect later insertion, not existing instances.
Direct insertion of an already present object fails. Removal checks dependent and
fixed external users; modprobe -r also tries unused dependencies while rmmod removes
only its named module. depmod rebuilds an index of supplied current-release objects.
Invalid parameters/releases fail with a separate diagnostic visible through dmesg.
Dependencies inserted before a later insertion failure remain visible. This is not
an atomic kernel transaction. Syntax errors/unsupported commands do not mutate.

Only blank/full-comment/options lines and plain integer scalar assignments are
supported. Duplicate parameters and advanced config are rejected honestly, including
blacklist, alias, install/remove scripts, soft/weak dependencies and continuations.
Drafts can contain errors; runtime insertion does not pretend they were applied.
Unknown parameter names are refused as unsupported before mutation: Linux6.8 may ignore unknown names, so this model does not teach a universal insertion failure for a typo. Known range-invalid integers produce the supplied teaching driver’s operational error. Integers accept decimal or leading-zero octal; hex and plus signs are outside this subset.

lsmod refcounts model only fixed external uses plus direct dependent modules.
modinfo descriptions/size/range annotations are supplied teaching metadata. Index
content is a finite abstraction, not an actual binary modules.dep parser. No built-in
drivers, firmware/USB enumeration, symbols/ELF/ABI execution, alias matching, competing
bindings, auto-loading, module packages, initramfs, reboot, force flags or real kernel
resource behavior. Absence from this model's lsmod cannot teach that a driver absent
from real lsmod is always unavailable: built-in drivers are explicitly outside scope.

## Authoritative references

Checked2026-11-06: official LPIC-1 objectivesv5.0/101-500,102-500 and LPIC-2v4.5/
201-450,202-450. The future lessons address only a narrow101.1/201.3slice, not full
hardware or kernel administration. Existing open rows remain open pending working
lessons and evidence.

- https://www.lpi.org/our-certifications/exam-101-102-objectives/
- https://www.lpi.org/our-certifications/exam-201-202-objectives/
- Upstream kmod v34, release commit889403790977880b8759020338e7cf9634b6ce68;
  modprobe documentation: https://github.com/kmod-project/kmod/blob/v34/man/modprobe.8.scd
- Configuration: https://github.com/kmod-project/kmod/blob/v34/man/modprobe.d.5.scd
- Simple insertion: https://github.com/kmod-project/kmod/blob/v34/man/insmod.8.scd

The upstream release page exposes that full commit link; fetching its commit page
was unavailable. Installed primary manuals are additionally retained privately
with compressed/text hashes (next-modules/INSTALLED_MANUAL_PINS.json). Source was
read as behavioral reference, not copied implementation. No kmod program or actual
kernel/module operation was run. Tests and primary-source review are not a real
kernel comparison, physical-hardware acceptance or full kmod conformance.

Kernel parameter parsing was additionally checked against Linuxv6.8 primary source:
https://github.com/torvalds/linux/blob/v6.8/kernel/module/main.c and
https://github.com/torvalds/linux/blob/v6.8/kernel/params.c (checked2026-11-06).
Only behavior was referenced; no GPL implementation was copied. No actual host
module command or kernel load was run.

The virtual console supplies an authorized operator. Real Linux module insertion/
removal and system configuration generally require administrative privilege; sudo,
real credentials and privilege escalation are not modeled. Session serialization
is capped at256KiB, source at128KiB, and each role keeps at most40 transcript rows
of8192bytes. Invalid/oversized imports retain the prior saved data. These are
simulator limits, not real kernel resource guarantees.
