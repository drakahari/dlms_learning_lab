# Network boot selection in the simulator

This LPIC-2 unit follows DHCP, file transfer and boot-entry recovery.
It distinguishes address assignment from a compatible network bootstrap
program and from the later PXELINUX kernel and initramfs selection. Check
CHECKPOINT.md for integration and candidate status.

## Supported model

- The exercise supplies IPv4 address leases, PXE classification and reachable
  TFTP server addresses. A PXE client supplies one architecture value. The
  model recognizes 0 (PC BIOS) and 7 (x64 UEFI). Real PXE clients can supply a
  list. Missing or unsupported architecture has its own diagnostic.
- A saved per-architecture rule names the server and relative boot filename.
  The simulated server resolves that exact key and reports the kind of
  artifact it supplies. A configured unconditional offer can give ordinary
  DHCP clients a boot file and is checked separately. A filename or suffix
  does not determine program compatibility.
- For the supplied BIOS PXELINUX 6.x branch, the boot program's parent
  directory is the working directory. The same directory supplies
  `ldlinux.c32`. The lookup probes `pxelinux.cfg/` using the supplied UUID,
  Ethernet MAC with hardware type 01, uppercase hexadecimal IPv4 prefixes
  from longest to shortest, then `default`. The first present file is used.
  Its content can fail later without switching to a lower priority match.
- The editable configuration supports case-insensitive `DEFAULT`, one
  `LABEL`, `LINUX` and one `INITRD`. The latter paths are literal and relative
  to the boot program's working directory. The modeled artifact metadata can
  detect an absent file, a wrong file type and a release mismatch.
- This subset takes no action after transfer and selection. It includes no
  real DHCP exchange, PXE packets, sockets, TFTP server, boot binary execution,
  cryptography, Secure Boot, firmware or host file access. There is no
  ProxyDHCP, alternate path-prefix option, real boot scheduling, full Syslinux
  configuration, menu execution, UEFI application internals, or U-Boot.

## Sources

Checked November 17, 2026. LPI lists v4.5 for 201-450/202-450. The LPIC-2 v5.0
wiki page is labeled draft. [LPI objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/) ·
[LPI draft](https://wiki.lpi.org/wiki/LPIC-2_Objectives_V5.0)

The model follows the separation between IP assignment and bootstrap transfer
in [UEFI 2.10 Errata A, chapter 24](https://uefi.org/specs/UEFI/2.10_A/24_Network_Protocols_SNP_PXE_BIS.html).
The architecture values use IANA's current registry. [BOOTP/DHCP options](https://www.iana.org/assignments/bootp-dhcp-parameters) ·
[Processor architecture types](https://www.iana.org/assignments/dhcpv6-parameters#processor-architecture)

Configuration lookup, the required core module for the supplied 6.x profile,
and relative paths follow the Syslinux project's references. [PXELINUX](https://wiki.syslinux.org/wiki/index.php?title=PXELINUX) ·
[Configuration directives](https://wiki.syslinux.org/wiki/index.php?title=Config)

This is original factual modeling. No upstream program, source, or assets
are bundled.
