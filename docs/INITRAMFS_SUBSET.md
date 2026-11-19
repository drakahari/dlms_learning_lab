# Early-userspace preparation in the simulator

The two lessons under **Prepare early userspace for a recoverable boot** practice
a finite part of LPIC-2 v4.5 objective 201.2. They build on the kernel staging,
boot-entry recovery, and module units. See CHECKPOINT.md for candidate status.

The simulation reuses the supplied kernel, module and boot-entry models. A
generic baseline contains prepared kernel and module trees. Learners edit a
dracut fragment, regenerate an image, and inspect the image selected by the
boot entry. Generation can succeed even when required drivers are missing.
Grading follows the selected image and the modeled root handoff.

## Supported commands and configuration

- dracut --kver RELEASE OUTPUT, or dracut OUTPUT RELEASE; -f/--force overwrites an
  existing virtual initramfs. One explicit supplied kernel and canonical output directly in
  the supplied virtual boot directory (no nested directories) are required. Unknown/default host kernels,
  distro-dependent output defaults and other options are explicitly unsupported.
- In dracut, -k means a kernel-module directory, unlike dkms -k. Never silently
  treat it as --kver. The first subset refuses it with a useful explanation.
- lsinitrd OUTPUT lists only the represented kernel-driver files and release.
  It must label this limited inventory; no fabricated archive sizes, ownership or
  full cpio listing. lsinitrd -m means dracut feature modules, not kernel drivers,
  and is outside the first subset. No unpacking, extraction or host file reads.
- The editable fragment supports literal omit_drivers=VALUE / +=VALUE assignments
  and comments. Quoted or plain literal names only, bounded text; += concatenates
  literal text before splitting the final list, so spacing matters. No shell
  expansion, command execution, scripts, hooks or general config-file evaluation.
  The supplied immutable baseline sets hostonly="no"; it is not a claim about
  another distribution's defaults. Only the five supplied NVMe/ext4 driver names.

All paths are keys in a bounded virtual store. No actual dracut, lsinitrd, compiler,
archive/compressor, firmware, host shell, device, service or network runs. Reject
unsupported syntax honestly before changing state. Keep existing kernel/staging
schemas and saved source snapshots untouched. Console assumes a simulated
administrator; privilege escalation/UID checks are outside this slice.

## Authoritative references and limits

- [LPI201/202objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/),
  v4.5,201-450/202-450, checked2026-11-17;201.2and dracut awareness remain incomplete.
- [Official dracut manual, tag112](https://raw.githubusercontent.com/dracut-ng/dracut/112/man/dracut.8.adoc),
  [tag commit](https://github.com/dracut-ng/dracut/commit/fe9fd498b6fb85c141b13c8067268e3c04110426),
  [tags](https://github.com/dracut-ng/dracut/tags). Available documented revision,
  not a claim that cached tags establish today's latest release.
- [Official configuration manual](https://dracut-ng.github.io/dracut/man/dracut.conf.5.html)
  and [inspection manual](https://dracut-ng.github.io/dracut/man/lsinitrd.1.html),
  checked2026-11-17. Tagged raw configuration/inspection pages returned cache misses;
  these two generated references are not byte-pinned source downloads.

Original factual modeling and authored teaching only; no upstream code, prose or
assets copied or redistributed. No real dracut conformance run is claimed. General
feature modules, host-only discovery, module dependencies outside the supplied
graph, archive/compression bytes, mkinitrd/mkinitramfs and real boot stay excluded.
