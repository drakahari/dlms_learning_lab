# Virtual iSCSI initiator

The LPIC-2 unit **Connect and maintain iSCSI storage** follows persistent
mounts and stable volume identity. It teaches discovery, node records,
active sessions and a saved startup policy with separate exercises.

## Supported commands and state

- `iscsiadm -m discoverydb -t sendtargets -p PORTAL --discover` reads a
  supplied IPv4 discovery endpoint and saves returned pairs of target name
  and portal. The destination can differ from the discovery address.
  Repeated queries preserve existing node policies and sessions. The
  exercise does not model removal of stale records.
- `iscsiadm -m node` lists saved pairs. A targeted `--op show` reports a
  short subset of the selected record. `--login` and `--logout` change the
  current sessions. These actions require both `-T TARGET` and `-p PORTAL`.
- `iscsiadm -m node -T TARGET -p PORTAL --op update -n node.startup -v VALUE`
  supports `manual` and `automatic`. A saved change does not start or stop
  the current session. `--loginall=automatic` attempts the automatic
  records in the current workspace. The separate **Preview automatic logins**
  reports what the supplied initiator would attempt on a clean start and
  leaves current sessions untouched.
- `iscsiadm -m session` and `-P 3` distinguish active sessions from saved
  records. Detailed output shows attached LUNs and the supplied disk names.
  `lsblk -o NAME,WWN` maps those names to world wide names. The simulator's
  read-only disk metadata replaces any sysfs, udev or block probe. The
  fixture's WWN is the stable disk identifier used here. Other WWID formats
  and `scsi_id` are outside this command subset.
- The console parses literal supported flags in either order, with paired
  short or long flags and supported long `--flag=value` forms. IPv4
  destinations default to port 3260. Output portal group fields are
  informational. The input selector uses the address and port only.

The fixtures represent one virtual interface, a bounded record inventory,
one connection per pair, and known access outcomes. They provide virtual
disk facts and unique session IDs. They include no CHAP, real network
messages or authentication, kernel drivers or adapters, background
processes, service activation, automatic boot from iSCSI, block reads
and writes, multipath, general discovery database administration, or
shell expansion. Unrecognized forms get an explicit diagnostic.

## References

Checked November 17, 2026. Current LPIC-2 v4.5 includes iSCSI access in
204.2. The LPIC-2 v5.0 wiki page remains a separate draft. This unit
covers the supplied initiator tasks within 204.2.K5, T7 and T8. Other
storage access, hardware and tuning items remain open.
[LPI objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/) ·
[LPI draft](https://wiki.lpi.org/wiki/LPIC-2_Objectives_V5.0)

The model follows SendTargets discovery and normal-session distinctions in
[RFC 7143](https://www.rfc-editor.org/rfc/rfc7143.html) and node/session
and startup operations in the [Open-iSCSI upstream guide](https://github.com/open-iscsi/open-iscsi/blob/master/README).
Automatic policy depends on the system login service in real installations.
[Upstream service template](https://github.com/open-iscsi/open-iscsi/blob/master/etc/systemd/iscsi.service.template)

The read-only output follows the relevant published option and columns for
[util-linux lsblk](https://man7.org/linux/man-pages/man8/lsblk.8.html).
The implementation reflects the checked upstream guidance, without a
claim of exact behavior for one Open-iSCSI release or Linux distribution.
All fixture data, screens and simulator code are original.
