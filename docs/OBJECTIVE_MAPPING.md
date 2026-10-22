# Objective evidence and gap ledger

Checked2026-10-22 against the current official [Cisco CCNA v1.1 PDF](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf), [LPIC-1 v5.0 objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/) and [LPIC-2 v4.5 objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/). Current/upcoming version dates remain in [CURRICULUM.md](CURRICULUM.md). Upcoming Cisco v2.0 stays in its separate planning rows; it receives no inherited current-version credit.

[SKILL_COVERAGE.csv](SKILL_COVERAGE.csv) indexes **1240 current references**:136 parent objectives,58 Cisco lettered subobjectives,375 LPI knowledge-area bullets and671 named-tool/term bullets. These references overlap in scope and are **not a completion percentage, curriculum count or additional labs**. There are96 narrowly evidenced Partial practice rows and1144 Open rows; no full-objective completion is asserted. The catalog contains58 authored assignments/fourteen units.

Cisco references retain official IDs. LPI references `.K1`, `.K2`… mean the one-based knowledge-area bullet under that parent; `.T1`, `.T2`… mean the one-based named files/tools/terms bullet. These suffixes are local bookkeeping, not issuer identifiers. Each row links directly to its official parent section and records the source position. A bullet containing several tools remains one source item; partial evidence never implies all of them work.

Rows record exact authored IDs, a short verified scope, relevant test files, the remaining capability/fidelity gap and the proposed parent assignment. Evidence is **explicit for each row**: it is never propagated from parent to child, a tool name to its complete utility, or an implemented command to all related objectives. Open means required teaching/model/content is absent or not accepted. Partial practice means only the named behavior has reviewed instruction/practice or accepted alternatives. The existence of test files validates reference integrity, not the truth of a pedagogical claim; independent content review remains necessary.

The finer review corrected an earlier misleading association: `lpic2-storage` demonstrates capacity and one saved/applied mount, not ext-family/Btrfs/XFS/ZFS/SMART maintenance. LPIC-2 203.2 is now Planned/Open rather than Partial. Narrow actual IPv6 route components/selection, reusable functions, service-journal filtering and zone reload are recorded without promoting their wider objectives.

## Source change detection

[OBJECTIVE_SOURCES.json](OBJECTIVE_SOURCES.json) contains source URLs, versions/check dates, input hashes, reference/position/anchor metadata and normalized SHA-256 fingerprints. Official prose is not copied into this repository. LPI parent fingerprints include the title, full objective description and weight; each description also has its own fingerprint. Cisco wrapped lines are joined before hashing. The HTML input hash identifies the fetched copy; the Cisco input hash identifies extracted PDF text, not the binary PDF.

Every coverage row pins its source fingerprint. A changed description, weight, bullet or ordering requires review before evidence can be carried forward. Regenerating the inventory alone cannot silently validate old claims. Cosmetic surrounding HTML changes are ignored by the semantic comparison; an entirely new exam version requires a separate inventory rather than overwriting current references.

Development-only reproduction, after fetching the public documents to temporary files and extracting Cisco PDF text with `pdftotext -layout`:

```sh
python3 tools/objective_inventory.py --ccna-text /tmp/dlms-objectives-ccna.txt --lpic1-html /tmp/dlms-objectives-lpic1.html --lpic2-html /tmp/dlms-objectives-lpic2.html --checked 2026-10-22 --check docs/OBJECTIVE_SOURCES.json
python3 tools/verify_coverage.py
python3 -m unittest discover -s tools/tests -p test_coverage.py -v
```

The inventory tool reads supplied local source copies only. It performs no network requests or subprocess execution. Offline validation checks exhaustive current references, source fingerprints, parent/kind/position/URL, explicit gaps, existing authored IDs and contained test-evidence paths. It rejects missing/duplicate references, inherited evidence on Open rows and unsupported completion labels. CI checks the committed ledger without downloading objectives. Learners need neither these tools nor internet access.

## Authoring rule

Before adding content, identify the precise source items and implement the required model first. Introduce prerequisites, provide isolated runnable practice, retain a meaningful independent assignment, and test legitimate alternatives, plausible wrong/partial work and success invalidation. Record exact limits and reviewed evidence before updating a row. If a requirement needs unimplemented behavior, leave it Open even when neighboring tasks pass.

The completion checklist remains open: a complete index exposes missing work; it does not resolve those requirements or convert a simulation limitation into full certification preparation.

The ordinary-user access unit adds explicit narrow chgrp evidence for104.5.T4 and strengthens existing regular-file/directory/membership rows. It does not cover special-file/SUID/SGID/sticky-bit/privileged ownership behavior or the full utility syntax.


M25 adds six narrowly partial evidence rows: CCNA v1.1 2.1/2.1.a (ordinary data access only) and2.2/2.2.a/b/c (explicit trunks, restricted802.1Q/native data classification). Default VLAN2.1.b, inter-VLAN2.1.c, switching/MAC learning1.13 and STP2.5 stay Open. A physical forest is a modeling boundary, not STP implementation; data-plane native translation does not predict Catalyst PVST/PVID protection. See VLAN_LESSONS.md and SWITCHING_SUBSET.md. Future v2.0 rows remain separately planned, not silently promoted from current practice.


M28 adds narrowly partial103.3.K4/T19 evidence for script-VM simple * selection, quoting/hidden/no-match handling and safe changing-file reports. Advanced/bracket/directory patterns and cp/mv/rm terminal expansion remain absent. A tested ? capability alone is not claimed as authored instruction. All full-objective and future-blueprint requirements stay open.
