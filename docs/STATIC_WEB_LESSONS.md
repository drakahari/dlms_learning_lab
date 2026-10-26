# Static web services unit

Authored 2026-10-26. **Four data-complete assignments are currently unbundled.**
Their native editor and normal catalog registration are not accepted yet. They do
not increase the 133-lab installed catalog or promote objective coverage. The
eight content test groups exercise actual in-process behavior, not a real server.

| Assignment | Distinct skill | Prerequisite |
| --- | --- | --- |
| Route site names without changing the default site | Move an alias, retain two named sites and the unknown-name fallback | Text editing; introduction supplies HTTP and configuration-block terms |
| Choose a landing page and publish a correction | Ordered directory pages, fallback, live file content, GET/HEAD and slash redirects | Site-name routing |
| Keep public pages open and staff notes private | More specific directory policy, allowed/denied paths, sibling ancestry and intact private files | Directory pages |
| Hand over a release that survives its next start | Diagnose saved/active mismatch, publish content, retain alias, private policy and a healthy neighbouring site | Previous three skills |

Each introduction explains the new concept before assessment. A separate example
has different site/file identities, annotated editor actions, a suggested change,
declared failing outcome rows and a repair. Insert fills only a practice draft;
Save and activation remain explicit. Challenge retains the full assignment with
help concealed by default. The example never earns assignment credit.

The four assignments contain 50 explicit outcome rows and their separate examples
another 50. These are assertions within four labs, not 100 scenarios. Static pages,
selected site, exact mapped resource, body/UTF-8 byte length and redirect targets
are independently stated. Healthy current and next-start HTTP200 responses are
mandatory alongside denials and preservation. A stopped service cannot stand in
for access control. A next-start test uses a simulation copy; it neither stops the
learner's current state nor proves operating-system boot behavior.

The taught methods, alternate directive case/layout and reordered file publication
with Stop/Start pass. Tests reject save-only repairs, stale active/saved shortcuts,
stopping the service, superficially successful wrong pages, broken success,
foreign example context and invalid/unreachable packs. Every one of the 50 assignment
rows also has a real state mutation that makes that row fail. Practice experiment/repair,
pending drafts, read-only query edits, favorites, actual backup/restore and selected
reset are checked separately. The first content run caught an independently
miscounted HEAD length in a test: the 28-character message plus newline is 29 bytes.
The model and authored expectation were correct; the test was corrected.

## References and honest scope

Checked 2026-10-26: LPI still lists LPIC-2 **v4.5**, exams 201-450/202-450.
Topic208.1 includes substantially more than these static-site exercises. Installation,
authentication, modules/scripts, load/performance/resource administration and real
server operation remain outside this unit; HTTPS, proxies and Nginx remain separate
capability gaps. No full-objective, endorsement, security-hardening or certification
preparation claim follows from this content.

- [LPI current version](https://www.lpi.org/our-certifications/lpic-2-overview/)
- [LPI Topic208.1 and other objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/)
- [Apache2.4 virtual-host matching](https://httpd.apache.org/docs/2.4/en/vhosts/details.html)
- [Apache2.4 directory pages and redirects](https://httpd.apache.org/docs/2.4/en/mod/mod_dir.html)
- [Apache2.4 authorization inheritance](https://httpd.apache.org/docs/2.4/en/mod/mod_authz_core.html)
- [HTTP HEAD semantics](https://www.rfc-editor.org/rfc/rfc9110.html#section-9.3.2)

The closed configuration grammar, request restrictions and fixed virtual-file
limits are in [APACHE_STATIC_SUBSET.md](APACHE_STATIC_SUBSET.md) and
[APACHE_STATIC_WORKSPACE.md](APACHE_STATIC_WORKSPACE.md). There are no real
listeners, DNS requests, Linux commands or host file operations in these lessons.


Independent source review found that testing private-file denial under only the
main name could miss an alias routed to a second public virtual host with the
same ServerName. The capstone now explicitly checks denial through both names,
currently and after next start. A regression constructs that valid source-order
configuration, confirms the alias leaks HTTP200, and requires exactly the two
alias-private rows to fail while all other promises still pass. The model allows
legitimate duplicate-name ordering; the task checks its actual promise instead
of banning valid configurations to hide the gap.
