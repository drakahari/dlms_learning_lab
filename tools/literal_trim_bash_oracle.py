#!/usr/bin/env python3
"""Fixed development literal prefix/suffix comparator. MAIN runner only; no learner input.

prepare launches nothing; capture requires exact Root approval; verify is read-only.
No source/pattern/binary/path/fixture/worktree/capture-ID selector is accepted.
"""
import argparse
import json
import os
from pathlib import Path
import re
import stat
import sys
import time
import ere_reference as shared
import case_classes_bash_oracle as plumbing

safe = shared.safe
ROOT = Path(__file__).resolve().parents[1]
BASH = Path('/usr/bin/bash')
MANUAL = Path('/usr/share/man/man1/bash.1.gz')
BASE = '.build-artifacts/reviews/bash-literal-trim/reference-captures'
CAPTURE = 'literal32-r1'
FIXTURE = 'tests/fixtures/shell-literal-trim-reference.json'
PROBE_SOURCE = 'examples/literal_trim_reference.rs'
PROBE = 'target/debug/examples/literal_trim_reference'
CAP = 2 * 1024 * 1024
BINARY_CAP = 256 * 1024 * 1024
EVIDENCE_CAP = 512 * 1024 * 1024
DEADLINE = 120.0
FIXTURE_SHA = '4b6b0cc10d4e98ac5ce14fe48a254f823d8f3cd25fe6120fd84dca868589e935'
# Generated literal inventories are closed, not caller-selected.
# Conservative closed library/project input superset; no caller-selected roster.
SOURCE_FAMILIES = ('src', 'scenarios', 'assets')
COMPILED = ('Cargo.lock', 'Cargo.toml', 'assets/licenses/upstream/AccessKit/accesskit/0b6901a8c8e6fb1cb5a2dc7c19dc7bee1cc5ad09/LICENSE-APACHE', 'assets/licenses/upstream/AccessKit/accesskit/0b6901a8c8e6fb1cb5a2dc7c19dc7bee1cc5ad09/LICENSE-MIT', 'assets/licenses/upstream/AccessKit/accesskit/0b6901a8c8e6fb1cb5a2dc7c19dc7bee1cc5ad09/LICENSE.chromium', 'assets/licenses/upstream/AccessKit/accesskit/5801c4a49ff9b2c9ec3f77b3c31272d0da4c5cd3/LICENSE-APACHE', 'assets/licenses/upstream/AccessKit/accesskit/5801c4a49ff9b2c9ec3f77b3c31272d0da4c5cd3/LICENSE-MIT', 'assets/licenses/upstream/AccessKit/accesskit/5801c4a49ff9b2c9ec3f77b3c31272d0da4c5cd3/LICENSE.chromium', 'assets/licenses/upstream/DoumanAsh/clipboard-win/3b27cf2bfd1adcfa6e0264eb51c1025ddaf0f342/LICENSE', 'assets/licenses/upstream/aclysma/profiling/8271551172eb6fa4cba47369aedd93790c623df9/LICENSE-APACHE', 'assets/licenses/upstream/aclysma/profiling/8271551172eb6fa4cba47369aedd93790c623df9/LICENSE-MIT', 'assets/licenses/upstream/brendanzab/gl-rs/ea503e8d5fb6d73c6030e6191ce738cd3bf3433e/LICENSE', 'assets/licenses/upstream/brendanzab/gl-rs/f150967b1c44ae888e6676f93f639ebc82771bdc/LICENSE', 'assets/licenses/upstream/emilk/egui/44cdd653e2317d300fb8a6c9c36b03f23991e803/LICENSE-APACHE', 'assets/licenses/upstream/emilk/egui/44cdd653e2317d300fb8a6c9c36b03f23991e803/LICENSE-MIT', 'assets/licenses/upstream/jni-rs/jni-rs/33045a124105c939d1e2cbdcb5a39e5d868ffa03/LICENSE-APACHE', 'assets/licenses/upstream/jni-rs/jni-rs/33045a124105c939d1e2cbdcb5a39e5d868ffa03/LICENSE-MIT', 'assets/licenses/upstream/jni-rs/jni-rs/5ae9458a4ec44c5318f37ddc7569c1d4ae8a69e7/LICENSE-APACHE', 'assets/licenses/upstream/jni-rs/jni-rs/5ae9458a4ec44c5318f37ddc7569c1d4ae8a69e7/LICENSE-MIT', 'assets/licenses/upstream/jni-rs/jni-sys/64d77b7a5f119d7b55b4e2c169a4668067ff59e6/LICENSE-APACHE', 'assets/licenses/upstream/jni-rs/jni-sys/64d77b7a5f119d7b55b4e2c169a4668067ff59e6/LICENSE-MIT', 'assets/licenses/upstream/madsmtm/objc2/4fc083f1c6d6784577e38b0ee8dbd344481e2fd2/LICENSE.txt', 'assets/licenses/upstream/madsmtm/objc2/5237cbf081d04ef75a1f6a207b4018e7c1ae7438/LICENSE.txt', 'assets/licenses/upstream/madsmtm/objc2/7b1abfd750a2cacaea71d6a56ecfb83cb7de560b/LICENSE.md', 'assets/licenses/upstream/madsmtm/objc2/8852b424193ca41602281b3d7540d7c8ed51e49a/LICENSE.md', 'assets/licenses/upstream/madsmtm/objc2/8d214f5477365ffcbcbb7de058c86ed9a518efb7/LICENSE.md', 'assets/licenses/upstream/madsmtm/objc2/d7d2fa23ceaa5e6096c923b081040e5d81b3b9df/LICENSE.md', 'assets/licenses/upstream/madsmtm/objc2/e282618be4c3a3b9542957e0c8540e9588472ce8/LICENSE.txt', 'assets/licenses/upstream/manifest-v1.json', 'assets/licenses/upstream/odilia-app/atspi/ef2b4d1b41a9d53a9bb5b484661d712e0d1964e7/LICENSE-APACHE2.txt', 'assets/licenses/upstream/odilia-app/atspi/ef2b4d1b41a9d53a9bb5b484661d712e0d1964e7/LICENSE-MIT.txt', 'assets/licenses/upstream/rust-mobile/ndk/49bbbba16c58ff63cb8a0ad0eca5a9fb7ecaec25/LICENSE-APACHE', 'assets/licenses/upstream/rust-mobile/ndk/49bbbba16c58ff63cb8a0ad0eca5a9fb7ecaec25/LICENSE-MIT', 'assets/licenses/upstream/rust-windowing/android-ndk-rs/10f2ba388fca20f7349996ebae26ccda7a6fda5c/LICENSE-APACHE', 'assets/licenses/upstream/rust-windowing/android-ndk-rs/10f2ba388fca20f7349996ebae26ccda7a6fda5c/LICENSE-MIT', 'assets/themes/LICENSE.dlms', 'assets/themes/LICENSE.omarchy', 'assets/themes/dlms-stock.json', 'assets/themes/omarchy-catppuccin-latte.toml', 'assets/themes/omarchy-catppuccin.toml', 'assets/themes/omarchy-ethereal.toml', 'assets/themes/omarchy-everforest.toml', 'assets/themes/omarchy-flexoki-light.toml', 'assets/themes/omarchy-gruvbox.toml', 'assets/themes/omarchy-hackerman.toml', 'assets/themes/omarchy-kanagawa.toml', 'assets/themes/omarchy-last-horizon.toml', 'assets/themes/omarchy-lumon.toml', 'assets/themes/omarchy-lupine.toml', 'assets/themes/omarchy-manifest.json', 'assets/themes/omarchy-matte-black.toml', 'assets/themes/omarchy-miasma.toml', 'assets/themes/omarchy-nord.toml', 'assets/themes/omarchy-osaka-jade.toml', 'assets/themes/omarchy-retro-82.toml', 'assets/themes/omarchy-ristretto.toml', 'assets/themes/omarchy-rose-pine.toml', 'assets/themes/omarchy-solitude.toml', 'assets/themes/omarchy-tokyo-night.toml', 'assets/themes/omarchy-vantablack.toml', 'assets/themes/omarchy-white.toml', 'docs/SHELL_SUBSET.md', 'examples/literal_trim_reference.rs', 'examples/support/automation_digest.rs', 'scenarios/accounts-lifecycle-v1.json', 'scenarios/acl-policy-v1.json', 'scenarios/apache-static-v1.json', 'scenarios/debian-packages-v1.json', 'scenarios/dhcpv4-client-v1.json', 'scenarios/discovery-v1.json', 'scenarios/disk-preparation-v1.json', 'scenarios/dns-forward-v1.json', 'scenarios/dns-reverse-v1.json', 'scenarios/etherchannel-v1.json', 'scenarios/file-backup-v1.json', 'scenarios/ipv4-private-planning-v1.json', 'scenarios/ipv4-static-v1.json', 'scenarios/ipv6-static-v1.json', 'scenarios/linux-access-v1.json', 'scenarios/linux-cron-v1.json', 'scenarios/linux-files-v1.json', 'scenarios/linux-find-v1.json', 'scenarios/linux-one-off-v1.json', 'scenarios/linux-processes-v1.json', 'scenarios/lvm-growth-v1.json', 'scenarios/mac-learning-v1.json', 'scenarios/nat-basic-v1.json', 'scenarios/nfs-v1.json', 'scenarios/ntp-client-server-v1.json', 'scenarios/ospf-p2p-v1.json', 'scenarios/physical-links-v1.json', 'scenarios/rpm-packages-v1.json', 'scenarios/shell-arrays-v1.json', 'scenarios/shell-capture-v1.json', 'scenarios/shell-case-v1.json', 'scenarios/shell-command-vectors-v1.json', 'scenarios/shell-composition-v1.json', 'scenarios/shell-counts-v1.json', 'scenarios/shell-ere-evidence-v1.json', 'scenarios/shell-file-batches-v1.json', 'scenarios/shell-getopts-v1.json', 'scenarios/shell-loop-recovery-v1.json', 'scenarios/shell-numeric-validation-v1.json', 'scenarios/shell-optional-inputs-v1.json', 'scenarios/shell-reusable-v1.json', 'scenarios/shell-safe-automation-v1.json', 'scenarios/shell-scope-v1.json', 'scenarios/shell-text-v1.json', 'scenarios/shell-unit-v1.json', 'scenarios/spanning-tree-v1.json', 'scenarios/starter-v1.json', 'scenarios/storage-v1.json', 'scenarios/unit-manager-v1.json', 'scenarios/units-v1.json', 'scenarios/vlan-data-v1.json', 'src/accounts.rs', 'src/accounts_teaching.rs', 'src/acl.rs', 'src/acl_editor.rs', 'src/acl_routing.rs', 'src/acl_teaching.rs', 'src/acl_workspace.rs', 'src/address_teaching.rs', 'src/apache_static.rs', 'src/apache_static_teaching.rs', 'src/apache_static_workspace.rs', 'src/backup.rs', 'src/catalog.rs', 'src/client_teaching.rs', 'src/course.rs', 'src/cron.rs', 'src/cron_teaching.rs', 'src/cron_workspace.rs', 'src/cron_workspace/budget.rs', 'src/debian_packages.rs', 'src/debian_packages_teaching.rs', 'src/debian_packages_terminal.rs', 'src/debian_packages_terminal/session.rs', 'src/debian_packages_workspace.rs', 'src/dhcpv4.rs', 'src/dhcpv4_teaching.rs', 'src/dhcpv4_workspace.rs', 'src/discovery.rs', 'src/discovery_teaching.rs', 'src/discovery_workspace.rs', 'src/dns.rs', 'src/dns_teaching.rs', 'src/dns_workspace.rs', 'src/engine.rs', 'src/ere.rs', 'src/ere/cooperative.rs', 'src/etherchannel.rs', 'src/etherchannel_teaching.rs', 'src/etherchannel_workspace.rs', 'src/file_backup.rs', 'src/file_backup_teaching.rs', 'src/file_backup_terminal.rs', 'src/file_backup_workspace.rs', 'src/file_backup_workspace/presentation.rs', 'src/filesystem.rs', 'src/filesystem/find.rs', 'src/filesystem/permissions.rs', 'src/hsrp.rs', 'src/ipv4.rs', 'src/ipv4_routing.rs', 'src/ipv4_routing_teaching.rs', 'src/ipv4_routing_workspace.rs', 'src/ipv6.rs', 'src/ipv6_teaching.rs', 'src/ipv6_workspace.rs', 'src/legacy_access_teaching.rs', 'src/legacy_logs_teaching.rs', 'src/lib.rs', 'src/links.rs', 'src/local_file.rs', 'src/lvm.rs', 'src/lvm_workspace.rs', 'src/mac_learning.rs', 'src/mac_learning_teaching.rs', 'src/mac_learning_workspace.rs', 'src/main.rs', 'src/nat.rs', 'src/nat_teaching.rs', 'src/nat_workspace.rs', 'src/native_observations.rs', 'src/nfs.rs', 'src/nfs_teaching.rs', 'src/nfs_workspace.rs', 'src/ntp.rs', 'src/ntp_workspace.rs', 'src/one_off.rs', 'src/one_off_teaching.rs', 'src/one_off_workspace.rs', 'src/one_off_workspace/budget.rs', 'src/one_off_workspace/decode.rs', 'src/ospf.rs', 'src/ospf_teaching.rs', 'src/ospf_workspace.rs', 'src/pack.rs', 'src/partition_fs.rs', 'src/partition_fs_workspace.rs', 'src/partition_fs_workspace/binding.rs', 'src/physical_link.rs', 'src/physical_link_assignment.rs', 'src/physical_link_teaching.rs', 'src/physical_link_workspace.rs', 'src/process_teaching.rs', 'src/processes.rs', 'src/progress.rs', 'src/rpm_packages.rs', 'src/rpm_packages_teaching.rs', 'src/rpm_packages_terminal.rs', 'src/rpm_packages_workspace.rs', 'src/script_teaching.rs', 'src/seed_dns_teaching.rs', 'src/seed_mount_teaching.rs', 'src/service_teaching.rs', 'src/shell.rs', 'src/shell/arithmetic.rs', 'src/shell/capture.rs', 'src/shell/case.rs', 'src/shell/cron.rs', 'src/shell/execution.rs', 'src/shell/getopts.rs', 'src/shell/globbing.rs', 'src/shell/grep.rs', 'src/shell/literal_trim.rs', 'src/shell/loop_control.rs', 'src/shell/one_off.rs', 'src/shell/parameters.rs', 'src/shell/preparation.rs', 'src/shell/substitution.rs', 'src/shell/text_tools.rs', 'src/shell/values.rs', 'src/spanning_tree.rs', 'src/spanning_tree_teaching.rs', 'src/spanning_tree_workspace.rs', 'src/startup_trace.rs', 'src/storage.rs', 'src/storage_teaching.rs', 'src/storage_workspace.rs', 'src/switching.rs', 'src/switching_teaching.rs', 'src/switching_workspace.rs', 'src/terminal_teaching.rs', 'src/theme.rs', 'src/ui.rs', 'src/ui/accounts_workspace.rs', 'src/ui/acl_workspace.rs', 'src/ui/address_learning.rs', 'src/ui/apache_static_workspace.rs', 'src/ui/client_learning.rs', 'src/ui/completion_index.rs', 'src/ui/completion_index/tests.rs', 'src/ui/controls.rs', 'src/ui/cron_workspace.rs', 'src/ui/cron_workspace/memo.rs', 'src/ui/data.rs', 'src/ui/debian_packages_workspace.rs', 'src/ui/dhcpv4_workspace.rs', 'src/ui/discovery_workspace.rs', 'src/ui/dns_workspace.rs', 'src/ui/etherchannel_workspace.rs', 'src/ui/etherchannel_workspace/memo.rs', 'src/ui/file_backup_workspace.rs', 'src/ui/ipv4_routing_workspace.rs', 'src/ui/ipv4_workspace.rs', 'src/ui/ipv6_workspace.rs', 'src/ui/legacy_learning.rs', 'src/ui/lvm_workspace.rs', 'src/ui/mac_learning_workspace.rs', 'src/ui/nat_workspace.rs', 'src/ui/nat_workspace/memo.rs', 'src/ui/nfs_workspace.rs', 'src/ui/nfs_workspace/memo.rs', 'src/ui/ntp_workspace.rs', 'src/ui/one_off_boundary.rs', 'src/ui/one_off_workspace.rs', 'src/ui/one_off_workspace/input.rs', 'src/ui/one_off_workspace/memo.rs', 'src/ui/ospf_workspace.rs', 'src/ui/partition_fs_workspace.rs', 'src/ui/physical_link_workspace.rs', 'src/ui/physical_link_workspace/memo.rs', 'src/ui/physical_link_workspace/tests.rs', 'src/ui/process_learning.rs', 'src/ui/rpm_packages.rs', 'src/ui/rpm_packages/memo.rs', 'src/ui/script_learning.rs', 'src/ui/scripting.rs', 'src/ui/seed_learning.rs', 'src/ui/service_learning.rs', 'src/ui/smoke.rs', 'src/ui/smoke/accounts.rs', 'src/ui/smoke/accounts_lessons.rs', 'src/ui/smoke/acl.rs', 'src/ui/smoke/acl_lessons.rs', 'src/ui/smoke/apache_static_lessons.rs', 'src/ui/smoke/capture_lessons.rs', 'src/ui/smoke/case_history.rs', 'src/ui/smoke/client_learning.rs', 'src/ui/smoke/command_vectors.rs', 'src/ui/smoke/cron_lessons.rs', 'src/ui/smoke/debian_packages_lessons.rs', 'src/ui/smoke/dhcpv4.rs', 'src/ui/smoke/dhcpv4_lessons.rs', 'src/ui/smoke/disk_preparation.rs', 'src/ui/smoke/dns_reverse.rs', 'src/ui/smoke/ere_evidence.rs', 'src/ui/smoke/etherchannel_lessons.rs', 'src/ui/smoke/file_backup_lessons.rs', 'src/ui/smoke/find_lessons.rs', 'src/ui/smoke/legacy_terminal.rs', 'src/ui/smoke/loop_recovery.rs', 'src/ui/smoke/lvm_lessons.rs', 'src/ui/smoke/mac_layout.rs', 'src/ui/smoke/mac_learning.rs', 'src/ui/smoke/mac_lessons.rs', 'src/ui/smoke/nat.rs', 'src/ui/smoke/nat_lessons.rs', 'src/ui/smoke/nfs_alignment.rs', 'src/ui/smoke/nfs_lessons.rs', 'src/ui/smoke/ntp_lessons.rs', 'src/ui/smoke/numeric_validation.rs', 'src/ui/smoke/one_off_lessons.rs', 'src/ui/smoke/optional_inputs.rs', 'src/ui/smoke/ospf.rs', 'src/ui/smoke/ospf_lessons.rs', 'src/ui/smoke/physical_link_lessons.rs', 'src/ui/smoke/private_office.rs', 'src/ui/smoke/reopen.rs', 'src/ui/smoke/reopen/dhcpv4.rs', 'src/ui/smoke/reopen/nfs.rs', 'src/ui/smoke/rpm_lessons.rs', 'src/ui/smoke/safe_automation_lessons.rs', 'src/ui/smoke/script_feedback.rs', 'src/ui/smoke/seed_teaching.rs', 'src/ui/smoke/spanning_tree.rs', 'src/ui/smoke/spanning_tree_lessons.rs', 'src/ui/smoke/substitution.rs', 'src/ui/smoke/unit_manager.rs', 'src/ui/smoke/unit_manager_lessons.rs', 'src/ui/spanning_tree_workspace.rs', 'src/ui/storage_workspace.rs', 'src/ui/switching_workspace.rs', 'src/ui/terminal_learning.rs', 'src/ui/tests.rs', 'src/ui/tests/apache_static.rs', 'src/ui/tests/apache_static_lessons.rs', 'src/ui/tests/client_learning.rs', 'src/ui/tests/command_vectors.rs', 'src/ui/tests/completion_index.rs', 'src/ui/tests/cron_lessons_native.rs', 'src/ui/tests/cron_workspace.rs', 'src/ui/tests/debian_packages.rs', 'src/ui/tests/dhcpv4.rs', 'src/ui/tests/dhcpv4_lessons.rs', 'src/ui/tests/disk_preparation_native.rs', 'src/ui/tests/ere_evidence_native.rs', 'src/ui/tests/etherchannel_native.rs', 'src/ui/tests/etherchannel_workspace.rs', 'src/ui/tests/file_backup_lessons_native.rs', 'src/ui/tests/file_backup_workspace.rs', 'src/ui/tests/getopts_history.rs', 'src/ui/tests/getopts_lessons.rs', 'src/ui/tests/legacy_learning.rs', 'src/ui/tests/loop_recovery.rs', 'src/ui/tests/lvm_growth.rs', 'src/ui/tests/lvm_lessons_native.rs', 'src/ui/tests/mac_learning.rs', 'src/ui/tests/nat.rs', 'src/ui/tests/nat_lessons.rs', 'src/ui/tests/nat_lessons_native.rs', 'src/ui/tests/nat_native.rs', 'src/ui/tests/nat_teaching.rs', 'src/ui/tests/nfs_lessons_native.rs', 'src/ui/tests/nfs_workspace.rs', 'src/ui/tests/ntp_client_server.rs', 'src/ui/tests/ntp_lessons_native.rs', 'src/ui/tests/numeric_validation.rs', 'src/ui/tests/one_off_admission.rs', 'src/ui/tests/one_off_lessons_native.rs', 'src/ui/tests/one_off_physical.rs', 'src/ui/tests/one_off_workspace.rs', 'src/ui/tests/optional_inputs.rs', 'src/ui/tests/ospf.rs', 'src/ui/tests/partition_fs_workspace.rs', 'src/ui/tests/physical_native_controller.rs', 'src/ui/tests/physical_normal_catalog.rs', 'src/ui/tests/physical_workspace.rs', 'src/ui/tests/private_office.rs', 'src/ui/tests/rpm_lessons_native.rs', 'src/ui/tests/rpm_packages.rs', 'src/ui/tests/safe_automation_lessons.rs', 'src/ui/tests/seed_learning.rs', 'src/ui/tests/spanning_tree.rs', 'src/ui/tests/ui_consistency.rs', 'src/ui/tests/unit_manager.rs', 'src/ui/unit_manager_workspace.rs', 'src/ui/workspace.rs', 'src/unit_manager.rs', 'src/unit_manager/journal.rs', 'src/unit_manager_teaching.rs', 'src/unit_manager_terminal.rs', 'src/unit_manager_terminal/session.rs', 'src/unit_manager_workspace.rs', 'src/workspace.rs', 'tests/fixtures/shell-literal-trim-reference.json')
CORE_PINS = {'src/ere.rs': 'e0543af81bb9388b71a1edd1fcfa770efcba12eab3847046079a36b95fb3db7d', 'src/ere/cooperative.rs': 'd71ff8619a5ce77a0a55ff337785600cd1e926cb007cfc5ce3e0c18de7304d62', 'src/shell.rs': '310f6509f2679e1b581f8478a50d62dbfdd961c7ee67d18e2ed7738944ca43f0', 'src/shell/arithmetic.rs': 'f425685c0e44f91e43e75c4f274ce1cdea58af7d2de0f6306c2cc24ec1da7c2e', 'src/shell/capture.rs': '2127c0c7a37d7ac2858695828026ce508b783aad84fd79cb0561518f3a725b4a', 'src/shell/case.rs': '4995592bd49f5fc4ff46b79231104e5808ef50e8532ac5592162a9e4a697878d', 'src/shell/cron.rs': '4e21cb9681ce13e5d201ee9534ae64916da9beeea8e5e348061356733063ec86', 'src/shell/execution.rs': '7305de1df1ac3f49c69d1ba87ac033e9e3624a9fd80f56fd0a96e71a8cd38e60', 'src/shell/getopts.rs': '9809755fabc557341a8ef0cb8a19f0958cbc2bceb38ecc2a14b7a40b9b99f7b2', 'src/shell/globbing.rs': 'd6cf355c640aa755b14fe46cd695a21bcc88a121c0286205ada14bfb50f6f3fe', 'src/shell/loop_control.rs': 'bbd90a32d7dc511787abbd59a78924eabed10b96d2ea4031c25c68a910c7a34d', 'src/shell/one_off.rs': '4d3b40f7228e608b20232c8599d03653ed19554829140d05daffdc2b4a1b3244', 'src/shell/parameters.rs': 'a7f0264d73e1c47f80734aa109d7907d247d9fabd1f39e81ba5f553c50764c97', 'src/shell/preparation.rs': '4e418b84bb95cdba1c03055a3e9ca84e7de2480840f6cf43a2205663a3aabed1', 'src/shell/substitution.rs': '74bf94e3ba7749a3ef7d0ea33de3613324f66ecaf6d9525f8035c69785306431', 'src/shell/text_tools.rs': 'deb064cd7ca459ffc0456adab6c91d8a0de6dd0574faaa809b75984830428540', 'src/shell/values.rs': '53f58bcf4759561a15995fcd7c565e6c137773a15b57de0d7c189b25c182ad60', 'src/shell/grep.rs': 'f0b554b0fcbcbdf99e3898e7402f48749be2b01e110398ec32a46f26e1fd7863', 'src/shell/literal_trim.rs': '9917307a858e06a7592811b49b466f9a5211a303cf915422a16520b83cea3f69'}
IDS = ('prefix-once', 'suffix-once', 'prefix-no-match', 'suffix-no-match', 'empty-prefix-word', 'empty-suffix-word', 'unset-value', 'empty-value', 'whole-removal-quoted-empty', 'positional-one-space-equals', 'positional-two', 'command-looking-data', 'literal-newline-data', 'utf8-prefix', 'utf8-suffix', 'bare-default-ifs', 'bare-empty-removal', 'bare-result-glob', 'quoted-result-no-glob', 'bare-result-unmatched-glob', 'function-positional-scope', 'local-name-scope', 'old-default-parser-first', 'argument-count-unchanged', 'arithmetic-borrowed-trim', 'prefix-output-path', 'suffix-output-path', 'function-status', 'stderr-and-status', 'literal-word-space', 'literal-word-hash-percent', 'source-value-not-mutated')
PROBE_SOURCE_SHA = '3f835faa39b31768b5f843e1b63651337932e87857f79070d4e245a9c73d5196'
HELPERS = ('tools/ere_reference.py', 'tools/case_bash_oracle.py',
           'tools/automation_before10_oracle.py', 'tools/build_safety.py', 'tools/dev.py', 'tools/case_classes_bash_oracle.py')
SOURCES = COMPILED + ('tests/shell_literal_trim.rs', 'tests/shell_literal_actual14.rs', 'docs/ERE_SUBSET.md',
    'docs/LITERAL_TRIM_REFERENCE.md', 'tools/literal_trim_bash_oracle.py',
    'tools/tests/test_literal_trim_bash_oracle.py', '.cargo/config.toml') + HELPERS

digest = shared.digest
canonical = shared.canonical

def git_child(base,name):
    current=base
    for part in name.split('/')[:-1]:
        current=current/part
        safe.require(stat.S_ISDIR(current.lstat().st_mode),'Ordinary Git ref parents only')
    return current/name.split('/')[-1]

def require_bash_version(result,executable):
    safe.require(result['status']==0 and result['stderr']=='' and type(result['stdout']) is str and result['stdout'].endswith('\n') and len(result['stdout'].encode())<=16384,'Held Bash version completed')
    safe.require(re.fullmatch(r'GNU bash, version [0-9]+(?:\.[0-9]+)+(?:\([0-9]+\))?[^\n]*',result['stdout'].split('\n')[0]) is not None,'Actual GNU Bash identity')

def safe_ref(name, *, requested=False):
    safe.require(isinstance(name, str) and len(name) <= 255
                 and re.fullmatch(r'refs/[A-Za-z0-9_./-]+', name)
                 and all(part and part not in ('.', '..') and not part.startswith('.')
                         and not part.endswith(('.', '.lock')) for part in name.split('/'))
                 and (not requested or name.startswith('refs/heads/')),
                 'Unsafe or unsupported symbolic source ref')
    return name


def git_line(raw):
    safe.require(raw and raw.isascii() and all(byte == 10 or 32 <= byte < 127 for byte in raw), 'Ordinary printable ASCII Git identity with LF only')
    text = raw.decode('ascii')
    if text.endswith('\n'): text = text[:-1]
    safe.require(text and '\n' not in text and '\r' not in text and text == text.strip(),
                 'Single canonical Git identity line')
    return text


def source_identity(root):
    """Local fixed-collector reader; old helpers and Git metadata stay immutable.

    Only an absent ordinary loose ref falls back to packed-refs. An alias,
    malformed loose value or open error must not become a silent fallback.
    """
    head = git_line(safe.read_bytes(root / '.git/HEAD', 4096))
    if re.fullmatch(r'[0-9a-f]{40}', head): return head
    safe.require(head.startswith('ref: '), 'Detached hash or symbolic HEAD required')
    requested = safe_ref(head[5:], requested=True)
    try:
        loose = safe.read_bytes(git_child(root / '.git', requested), 4096)
    except FileNotFoundError:
        pass
    else:
        value = git_line(loose)
        safe.require(re.fullmatch(r'[0-9a-f]{40}', value), 'Malformed ordinary loose source ref')
        return value  # Git loose-reference precedence; packed fallback is unused.
    raw = safe.read_bytes(root / '.git/packed-refs', 1024 * 1024)
    safe.require(raw.isascii() and all(byte == 10 or 32 <= byte < 127 for byte in raw),
                 'Packed refs allow printable ASCII and LF separators only')
    lines = raw.decode('ascii').split('\n')
    if lines and lines[-1] == '': lines.pop()  # One terminal LF is not an extra row.
    safe.require(len(lines) <= 4096, 'Packed-reference count bound')
    refs = {}; may_peel = False
    for line in lines:
        safe.require(len(line) <= 512, 'Packed-reference line bound')
        if not line or line.startswith('#'):
            may_peel = False
            continue
        if line.startswith('^'):
            safe.require(may_peel and re.fullmatch(r'\^[0-9a-f]{40}', line), 'Orphan, repeated or malformed peeled ref')
            may_peel = False
            continue
        match = re.fullmatch(r'([0-9a-f]{40}) (refs/[A-Za-z0-9_./-]+)', line)
        safe.require(match is not None, 'Malformed packed-reference record')
        value, name = match.groups(); safe_ref(name)
        safe.require(name not in refs, 'Duplicate packed-reference name')
        refs[name] = value; may_peel = True
    safe.require(requested in refs, 'HEAD source ref absent from bounded packed refs')
    return refs[requested]


PARENT14_SEALS = {'.build-artifacts/reviews/bash-literal-trim/captures/public14-193-477-r1/MANIFEST.json': '6046f5fb00aed71e341cf16c5a09f76d29f74a54b5f590f2740da7f24fa8ab57', '.build-artifacts/reviews/bash-literal-trim/captures/public14-193-477-r1/ATTESTATION.json': '9089741a7451c4557c3988d20d27e72cd124e8dca2b92dfc984f0f9099825a3a', '.build-artifacts/reviews/bash-literal-trim/captures/public14-193-477-r1/SOURCE.json': 'f461f2760b9474a8b667605a34562756934eb0fc32a4d49f40cf4380ff1ca6fc'}
SOURCES = SOURCES + tuple(PARENT14_SEALS) + ('tools/tests/fixtures/literal-trim-reference.json',)

def strict(raw):
    safe.require(len(raw) <= CAP, 'Bounded closed JSON')
    return shared.strict(raw)

def fixed(root):
    return fixed_bytes(safe.read_bytes(root / FIXTURE, CAP))


def fixed_bytes(raw):
    safe.require(digest(raw) == FIXTURE_SHA, 'Exact fixed32 utility recipes required')
    value = strict(raw)
    safe.require(set(value) == {'schema_version', 'kind', 'scope', 'environment',
        'host_execution', 'source_identity', 'cases', 'vm_only'}
        and type(value['schema_version']) is int and value['schema_version'] == 1
        and value['host_execution'] is None and value['source_identity'] is None,
        'Closed unobserved prediction fixture')
    safe.require(tuple(r['id'] for r in value['cases']) == IDS and len(value['vm_only']) == 8,
                 'Exactly32 ordered recipes and8 reason-only exclusions')
    for row in value['cases']:
        safe.require(row['observed'] is None and row['source']==row['vm_source'] and len(row['source'].encode())<=16384, 'No invented observations')
        files(row['files'])
    return value

def files(value):
    safe.require(type(value) is dict and len(value) <= 3, 'At most3 fixed regular files')
    for name, text in value.items():
        safe.require(type(name) is str and 0 < len(name.encode()) <= 64
            and '/' not in name and name not in ('.', '..')
            and all(32 <= ord(c) < 127 for c in name), 'Fixed contained leaf file only')
        safe.require(type(text) is str and '\0' not in text and len(text.encode()) <= 4096,
                     'Small fixed UTF8 file data')
    return value

def host_argv(executable, row, rows):
    safe.require(type(executable) is str and re.fullmatch(r'/proc/self/fd/[0-9]+',executable), 'Held Bash descriptor only')
    safe.require(row in rows and row['id'] in IDS and canonical(row)==canonical(rows[IDS.index(row['id'])]), 'Exact closed immutable script and argv data')
    safe.require(row['source']==row['vm_source'] and type(row['args']) is list and len(row['args'])<=4 and all(type(a) is str and '\0' not in a and len(a.encode())<=4096 for a in row['args']), 'Bounded argv data; never eval')
    return [executable,'--noprofile','--norc','-c',row['source'],'--',*row['args']]

def project_roster(root):
    expected = {name for name in COMPILED if Path(name).parts[0] in SOURCE_FAMILIES}
    directories = set(SOURCE_FAMILIES)
    for name in expected:
        parent = Path(name).parent
        while parent != Path('.'):
            directories.add(parent.as_posix())
            parent = parent.parent
    found = set()
    examined = 0
    for family in SOURCE_FAMILIES:
        base = root / family
        safe.require(stat.S_ISDIR(base.lstat().st_mode), 'Ordinary fixed source family only')
        examined += 1
        for path in base.rglob('*'):
            examined += 1
            safe.require(examined <= len(expected) + len(directories), 'Bounded closed source inventory')
            name = path.relative_to(root).as_posix()
            info = path.lstat()
            if stat.S_ISDIR(info.st_mode):
                safe.require(name in directories, 'No unreviewed project source directory')
            else:
                safe.require(stat.S_ISREG(info.st_mode) and info.st_nlink == 1 and name in expected,
                             'No omitted/extra/alias project source')
                found.add(name)
    safe.require(found == expected, 'Every actual library/project input must be listed')
    return found


def source_pins(root):
    project_roster(root)
    for name,sha in PARENT14_SEALS.items():
        safe.require(safe.pin(root/name,CAP)["sha256"]==sha,"Sealed genuine14 parent changed")
    for name, sha in CORE_PINS.items():
        safe.require(safe.pin(root / name, CAP)['sha256'] == sha,
                     'Reviewed evaluator15 source changed: ' + name)
    safe.require(safe.pin(root / PROBE_SOURCE, CAP)['sha256'] == PROBE_SOURCE_SHA,
                 'Reviewed fixed probe changed')
    fixed(root)
    chain = root / '.build-artifacts/toolchains/rustup/toolchains/stable-x86_64-unknown-linux-gnu'
    paths = [(root / name, CAP) for name in SOURCES] + [
        (chain / 'bin/rustc', 128 * CAP),
        (chain / 'lib/rustlib/multirust-channel-manifest.toml', CAP),
        (root / '.build-artifacts/toolchains/READY.json', CAP),
        (Path(sys.executable).resolve(strict=True), 64 * CAP), (root / PROBE, BINARY_CAP),
        (BASH, 8 * CAP), (MANUAL, CAP)]
    pins = [safe.pin(path, cap) for path, cap in paths]
    safe.require(len(pins) == len({p['path'] for p in pins}), 'No omitted/duplicate source pins')
    return pins

def control(root, pins):
    return {'schema_version': 1, 'capture': CAPTURE, 'root': str(root), 'root_reviewed': False,
        'main_head': source_identity(root), 'pins': pins,
        'compiled_sources': {name: next(p['sha256'] for p in pins if p['path'] == str(root / name))
                             for name in COMPILED},
        'fixture_sha256': FIXTURE_SHA, 'host_rows': 32, 'GNU_exclusion_reasons': 8,
        'scope': 'Fixed real VM15 literal trim only; not full Bash/GNU, curriculum or native acceptance',
        'compiler_scope': 'Cached rustc/channel/READY, Cargo.lock and Python executable; not complete toolchain/stdlib reproducibility'}


def check_control(root, expected):
    keys = {'schema_version', 'capture', 'root', 'root_reviewed', 'main_head', 'pins',
            'compiled_sources', 'fixture_sha256', 'host_rows', 'GNU_exclusion_reasons',
            'scope', 'compiler_scope'}
    safe.require(type(expected) is dict and set(expected) == keys
        and type(expected['schema_version']) is int and expected['schema_version'] == 1
        and expected['capture'] == CAPTURE and expected['root'] == str(root)
        and expected['root_reviewed'] is False and expected['fixture_sha256'] == FIXTURE_SHA
        and type(expected['host_rows']) is int and expected['host_rows'] == 32
        and type(expected['GNU_exclusion_reasons']) is int and expected['GNU_exclusion_reasons'] == 8
        and type(expected['main_head']) is str and re.fullmatch(r'[0-9a-f]{40}', expected['main_head']), 'Fixed retained control scope')
    chain = root / '.build-artifacts/toolchains/rustup/toolchains/stable-x86_64-unknown-linux-gnu'
    paths = {str(root / n) for n in SOURCES} | {str(chain / 'bin/rustc'),
        str(chain / 'lib/rustlib/multirust-channel-manifest.toml'),
        str(root / '.build-artifacts/toolchains/READY.json'),
        str(Path(sys.executable).resolve(strict=True)), str(root / PROBE),
        str(BASH), str(MANUAL)}
    pins = expected['pins']
    safe.require(type(pins) is list and len(pins) == len(paths), 'Complete closed pin inventory')
    for pin in pins:
        safe.require(type(pin) is dict and set(pin) == {'path', 'bytes', 'sha256'}
            and type(pin['path']) is str and type(pin['bytes']) is int and 0 <= pin['bytes'] <= BINARY_CAP
            and type(pin['sha256']) is str and re.fullmatch(r'[0-9a-f]{64}', pin['sha256']),
            'Closed typed bounded source/compiler/executable pin')
    table = {pin['path']: pin for pin in pins}
    safe.require(set(table) == paths and len(table) == len(pins), 'No missing/extra/duplicate pin')
    for name,sha in PARENT14_SEALS.items():
        safe.require(table[str(root/name)]['sha256']==sha,'Exact genuine14 parent seals in retained control')
    safe.require(expected['compiled_sources'] == {n: table[str(root / n)]['sha256'] for n in COMPILED},
                 'Exact direct compiled-source correspondence')
    for name, sha in {**CORE_PINS, PROBE_SOURCE: PROBE_SOURCE_SHA, FIXTURE: FIXTURE_SHA}.items():
        safe.require(table[str(root / name)]['sha256'] == sha, 'Reviewed fixed source pin')
    return table


def identity(root, expected):
    return {'kind': 'fixed-literal32-reference-r1', 'manifest_dir': str(root),
        'engine_revision': 15, 'fixture_sha256': FIXTURE_SHA, 'host_cases': 32,
        'GNU_exclusion_reasons': 8, 'compiled_sources': expected['compiled_sources']}

def parse_probe(raw, root, fixture, expected):
    value = strict(raw)
    safe.require(set(value) == {'identity', 'outcomes', 'GNU_exclusions'}
        and canonical(value['identity']) == canonical(identity(root, expected))
        and canonical(value['GNU_exclusions']) == canonical(fixture['vm_only']),
        'Actual compiled probe identity and exact reason-only exclusions')
    outcomes = value['outcomes']
    safe.require(type(outcomes) is list and len(outcomes) == 32, 'All32 actual utility results')
    for row, item in zip(fixture['cases'], outcomes):
        safe.require(set(item) == {'id', 'row_sha256', 'outcome'} and item['id'] == row['id']
            and item['row_sha256'] == digest(canonical(row)), 'Ordered fixed row/input identity')
        outcome = item['outcome']
        safe.require(type(outcome) is dict and set(outcome) == {'stdout', 'stderr', 'status', 'files', 'error'}
            and type(outcome['status']) is int and -128 <= outcome['status'] <= 255
            and all(type(outcome[k]) is str and len(outcome[k].encode()) <= 16384
                    for k in ('stdout', 'stderr'))
            and (outcome['error'] is None or (type(outcome['error']) is str and len(outcome['error']) <= 4096)),
            'Closed typed bounded actual VM outcome')
        safe.require(type(outcome['files']) is dict, 'Typed virtual file map')
        normalized = {}
        for path, text in outcome['files'].items():
            safe.require(type(path) is str and path.startswith('/work/'), 'Virtual files only')
            normalized[path[6:]] = text
        files(normalized)
        safe.require(len(normalized) == len(outcome['files']), 'Unique normalized virtual files')
    return outcomes

def snapshot(work, expected):
    files(expected)
    safe.require({p.name for p in work.iterdir()} == set(expected), 'Exact disposable file set')
    actual = {}
    for name in expected:
        path = work / name
        with safe.ordinary(path, 4096) as (stream, info):
            safe.require(info.st_nlink == 1, 'Reference files must remain independent ordinary leaves')
            raw = stream.read(info.st_size)
        actual[name] = raw.decode('utf-8', errors='strict')
    return actual

def result(folder, budget, observed):
    safe.json_new(folder / 'RESULT.json', observed, budget, CAP)
    safe.json_new(folder / 'STATUS.json', {'complete': True, 'status': observed['status']}, budget, CAP)

def child(out, name, command, data, inputs, budget, inherited, start, *, vm=False):
    safe.require(time.monotonic() - start < DEADLINE, 'Whole fixed reference deadline')
    folder = safe.private_dir(out, name)
    work = safe.private_dir(folder, 'work')
    empty_bin = safe.private_dir(folder, 'bin')
    for filename, text in files(inputs).items():
        safe.write_new(work / filename, text.encode(), budget, 4096)
    safe.require(snapshot(work, inputs) == inputs, 'Exact independent fixture materialization')
    try:
        observed = safe.capture_command(command, folder, work, {**safe.closed_environment({},work),'PATH':str(empty_bin)}, data,
            budget, 512 * 1024 if vm else 16 * 1024, timeout=15 if vm else 2, inherited=inherited,
            popen=lambda command, **kw: plumbing.bounded_popen(command,vm=vm,**kw))
    except Exception as error:
        safe.json_new(folder / 'STATUS.json', {'complete': False, 'status': None,
                      'error': str(error)[:1000]}, budget, CAP)
        raise
    result(folder, budget, observed)
    return observed, snapshot(work, inputs)

def compared(row, vm, gnu, after):
    normalized = {path[6:]: text for path, text in vm['files'].items()}
    observed = {'stdout': vm['stdout'], 'stderr': vm['stderr'], 'status': vm['status'], 'files': normalized}
    host = {**gnu, 'files': after}
    return {'id': row['id'], 'VM': observed, 'VM_error': vm['error'], 'GNU': host,
        'expected': row['expected'], 'passed': vm['error'] is None
        and canonical(observed) == canonical(host) == canonical(row['expected'])}

def source_retention(root, out, budget):
    directory = safe.private_dir(out, 'source')
    for i, name in enumerate(SOURCES):
        safe.write_new(directory / f'file{i}.raw', safe.read_bytes(root / name, CAP), budget, CAP)
    safe.json_new(directory / 'INDEX.json', list(SOURCES), budget, CAP)

def output_paths(fixture):
    paths = ['CONTROL.json', 'probe.elf', 'bash.elf', 'source/INDEX.json']
    paths += [f'source/file{i}.raw' for i in range(len(SOURCES))]
    for name in ['probe-identity', 'probe-outcomes', 'bash-version'] + [f'case{i}' for i in range(32)]:
        paths += [f'{name}/{f}' for f in ['stdout.raw', 'stderr.raw', 'RESULT.json', 'STATUS.json']]
    for i, row in enumerate(fixture['cases']):
        paths += [f'case{i}/work/{name}' for name in row['files']]
        paths.append(f'row{i}.json')
    return paths

def evidence_pin(out, path):
    pin = safe.pin(out / path, BINARY_CAP if path.endswith('.elf') else CAP)
    return {'path': path, 'bytes': pin['bytes'], 'sha256': pin['sha256']}


def artifact_inventory(out, expected):
    allowed_dirs = {str(Path(name).parent) for name in expected if '/' in name}
    allowed_dirs |= {str(Path(name).parent.parent) for name in expected if name.count('/') > 1}
    allowed_dirs |= {name + '/bin' for name in ['probe-identity','probe-outcomes','bash-version']+[f'case{i}' for i in range(32)]}
    allowed_dirs |= {name + '/work' for name in ['probe-identity', 'probe-outcomes', 'bash-version'] + [f'case{i}' for i in range(32)]}
    pending = [(out, 0)]
    names = set()
    examined = 0
    while pending:
        directory, depth = pending.pop()
        for path in directory.iterdir():
            examined += 1
            safe.require(examined <= len(expected) + len(allowed_dirs), 'Exact closed retained evidence entry bound')
            info = path.lstat()
            if stat.S_ISDIR(info.st_mode):
                safe.require(depth < 3 and str(path.relative_to(out)) in allowed_dirs, 'Fixed evidence directories only')
                pending.append((path, depth + 1))
            else:
                safe.require(stat.S_ISREG(info.st_mode), 'No symlinks/devices in retained evidence')
                names.add(str(path.relative_to(out)))
    safe.require(names == expected, 'No missing/excess retained evidence files')


def retained_result(out, name):
    folder = out / name
    item = strict(safe.read_bytes(folder / 'RESULT.json', CAP))
    safe.require(type(item) is dict and set(item) == {'stdout', 'stderr', 'status'}
        and type(item['status']) is int and type(item['stdout']) is str and type(item['stderr']) is str,
        'Exact typed retained child RESULT')
    for key in ('stdout', 'stderr'):
        safe.require(safe.read_bytes(folder / (key + '.raw'), CAP) == item[key].encode(),
                     'Raw stream and RESULT disagree')
    status = strict(safe.read_bytes(folder / 'STATUS.json', CAP))
    safe.require(type(status) is dict and set(status) == {'complete', 'status'}
        and status['complete'] is True and type(status['status']) is int
        and status['status'] == item['status'], 'Actual child complete/status disagreement')
    return item

def verify(root):
    shared.root_guard(root)
    out = root / BASE / CAPTURE
    fixture = fixed_bytes(safe.read_bytes(
        out / 'source' / f'file{SOURCES.index(FIXTURE)}.raw', CAP))
    manifest_raw = safe.read_bytes(out / 'MANIFEST.json', CAP)
    safe.require(safe.read_bytes(out / 'COMPLETE', 65) == (digest(manifest_raw) + '\n').encode(),
                 'Exact committed manifest marker')
    safe.require(safe.read_bytes(out / 'COMPLETION-PREPARED', 65) == safe.read_bytes(out / 'COMPLETE', 65), 'Prepared/committed marker agreement')
    manifest = strict(manifest_raw)
    safe.require(set(manifest) == {'schema_version', 'control', 'GNU_descriptor', 'accepted', 'comparisons', 'GNU_exclusions', 'outputs', 'claims', 'GNU_exclusions_executed', 'limits'}
        and type(manifest['schema_version']) is int and manifest['schema_version'] == 1
        and manifest['GNU_exclusions_executed'] is False, 'Closed fixed utility manifest, not invented exclusion results')
    expected = manifest['control']
    pin_table = check_control(root, expected)
    approved = safe.read_bytes(root / BASE / (CAPTURE + '-APPROVED.json'), CAP)
    shared.approve(approved, expected)
    safe.require(safe.read_bytes(out / 'CONTROL.json', CAP) == approved, 'Literal Root approval retained')
    pins = manifest['outputs']
    safe.require(type(pins) is list and [x['path'] for x in pins] == output_paths(fixture),
                 'Complete fixed evidence inventory and order')
    safe.require(all(type(x['bytes']) is int and type(x['sha256']) is str
        and evidence_pin(out, x['path']) == x for x in pins), 'Retained output bytes/hash mismatch')
    artifact_inventory(out, set(output_paths(fixture)) | {'MANIFEST.json', 'COMPLETE', 'COMPLETION-PREPARED'})
    sources = strict(safe.read_bytes(out / 'source/INDEX.json', CAP))
    safe.require(sources == list(SOURCES), 'Every imported helper and compiled source retained')
    for i, name in enumerate(SOURCES):
        raw = safe.read_bytes(out / f'source/file{i}.raw', CAP)
        pin = pin_table[str(root / name)]
        safe.require(len(raw) == pin['bytes'] and digest(raw) == pin['sha256'], 'Retained approved source identity')
    for name, original in [('probe.elf', str(root / PROBE)), ('bash.elf', str(BASH))]:
        actual = safe.pin(out / name, BINARY_CAP)
        safe.require((actual['bytes'], actual['sha256']) ==
                     (pin_table[original]['bytes'], pin_table[original]['sha256']), 'Retained held ELF identity')
    for name in ('probe-identity', 'probe-outcomes', 'bash-version'):
        snapshot(out / name / 'work', {})
        safe.require(not any((out/name/'bin').iterdir()),'Private PATH remains empty')
    ident = retained_result(out, 'probe-identity')
    safe.require(ident['status'] == 0 and not ident['stderr']
        and canonical(strict(ident['stdout'])) == canonical(identity(root, expected)), 'Actual probe identity')
    probe = retained_result(out, 'probe-outcomes')
    safe.require(probe['status'] == 0 and not probe['stderr'], 'Actual probe completed successfully')
    outcomes = parse_probe(probe['stdout'], root, fixture, expected)
    version = retained_result(out, 'bash-version')
    require_bash_version(version, manifest['GNU_descriptor'])
    rows = []
    for i, (row, observed) in enumerate(zip(fixture['cases'], outcomes)):
        host = retained_result(out, f'case{i}')
        after = snapshot(out / f'case{i}/work', row['files'])
        safe.require(not any((out/f'case{i}'/'bin').iterdir()),'Private PATH remains empty')
        comparison = compared(row, observed['outcome'], host, after)
        safe.require(strict(safe.read_bytes(out / f'row{i}.json', CAP)) == comparison,
                     'Recomputed full comparison differs from retained row')
        rows.append(comparison)
    safe.require(manifest['claims'] == expected['scope'], 'Honest retained scope')
    safe.require(manifest['comparisons'] == rows and manifest['GNU_exclusions'] == fixture['vm_only']
        and manifest['accepted'] is True and all(row['passed'] for row in rows), 'No self-asserted match accepted')
    return {'status': 'VERIFIED32-FIXED-UTILITY-COMPARISONS', 'GNU_exclusion_reasons': 8,
            'manifest_sha256': digest(manifest_raw)}

def run(root, mode):
    if mode == 'verify':
        print(json.dumps(verify(root)))
        return 0
    inherited = shared.root_guard(root)
    fixture = fixed(root)
    pins = source_pins(root)
    expected = control(root, pins)
    check_control(root, expected)
    base = shared.owned_directory(root, root / BASE)
    safe.require(not (base / CAPTURE).exists() and not (base / CAPTURE).is_symlink(), 'Fresh capture only')
    if mode == 'prepare':
        safe.json_new(base / (CAPTURE + '-PROPOSED.json'), expected, safe.Budget(CAP), CAP)
        print(json.dumps({'status': 'PROPOSED-NO-CHILD-EXECUTION', 'pins': len(pins), 'GNU_rows': 32}))
        return 0
    safe.require(mode == 'capture', 'Fixed capture mode')
    approved = safe.read_bytes(base / (CAPTURE + '-APPROVED.json'), CAP)
    shared.approve(approved, expected)
    out = safe.private_dir(base, CAPTURE)
    budget = safe.Budget(EVIDENCE_CAP)
    start = time.monotonic()
    try:
        safe.write_new(out / 'CONTROL.json', approved, budget, CAP)
        table = {p['path']: p for p in pins}
        with shared.pinned_executable(root / PROBE, table[str(root / PROBE)], BINARY_CAP) as (vm, vfd), \
             shared.pinned_executable(BASH, table[str(BASH)], 8 * CAP) as (gnu, gfd):
            inherited = (*inherited, vfd, gfd)
            ident, _ = child(out, 'probe-identity', [vm, '--identity'], b'', {}, budget, inherited, start, vm=True)
            safe.require(ident['status'] == 0 and not ident['stderr']
                and canonical(strict(ident['stdout'])) == canonical(identity(root, expected)), 'Compiled fixed probe identity')
            probe, _ = child(out, 'probe-outcomes', [vm], b'', {}, budget, inherited, start, vm=True)
            safe.require(probe['status'] == 0 and not probe['stderr'], 'Fixed VM probe failed')
            outcomes = parse_probe(probe['stdout'], root, fixture, expected)
            version, _ = child(out, 'bash-version', [gnu, '--version'], b'', {}, budget, inherited, start)
            require_bash_version(version, gnu)
            rows = []
            for i, (row, observation) in enumerate(zip(fixture['cases'], outcomes)):
                host, after = child(out, f'case{i}', host_argv(gnu, row, fixture['cases']),
                    row['stdin'].encode(), row['files'], budget, inherited, start)
                comparison = compared(row, observation['outcome'], host, after)
                safe.json_new(out / f'row{i}.json', comparison, budget, CAP)
                rows.append(comparison)
            shared.retain_executable(vfd, table[str(root / PROBE)], out, 'probe.elf', budget)
            shared.retain_executable(gfd, table[str(BASH)], out, 'bash.elf', budget)
        safe.require(time.monotonic() - start < DEADLINE, 'Whole utility deadline')
        safe.require(source_pins(root) == pins and source_identity(root) == expected['main_head'],
                     'Source/compiler/executable/HEAD changed during capture')
        source_retention(root, out, budget)
        safe.require(source_pins(root) == pins
            and safe.read_bytes(base / (CAPTURE + '-APPROVED.json'), CAP) == approved, 'Source/approval drift during retention')
        accepted = all(row['passed'] for row in rows)
        manifest = {'schema_version': 1, 'control': expected, 'GNU_descriptor': gnu,
            'accepted': accepted, 'comparisons': rows, 'GNU_exclusions': fixture['vm_only'],
            'outputs': [evidence_pin(out, path) for path in output_paths(fixture)],
            'claims': expected['scope'], 'GNU_exclusions_executed': False,
            'limits': {'GNU_seconds': 2, 'probe_seconds': 15, 'whole_seconds': DEADLINE,
                       'stream_bytes': [16384, 524288], 'evidence_bytes': EVIDENCE_CAP}}
        published, warning = safe.publish(out, manifest, budget)
    except Exception as error:
        safe.json_new(out / 'FAILED.json', {'accepted': False, 'error': str(error)[:2000]}, budget, CAP)
        raise
    print(json.dumps({'status': 'MATCHED32' if accepted else 'COMPLETED-WITH-MISMATCHES',
                      'manifest_sha256': published['sha256'], 'durability_warning': warning}))
    return 0 if accepted else 1

def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--mode', required=True, choices=['prepare', 'capture', 'verify'])
    args = parser.parse_args(argv)
    return run(ROOT.resolve(strict=True), args.mode)

if __name__ == '__main__':
    raise SystemExit(main())
