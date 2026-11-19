# Known-leaf account-template maintenance

Development capability, 2026-11-05, now used by `lpic1-skeleton-handover` in the
source287/90catalog. Bounded core/draft/teaching/source adapters, authored outcomes
and actual native113-action review pass. The separate287Fedora candidate also
passes packaged/native/private-upgrade and installed NFS cold-reopen gates.
See ACCOUNT_SKELETON_LESSON.md and CHECKPOINT.md. Evaluator19 is unchanged.

The existing Accounts model already copies current supplied skeleton entries when
its simulated useradd creates a home. The new optional policy permits changes to
at most four known ordinary-text leaves strictly inside a supplied skeleton. Its
empty default stays omitted from old serialized policies. There is no new terminal
command: SaveSkeletonLeaf and RemoveSkeletonLeaf are typed native actions only.

A save can change bounded text and permission mode000–777; the source UID, GID and
text-file kind stay fixed. Removal must be explicitly allowed. That fixed identity
survives removal/recreation. Ancestor directories, every unlisted template entry,
other supplied skeletons and explicitly protected leaves stay unchanged. Full
held-source policy comparison prevents imported editable rights authorizing
themselves. State.valid_initial requires all declared source leaves present;
State.valid permits legitimate runtime removal. Native source-aware actions check
the original before and after a bounded atomic candidate transition.

Existing limits remain:32users/32groups,256entries,64KiB per text file,256KiB total
text,4MiB raw State input, bounded paths/outputs. No special permission bits, links,
renames, arbitrary new paths or general /etc editing are added. New policy readers
reject duplicate/excess keys and oversized names. A structurally valid imported
runtime State alone is not the authored source or a permission grant.

Existing useradd -m/default or supplied -k copies the edited text, dotfiles, nested
entries and child modes, remapping numeric ownership to the new account. Home
root0700 is this exercise's policy, not a universal Linux default. An already
created home is not updated retroactively. No profile text is executed, and there
is no real account, authentication/login/PAM, shell startup, host filesystem or
system change. Choosing another unchanged skeleton would repeat onboarding;
it is not sufficient curriculum coverage for maintaining defaults.

Actual core gate:18new groups and27retained Accounts groups pass. Retained
content11/workspace18/feedback6 pass. These exercise real existing in-process
copying, valid alternatives, wrong/early creation, source/identity/path/limit
refusals and preserved old homes. The first retained command used a nonexistent
plural target name and was rejected before execution; its log remains retained.
No native, new-lesson or host-reference acceptance follows from these core tests.

The optional draft/teaching adapter adds source-bound known-file drafts (8KiB per
file), raw invalid octal/NUL retention for retry, selected-file atomic Save, and
separate native example steps. It never fills assignment drafts from its author
reference. Seventeen adapter groups and seven source/workspace/persistence groups
pass, including partial work, stale-success invalidation, mismatched/erased rights
or metadata, failed initialization atomicity, unchanged disk on refused saves,
and reset/resume. All unrelated closed workspace projections explicitly reject
the new fields. No historical shell collector or sealed evidence is rewritten.

Native widget tests and actual development rendering now pass; see the lesson
guide for scope and failed-run retention. Adapter tests alone do not establish
rendered usability or curriculum coverage. Current parser, AddUser semantics,
old empty-policy wire, progress schema and installed owner app remain unchanged.

Technical source checked2026-11-05: shadow-maint useradd(8), mirrored at
https://man7.org/linux/man-pages/man8/useradd.8.html (-k, -m, NOTES). Copying starter
files happens when a home is created; this capability models ordinary text files
only, without symlink rewriting, ACL/xattrs or host authentication.
