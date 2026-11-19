# CCNA instructional presentation pilot

Local review only, 2026-11-01; based on `6b54e34869e0`. No push, merge,
installation replacement or public release is authorized for this pilot.

## Magic-number guidance refinement

The first introduction keeps four numbered steps and its worked table, but moves
detailed binary arithmetic and the /27 comparison into expandable help. The
highlight explicitly reads **77 belongs in64–127**.

Try example now teaches beside its inputs: identify the mask value and octet,
calculate `256 − mask`, count boundaries in a compact0–255 range diagram, and
locate the current IP's relevant octet. The containing interval stays unmarked
until a correct check or **Show worked answer**. Editable example IP/prefix values
update guidance and labels; edits invalidate the prior check. Lesson1's supplied
mask/value scaffolding follows the new prefix without filling block answers.

Mask values and octet positions have distinct labels. Numeric block endpoints
are labeled as values of the relevant IP octet; full-address fields retain their
address labels. Neutral format cues never reveal assignment answers. Practice and
assignments use **Check answers**. Assignment layout, stable answer positions,
grading and progress format remain unchanged. Other CCNA lessons and other tracks
keep their earlier presentation.

## Focused revision: entry point and IPv4 teaching

The review app now starts in **CCNA → IPv4 foundations: the magic-number method**.
Choose **Open first lesson · introduction** to open lesson1, **Find a subnet’s
block size**. The unit’s full name and lesson number remain visible in the lesson.
This action opens Learn/Introduction while keeping answers, checks, favorites and
practice drafts. Recent/continue and favorites can still reach earlier work.

The four lessons were already bundled in the first pilot, but the library’s lab
cards followed the old raw catalog order. CCNA cards now follow unit/lesson order;
the new unit appears before **IPv4 addressing and diagnosis**. Other tracks’
ordering is unchanged. The normal201 catalog and all205 review scenario objects
retain their identifiers and grading.

The first introduction explicitly teaches mask-octet selection, `256 − mask`,
boundaries, locating77 in64–127, and deriving full network/broadcast/usable
addresses. Guided missing-step practice precedes independent assignments.

For the existing **Read an IPv4 address** lesson, choose **CCNA → unit: IPv4
addressing and diagnosis → Read an IPv4 address → Introduction**. It uses labeled
calculations and an aligned worked table. Try example separates four specific
actions from observations: Fill worked example, Check configuration, another
usable second-workstation address, then broadcast and retry feedback. The full
assignment retains its topology/five fields with a short requirements checklist.
CCNA prose uses left-aligned prelaid text and ordinary spaces around inline
monospace addresses; column justification no longer stretches it.

## What changes

The complete five-image reference set was inspected in
`learning-lab-reference-05ICWH`: two current Bash screenshots and three proposed
mockups. The mockups guide hierarchy, spacing and separation of expected behavior
from results; they are not implementation evidence. Bash content is not copied.

All 57 existing CCNA assignments retain their IDs, versions, initial state,
requirements and grading. The opt-in CCNA renderer organizes introductions into
objective, prerequisite context, numbered concepts and worked-example context;
practice into expected behavior, numbered actions and its existing workspace;
assignments into one task, requirements and workspace. Addresses use monospace.
Long instructions remain available, rather than being replaced by a quiz.

The existing networking families were inventoried before editing:

| Native family | Existing labs | Tools retained |
| --- | ---: | --- |
| IPv4 addressing / VLSM | 2 | Topology, five-field worksheet / allocation table |
| Client diagnosis | 1 | Configuration and simulated terminal |
| IPv6 | 6 | Address/route editors and simulated tests |
| VLAN switching | 4 | Aligned interface configuration and broadcast inspection |
| IPv4 routing / private office | 7 | Topology, route tables and forwarding tests |
| Discovery | 4 | Topology and modeled discovery messages |
| IPv4 ACL | 4 | Rule editor and packet probes |
| OSPF | 4 | Configuration, adjacencies and routing evidence |
| MAC learning | 4 | Frame actions and forwarding tables |
| DHCPv4 | 3 | Server policy, leases and client tests |
| Spanning tree | 4 | Bridge/port configuration and graph evidence |
| Basic NAT | 3 | Configuration, flows and translation tables |
| EtherChannel | 3 | Member/group configuration and service evidence |
| Physical links | 4 | Link configuration and diagnostics |
| NTP | 4 | Device configuration and simulated status |

LPIC-1, LPIC-2 and Bash renderers/content are unchanged. Shared teaching entry
points explicitly check `track == "CCNA"`; their other-track branches retain the
original rendering. Existing shared control geometry and all 28 themes remain.

## Four new assignments, one first CCNA unit

| Stable ID | Skill | Independent assignment |
| --- | --- | --- |
| `ccna-magic-blocks` | Mask value → block size → containing block | `192.168.50.146/27` |
| `ccna-magic-boundaries` | Network, next subnet, broadcast and conventional host range | `192.168.60.203/28` |
| `ccna-magic-cross-octet` | Third-octet boundaries and all later host octets | `172.19.143.9/20` |
| `ccna-magic-octet-aligned` | Prefix ending at an octet boundary, including carry | `10.24.255.254/24` |

The first worked example is `192.168.10.77/26`: mask `255.255.255.192`,
`256 − 192 = 64`, boundaries `0, 64, 128, 192`, containing range `64–127`,
network `192.168.10.64`, broadcast `192.168.10.127`, conventional usable hosts
`192.168.10.65–192.168.10.126`. The range diagram and aligned calculations make
the connection visible. Optional bit help distinguishes the mask from the IP
and labels network bits / host bits. It explains all-zero and all-one host bits.

Each lesson has Introduction, editable missing-step practice with different
values, and a complete independent worksheet. Requesting a worked practice
answer is optional. Practice never fills or grades the assignment. Challenge
opens the same independent task with help closed. Blank/malformed fields,
mask/address confusion, incorrect block boundaries and reserved host addresses
have specific feedback. Current completion requires checking the current answers;
editing to an invalid answer removes current success.

Calculations support conventional `/1` through `/30`. `/31` point-to-point links
and `/32` host routes are explicitly special cases, not failed applications of
"subtract two." For aligned prefixes the method uses the **last network octet**:
`/24` uses mask octet 3 (`255`, step 1), then resets octet 4. A `/20` step of 16
in octet 3 represents 4,096 whole addresses, not 16. The last IPv4 subnet has no
next subnet; the worksheet accepts `none` in that case.

These four distinct skills are new authored lessons; editable numerical variants
are practice, not additional authored labs. They do not claim complete CCNA
coverage or replace the existing second-workstation deployment assignment.

## Review build and progress protection

```sh
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 python3 tools/dev.py candidate --ccna-pilot
```

The existing runner still owns the shared build lock, disk temporary directory,
two Cargo jobs, two Rust test threads and memory observations. No dependencies
were added. Each review build goes to a new timestamped `dist/ccna-pilot-*` folder.

Run that folder's `dlms-learning-lab` directly; **do not run its installer**.
It defaults to a separate `ccna-presentation-pilot` subdirectory beside the normal
progress file. On the first launch only, it copies existing progress/preferences;
later pilot launches resume that copy. Original progress/preferences are never
written. Normal and pilot locks are separate. Backup/export in the pilot applies
to the pilot profile. Do not import its new lesson records into an older app.

For a development binary, use `--ccna-pilot` to get the same isolated profile and
205-lab / 55-unit review catalog. Without that opt-in the normal 201-lab / 54-unit
catalog remains intact, including historical registration fixtures. The review
catalog prepends one unit and appends four scenarios; all old unit/scenario
objects remain exact. Other-track counts stay LPIC-1 44, LPIC-2 36, Bash 64;
the review build has 61 CCNA assignments in 16 units.

`BUILD_INFO.json` identifies the local uncommitted source inputs, embedded packs,
binary checksum and target. License/theme/third-party notices remain bundled.
This is a Fedora review candidate, not a claim of Windows/macOS runtime,
screen-reader, physical-device or novice usability acceptance.

## Content and implementation boundary

New declarative task/initial-state/goal/hint data is in
`scenarios/ccna-magic-number-v1.json`, with a separate review unit index.
`src/magic_subnet.rs` calculates and diagnoses typed worksheets;
`src/ui/magic_subnet.rs` provides the four teaching stages and native controls.
No terminal, subprocess, network or executable content is introduced. A bounded
`set FIELD VALUE` developer reachability adapter validates pack solutions; it is
not a learner terminal or taught Linux command.

References checked for this pilot on 2026-11-01:

- [Cisco: IP Addressing and Subnetting for New Users](https://www.cisco.com/c/en/us/support/docs/ip/routing-information-protocol-rip/13788-3.html)
- [RFC 1878: Variable Length Subnet Table for IPv4](https://www.rfc-editor.org/rfc/rfc1878)
- [RFC 3021: Using 31-Bit Prefixes on IPv4 Point-to-Point Links](https://www.rfc-editor.org/rfc/rfc3021)

Focused verification evidence and limitations are recorded in CHECKPOINT.md and
the private `.build-artifacts/ccna-pilot/` logs. Reference images and native
captures remain ignored local review evidence, not public repository files.
