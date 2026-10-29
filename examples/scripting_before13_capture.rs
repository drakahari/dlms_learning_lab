//! Genuine evaluator12 development fixture producer, never learner input.
//! Feature-gated before evaluator13; original9/10/11 producers/evidence stay intact.
use dlms_learning_lab::{
    engine::{Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching,
    shell::{self, Record},
    workspace,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
#[path = "../tests/support/actual11_evidence.rs"]
mod actual11;
const INDEX: &[u8] = include_bytes!("../tests/fixtures/bash-before13-source-index.json");
const INDEX_SHA: &str = "69efc842123cb50df55c29141f40441f804a3fe1a7e6ac80a1aaa6f9aef3fcbc";
const DESTINATION: &str =
    ".build-artifacts/reviews/bash-loop-recovery/captures/public12-171-420-r1";
const NEW: [&str; 3] = [
    "shell-numeric-retry",
    "shell-numeric-intake",
    "shell-numeric-publication",
];
const DRAFTS: [&str; 2] = ["shell-command-plan", "shell-handover"];
const COMPILED_CORE: &[(&str, &[u8])] = &[
    (
        ".cargo/config.toml",
        include_bytes!("../.cargo/config.toml"),
    ),
    ("Cargo.lock", include_bytes!("../Cargo.lock")),
    ("Cargo.toml", include_bytes!("../Cargo.toml")),
    (
        "docs/BUILD_WORKFLOW.md",
        include_bytes!("../docs/BUILD_WORKFLOW.md"),
    ),
    (
        "examples/automation_before10_capture.rs",
        include_bytes!("../examples/automation_before10_capture.rs"),
    ),
    (
        "examples/parameters_before11_capture.rs",
        include_bytes!("../examples/parameters_before11_capture.rs"),
    ),
    (
        "examples/scripting_before12_capture.rs",
        include_bytes!("../examples/scripting_before12_capture.rs"),
    ),
    (
        "examples/scripting_before13_capture.rs",
        include_bytes!("../examples/scripting_before13_capture.rs"),
    ),
    (
        "examples/support/automation_digest.rs",
        include_bytes!("../examples/support/automation_digest.rs"),
    ),
    (
        "scenarios/accounts-lifecycle-v1.json",
        include_bytes!("../scenarios/accounts-lifecycle-v1.json"),
    ),
    (
        "scenarios/acl-policy-v1.json",
        include_bytes!("../scenarios/acl-policy-v1.json"),
    ),
    (
        "scenarios/apache-static-v1.json",
        include_bytes!("../scenarios/apache-static-v1.json"),
    ),
    (
        "scenarios/debian-packages-v1.json",
        include_bytes!("../scenarios/debian-packages-v1.json"),
    ),
    (
        "scenarios/dhcpv4-client-v1.json",
        include_bytes!("../scenarios/dhcpv4-client-v1.json"),
    ),
    (
        "scenarios/discovery-v1.json",
        include_bytes!("../scenarios/discovery-v1.json"),
    ),
    (
        "scenarios/dns-forward-v1.json",
        include_bytes!("../scenarios/dns-forward-v1.json"),
    ),
    (
        "scenarios/dns-reverse-v1.json",
        include_bytes!("../scenarios/dns-reverse-v1.json"),
    ),
    (
        "scenarios/etherchannel-v1.json",
        include_bytes!("../scenarios/etherchannel-v1.json"),
    ),
    (
        "scenarios/file-backup-v1.json",
        include_bytes!("../scenarios/file-backup-v1.json"),
    ),
    (
        "scenarios/ipv4-static-v1.json",
        include_bytes!("../scenarios/ipv4-static-v1.json"),
    ),
    (
        "scenarios/ipv6-static-v1.json",
        include_bytes!("../scenarios/ipv6-static-v1.json"),
    ),
    (
        "scenarios/linux-access-v1.json",
        include_bytes!("../scenarios/linux-access-v1.json"),
    ),
    (
        "scenarios/linux-cron-v1.json",
        include_bytes!("../scenarios/linux-cron-v1.json"),
    ),
    (
        "scenarios/linux-files-v1.json",
        include_bytes!("../scenarios/linux-files-v1.json"),
    ),
    (
        "scenarios/linux-find-v1.json",
        include_bytes!("../scenarios/linux-find-v1.json"),
    ),
    (
        "scenarios/linux-processes-v1.json",
        include_bytes!("../scenarios/linux-processes-v1.json"),
    ),
    (
        "scenarios/mac-learning-v1.json",
        include_bytes!("../scenarios/mac-learning-v1.json"),
    ),
    (
        "scenarios/nat-basic-v1.json",
        include_bytes!("../scenarios/nat-basic-v1.json"),
    ),
    (
        "scenarios/nfs-v1.json",
        include_bytes!("../scenarios/nfs-v1.json"),
    ),
    (
        "scenarios/ospf-p2p-v1.json",
        include_bytes!("../scenarios/ospf-p2p-v1.json"),
    ),
    (
        "scenarios/rpm-packages-v1.json",
        include_bytes!("../scenarios/rpm-packages-v1.json"),
    ),
    (
        "scenarios/shell-arrays-v1.json",
        include_bytes!("../scenarios/shell-arrays-v1.json"),
    ),
    (
        "scenarios/shell-capture-v1.json",
        include_bytes!("../scenarios/shell-capture-v1.json"),
    ),
    (
        "scenarios/shell-case-v1.json",
        include_bytes!("../scenarios/shell-case-v1.json"),
    ),
    (
        "scenarios/shell-command-vectors-v1.json",
        include_bytes!("../scenarios/shell-command-vectors-v1.json"),
    ),
    (
        "scenarios/shell-composition-v1.json",
        include_bytes!("../scenarios/shell-composition-v1.json"),
    ),
    (
        "scenarios/shell-counts-v1.json",
        include_bytes!("../scenarios/shell-counts-v1.json"),
    ),
    (
        "scenarios/shell-file-batches-v1.json",
        include_bytes!("../scenarios/shell-file-batches-v1.json"),
    ),
    (
        "scenarios/shell-getopts-v1.json",
        include_bytes!("../scenarios/shell-getopts-v1.json"),
    ),
    (
        "scenarios/shell-numeric-validation-v1.json",
        include_bytes!("../scenarios/shell-numeric-validation-v1.json"),
    ),
    (
        "scenarios/shell-optional-inputs-v1.json",
        include_bytes!("../scenarios/shell-optional-inputs-v1.json"),
    ),
    (
        "scenarios/shell-reusable-v1.json",
        include_bytes!("../scenarios/shell-reusable-v1.json"),
    ),
    (
        "scenarios/shell-safe-automation-v1.json",
        include_bytes!("../scenarios/shell-safe-automation-v1.json"),
    ),
    (
        "scenarios/shell-scope-v1.json",
        include_bytes!("../scenarios/shell-scope-v1.json"),
    ),
    (
        "scenarios/shell-text-v1.json",
        include_bytes!("../scenarios/shell-text-v1.json"),
    ),
    (
        "scenarios/shell-unit-v1.json",
        include_bytes!("../scenarios/shell-unit-v1.json"),
    ),
    (
        "scenarios/spanning-tree-v1.json",
        include_bytes!("../scenarios/spanning-tree-v1.json"),
    ),
    (
        "scenarios/starter-v1.json",
        include_bytes!("../scenarios/starter-v1.json"),
    ),
    (
        "scenarios/storage-v1.json",
        include_bytes!("../scenarios/storage-v1.json"),
    ),
    (
        "scenarios/unit-manager-v1.json",
        include_bytes!("../scenarios/unit-manager-v1.json"),
    ),
    (
        "scenarios/units-v1.json",
        include_bytes!("../scenarios/units-v1.json"),
    ),
    (
        "scenarios/vlan-data-v1.json",
        include_bytes!("../scenarios/vlan-data-v1.json"),
    ),
    ("src/accounts.rs", include_bytes!("../src/accounts.rs")),
    (
        "src/accounts_teaching.rs",
        include_bytes!("../src/accounts_teaching.rs"),
    ),
    ("src/acl.rs", include_bytes!("../src/acl.rs")),
    ("src/acl_editor.rs", include_bytes!("../src/acl_editor.rs")),
    (
        "src/acl_routing.rs",
        include_bytes!("../src/acl_routing.rs"),
    ),
    (
        "src/acl_teaching.rs",
        include_bytes!("../src/acl_teaching.rs"),
    ),
    (
        "src/acl_workspace.rs",
        include_bytes!("../src/acl_workspace.rs"),
    ),
    (
        "src/address_teaching.rs",
        include_bytes!("../src/address_teaching.rs"),
    ),
    (
        "src/apache_static.rs",
        include_bytes!("../src/apache_static.rs"),
    ),
    (
        "src/apache_static_teaching.rs",
        include_bytes!("../src/apache_static_teaching.rs"),
    ),
    (
        "src/apache_static_workspace.rs",
        include_bytes!("../src/apache_static_workspace.rs"),
    ),
    ("src/backup.rs", include_bytes!("../src/backup.rs")),
    ("src/catalog.rs", include_bytes!("../src/catalog.rs")),
    (
        "src/client_teaching.rs",
        include_bytes!("../src/client_teaching.rs"),
    ),
    ("src/course.rs", include_bytes!("../src/course.rs")),
    ("src/cron.rs", include_bytes!("../src/cron.rs")),
    (
        "src/cron_teaching.rs",
        include_bytes!("../src/cron_teaching.rs"),
    ),
    (
        "src/cron_workspace.rs",
        include_bytes!("../src/cron_workspace.rs"),
    ),
    (
        "src/cron_workspace/budget.rs",
        include_bytes!("../src/cron_workspace/budget.rs"),
    ),
    (
        "src/debian_packages.rs",
        include_bytes!("../src/debian_packages.rs"),
    ),
    (
        "src/debian_packages_teaching.rs",
        include_bytes!("../src/debian_packages_teaching.rs"),
    ),
    (
        "src/debian_packages_terminal.rs",
        include_bytes!("../src/debian_packages_terminal.rs"),
    ),
    (
        "src/debian_packages_terminal/session.rs",
        include_bytes!("../src/debian_packages_terminal/session.rs"),
    ),
    (
        "src/debian_packages_workspace.rs",
        include_bytes!("../src/debian_packages_workspace.rs"),
    ),
    ("src/dhcpv4.rs", include_bytes!("../src/dhcpv4.rs")),
    (
        "src/dhcpv4_teaching.rs",
        include_bytes!("../src/dhcpv4_teaching.rs"),
    ),
    (
        "src/dhcpv4_workspace.rs",
        include_bytes!("../src/dhcpv4_workspace.rs"),
    ),
    ("src/discovery.rs", include_bytes!("../src/discovery.rs")),
    (
        "src/discovery_teaching.rs",
        include_bytes!("../src/discovery_teaching.rs"),
    ),
    (
        "src/discovery_workspace.rs",
        include_bytes!("../src/discovery_workspace.rs"),
    ),
    ("src/dns.rs", include_bytes!("../src/dns.rs")),
    (
        "src/dns_teaching.rs",
        include_bytes!("../src/dns_teaching.rs"),
    ),
    (
        "src/dns_workspace.rs",
        include_bytes!("../src/dns_workspace.rs"),
    ),
    ("src/engine.rs", include_bytes!("../src/engine.rs")),
    (
        "src/etherchannel.rs",
        include_bytes!("../src/etherchannel.rs"),
    ),
    (
        "src/etherchannel_teaching.rs",
        include_bytes!("../src/etherchannel_teaching.rs"),
    ),
    (
        "src/etherchannel_workspace.rs",
        include_bytes!("../src/etherchannel_workspace.rs"),
    ),
    (
        "src/file_backup.rs",
        include_bytes!("../src/file_backup.rs"),
    ),
    (
        "src/file_backup_teaching.rs",
        include_bytes!("../src/file_backup_teaching.rs"),
    ),
    (
        "src/file_backup_terminal.rs",
        include_bytes!("../src/file_backup_terminal.rs"),
    ),
    (
        "src/file_backup_workspace.rs",
        include_bytes!("../src/file_backup_workspace.rs"),
    ),
    (
        "src/file_backup_workspace/presentation.rs",
        include_bytes!("../src/file_backup_workspace/presentation.rs"),
    ),
    ("src/filesystem.rs", include_bytes!("../src/filesystem.rs")),
    (
        "src/filesystem/find.rs",
        include_bytes!("../src/filesystem/find.rs"),
    ),
    (
        "src/filesystem/permissions.rs",
        include_bytes!("../src/filesystem/permissions.rs"),
    ),
    ("src/ipv4.rs", include_bytes!("../src/ipv4.rs")),
    (
        "src/ipv4_routing.rs",
        include_bytes!("../src/ipv4_routing.rs"),
    ),
    (
        "src/ipv4_routing_teaching.rs",
        include_bytes!("../src/ipv4_routing_teaching.rs"),
    ),
    (
        "src/ipv4_routing_workspace.rs",
        include_bytes!("../src/ipv4_routing_workspace.rs"),
    ),
    ("src/ipv6.rs", include_bytes!("../src/ipv6.rs")),
    (
        "src/ipv6_teaching.rs",
        include_bytes!("../src/ipv6_teaching.rs"),
    ),
    (
        "src/ipv6_workspace.rs",
        include_bytes!("../src/ipv6_workspace.rs"),
    ),
    (
        "src/legacy_access_teaching.rs",
        include_bytes!("../src/legacy_access_teaching.rs"),
    ),
    (
        "src/legacy_logs_teaching.rs",
        include_bytes!("../src/legacy_logs_teaching.rs"),
    ),
    ("src/lib.rs", include_bytes!("../src/lib.rs")),
    ("src/local_file.rs", include_bytes!("../src/local_file.rs")),
    (
        "src/mac_learning.rs",
        include_bytes!("../src/mac_learning.rs"),
    ),
    (
        "src/mac_learning_teaching.rs",
        include_bytes!("../src/mac_learning_teaching.rs"),
    ),
    (
        "src/mac_learning_workspace.rs",
        include_bytes!("../src/mac_learning_workspace.rs"),
    ),
    ("src/main.rs", include_bytes!("../src/main.rs")),
    ("src/nat.rs", include_bytes!("../src/nat.rs")),
    (
        "src/nat_teaching.rs",
        include_bytes!("../src/nat_teaching.rs"),
    ),
    (
        "src/nat_workspace.rs",
        include_bytes!("../src/nat_workspace.rs"),
    ),
    ("src/nfs.rs", include_bytes!("../src/nfs.rs")),
    (
        "src/nfs_teaching.rs",
        include_bytes!("../src/nfs_teaching.rs"),
    ),
    (
        "src/nfs_workspace.rs",
        include_bytes!("../src/nfs_workspace.rs"),
    ),
    ("src/ospf.rs", include_bytes!("../src/ospf.rs")),
    (
        "src/ospf_teaching.rs",
        include_bytes!("../src/ospf_teaching.rs"),
    ),
    (
        "src/ospf_workspace.rs",
        include_bytes!("../src/ospf_workspace.rs"),
    ),
    ("src/pack.rs", include_bytes!("../src/pack.rs")),
    (
        "src/process_teaching.rs",
        include_bytes!("../src/process_teaching.rs"),
    ),
    ("src/processes.rs", include_bytes!("../src/processes.rs")),
    ("src/progress.rs", include_bytes!("../src/progress.rs")),
    (
        "src/rpm_packages.rs",
        include_bytes!("../src/rpm_packages.rs"),
    ),
    (
        "src/rpm_packages_teaching.rs",
        include_bytes!("../src/rpm_packages_teaching.rs"),
    ),
    (
        "src/rpm_packages_terminal.rs",
        include_bytes!("../src/rpm_packages_terminal.rs"),
    ),
    (
        "src/rpm_packages_workspace.rs",
        include_bytes!("../src/rpm_packages_workspace.rs"),
    ),
    (
        "src/script_teaching.rs",
        include_bytes!("../src/script_teaching.rs"),
    ),
    (
        "src/seed_dns_teaching.rs",
        include_bytes!("../src/seed_dns_teaching.rs"),
    ),
    (
        "src/seed_mount_teaching.rs",
        include_bytes!("../src/seed_mount_teaching.rs"),
    ),
    (
        "src/service_teaching.rs",
        include_bytes!("../src/service_teaching.rs"),
    ),
    ("src/shell.rs", include_bytes!("../src/shell.rs")),
    (
        "src/shell/arithmetic.rs",
        include_bytes!("../src/shell/arithmetic.rs"),
    ),
    (
        "src/shell/capture.rs",
        include_bytes!("../src/shell/capture.rs"),
    ),
    ("src/shell/case.rs", include_bytes!("../src/shell/case.rs")),
    ("src/shell/cron.rs", include_bytes!("../src/shell/cron.rs")),
    (
        "src/shell/execution.rs",
        include_bytes!("../src/shell/execution.rs"),
    ),
    (
        "src/shell/getopts.rs",
        include_bytes!("../src/shell/getopts.rs"),
    ),
    (
        "src/shell/globbing.rs",
        include_bytes!("../src/shell/globbing.rs"),
    ),
    (
        "src/shell/parameters.rs",
        include_bytes!("../src/shell/parameters.rs"),
    ),
    (
        "src/shell/preparation.rs",
        include_bytes!("../src/shell/preparation.rs"),
    ),
    (
        "src/shell/substitution.rs",
        include_bytes!("../src/shell/substitution.rs"),
    ),
    (
        "src/shell/text_tools.rs",
        include_bytes!("../src/shell/text_tools.rs"),
    ),
    (
        "src/shell/values.rs",
        include_bytes!("../src/shell/values.rs"),
    ),
    (
        "src/spanning_tree.rs",
        include_bytes!("../src/spanning_tree.rs"),
    ),
    (
        "src/spanning_tree_teaching.rs",
        include_bytes!("../src/spanning_tree_teaching.rs"),
    ),
    (
        "src/spanning_tree_workspace.rs",
        include_bytes!("../src/spanning_tree_workspace.rs"),
    ),
    (
        "src/startup_trace.rs",
        include_bytes!("../src/startup_trace.rs"),
    ),
    ("src/storage.rs", include_bytes!("../src/storage.rs")),
    (
        "src/storage_teaching.rs",
        include_bytes!("../src/storage_teaching.rs"),
    ),
    (
        "src/storage_workspace.rs",
        include_bytes!("../src/storage_workspace.rs"),
    ),
    ("src/switching.rs", include_bytes!("../src/switching.rs")),
    (
        "src/switching_teaching.rs",
        include_bytes!("../src/switching_teaching.rs"),
    ),
    (
        "src/switching_workspace.rs",
        include_bytes!("../src/switching_workspace.rs"),
    ),
    (
        "src/terminal_teaching.rs",
        include_bytes!("../src/terminal_teaching.rs"),
    ),
    ("src/theme.rs", include_bytes!("../src/theme.rs")),
    ("src/ui.rs", include_bytes!("../src/ui.rs")),
    (
        "src/ui/accounts_workspace.rs",
        include_bytes!("../src/ui/accounts_workspace.rs"),
    ),
    (
        "src/ui/acl_workspace.rs",
        include_bytes!("../src/ui/acl_workspace.rs"),
    ),
    (
        "src/ui/address_learning.rs",
        include_bytes!("../src/ui/address_learning.rs"),
    ),
    (
        "src/ui/apache_static_workspace.rs",
        include_bytes!("../src/ui/apache_static_workspace.rs"),
    ),
    (
        "src/ui/client_learning.rs",
        include_bytes!("../src/ui/client_learning.rs"),
    ),
    (
        "src/ui/controls.rs",
        include_bytes!("../src/ui/controls.rs"),
    ),
    (
        "src/ui/cron_workspace.rs",
        include_bytes!("../src/ui/cron_workspace.rs"),
    ),
    (
        "src/ui/cron_workspace/memo.rs",
        include_bytes!("../src/ui/cron_workspace/memo.rs"),
    ),
    ("src/ui/data.rs", include_bytes!("../src/ui/data.rs")),
    (
        "src/ui/debian_packages_workspace.rs",
        include_bytes!("../src/ui/debian_packages_workspace.rs"),
    ),
    (
        "src/ui/dhcpv4_workspace.rs",
        include_bytes!("../src/ui/dhcpv4_workspace.rs"),
    ),
    (
        "src/ui/discovery_workspace.rs",
        include_bytes!("../src/ui/discovery_workspace.rs"),
    ),
    (
        "src/ui/dns_workspace.rs",
        include_bytes!("../src/ui/dns_workspace.rs"),
    ),
    (
        "src/ui/etherchannel_workspace.rs",
        include_bytes!("../src/ui/etherchannel_workspace.rs"),
    ),
    (
        "src/ui/etherchannel_workspace/memo.rs",
        include_bytes!("../src/ui/etherchannel_workspace/memo.rs"),
    ),
    (
        "src/ui/file_backup_workspace.rs",
        include_bytes!("../src/ui/file_backup_workspace.rs"),
    ),
    (
        "src/ui/ipv4_routing_workspace.rs",
        include_bytes!("../src/ui/ipv4_routing_workspace.rs"),
    ),
    (
        "src/ui/ipv4_workspace.rs",
        include_bytes!("../src/ui/ipv4_workspace.rs"),
    ),
    (
        "src/ui/ipv6_workspace.rs",
        include_bytes!("../src/ui/ipv6_workspace.rs"),
    ),
    (
        "src/ui/legacy_learning.rs",
        include_bytes!("../src/ui/legacy_learning.rs"),
    ),
    (
        "src/ui/mac_learning_workspace.rs",
        include_bytes!("../src/ui/mac_learning_workspace.rs"),
    ),
    (
        "src/ui/nat_workspace.rs",
        include_bytes!("../src/ui/nat_workspace.rs"),
    ),
    (
        "src/ui/nat_workspace/memo.rs",
        include_bytes!("../src/ui/nat_workspace/memo.rs"),
    ),
    (
        "src/ui/nfs_workspace.rs",
        include_bytes!("../src/ui/nfs_workspace.rs"),
    ),
    (
        "src/ui/nfs_workspace/memo.rs",
        include_bytes!("../src/ui/nfs_workspace/memo.rs"),
    ),
    (
        "src/ui/ospf_workspace.rs",
        include_bytes!("../src/ui/ospf_workspace.rs"),
    ),
    (
        "src/ui/process_learning.rs",
        include_bytes!("../src/ui/process_learning.rs"),
    ),
    (
        "src/ui/rpm_packages.rs",
        include_bytes!("../src/ui/rpm_packages.rs"),
    ),
    (
        "src/ui/rpm_packages/memo.rs",
        include_bytes!("../src/ui/rpm_packages/memo.rs"),
    ),
    (
        "src/ui/script_learning.rs",
        include_bytes!("../src/ui/script_learning.rs"),
    ),
    (
        "src/ui/scripting.rs",
        include_bytes!("../src/ui/scripting.rs"),
    ),
    (
        "src/ui/seed_learning.rs",
        include_bytes!("../src/ui/seed_learning.rs"),
    ),
    (
        "src/ui/service_learning.rs",
        include_bytes!("../src/ui/service_learning.rs"),
    ),
    ("src/ui/smoke.rs", include_bytes!("../src/ui/smoke.rs")),
    (
        "src/ui/smoke/accounts.rs",
        include_bytes!("../src/ui/smoke/accounts.rs"),
    ),
    (
        "src/ui/smoke/accounts_lessons.rs",
        include_bytes!("../src/ui/smoke/accounts_lessons.rs"),
    ),
    (
        "src/ui/smoke/acl.rs",
        include_bytes!("../src/ui/smoke/acl.rs"),
    ),
    (
        "src/ui/smoke/acl_lessons.rs",
        include_bytes!("../src/ui/smoke/acl_lessons.rs"),
    ),
    (
        "src/ui/smoke/apache_static_lessons.rs",
        include_bytes!("../src/ui/smoke/apache_static_lessons.rs"),
    ),
    (
        "src/ui/smoke/capture_lessons.rs",
        include_bytes!("../src/ui/smoke/capture_lessons.rs"),
    ),
    (
        "src/ui/smoke/case_history.rs",
        include_bytes!("../src/ui/smoke/case_history.rs"),
    ),
    (
        "src/ui/smoke/client_learning.rs",
        include_bytes!("../src/ui/smoke/client_learning.rs"),
    ),
    (
        "src/ui/smoke/command_vectors.rs",
        include_bytes!("../src/ui/smoke/command_vectors.rs"),
    ),
    (
        "src/ui/smoke/cron_lessons.rs",
        include_bytes!("../src/ui/smoke/cron_lessons.rs"),
    ),
    (
        "src/ui/smoke/debian_packages_lessons.rs",
        include_bytes!("../src/ui/smoke/debian_packages_lessons.rs"),
    ),
    (
        "src/ui/smoke/dhcpv4.rs",
        include_bytes!("../src/ui/smoke/dhcpv4.rs"),
    ),
    (
        "src/ui/smoke/dhcpv4_lessons.rs",
        include_bytes!("../src/ui/smoke/dhcpv4_lessons.rs"),
    ),
    (
        "src/ui/smoke/dns_reverse.rs",
        include_bytes!("../src/ui/smoke/dns_reverse.rs"),
    ),
    (
        "src/ui/smoke/etherchannel_lessons.rs",
        include_bytes!("../src/ui/smoke/etherchannel_lessons.rs"),
    ),
    (
        "src/ui/smoke/file_backup_lessons.rs",
        include_bytes!("../src/ui/smoke/file_backup_lessons.rs"),
    ),
    (
        "src/ui/smoke/find_lessons.rs",
        include_bytes!("../src/ui/smoke/find_lessons.rs"),
    ),
    (
        "src/ui/smoke/legacy_terminal.rs",
        include_bytes!("../src/ui/smoke/legacy_terminal.rs"),
    ),
    (
        "src/ui/smoke/mac_layout.rs",
        include_bytes!("../src/ui/smoke/mac_layout.rs"),
    ),
    (
        "src/ui/smoke/mac_learning.rs",
        include_bytes!("../src/ui/smoke/mac_learning.rs"),
    ),
    (
        "src/ui/smoke/mac_lessons.rs",
        include_bytes!("../src/ui/smoke/mac_lessons.rs"),
    ),
    (
        "src/ui/smoke/nat.rs",
        include_bytes!("../src/ui/smoke/nat.rs"),
    ),
    (
        "src/ui/smoke/nat_lessons.rs",
        include_bytes!("../src/ui/smoke/nat_lessons.rs"),
    ),
    (
        "src/ui/smoke/nfs_alignment.rs",
        include_bytes!("../src/ui/smoke/nfs_alignment.rs"),
    ),
    (
        "src/ui/smoke/nfs_lessons.rs",
        include_bytes!("../src/ui/smoke/nfs_lessons.rs"),
    ),
    (
        "src/ui/smoke/numeric_validation.rs",
        include_bytes!("../src/ui/smoke/numeric_validation.rs"),
    ),
    (
        "src/ui/smoke/optional_inputs.rs",
        include_bytes!("../src/ui/smoke/optional_inputs.rs"),
    ),
    (
        "src/ui/smoke/ospf.rs",
        include_bytes!("../src/ui/smoke/ospf.rs"),
    ),
    (
        "src/ui/smoke/ospf_lessons.rs",
        include_bytes!("../src/ui/smoke/ospf_lessons.rs"),
    ),
    (
        "src/ui/smoke/reopen.rs",
        include_bytes!("../src/ui/smoke/reopen.rs"),
    ),
    (
        "src/ui/smoke/reopen/dhcpv4.rs",
        include_bytes!("../src/ui/smoke/reopen/dhcpv4.rs"),
    ),
    (
        "src/ui/smoke/reopen/nfs.rs",
        include_bytes!("../src/ui/smoke/reopen/nfs.rs"),
    ),
    (
        "src/ui/smoke/rpm_lessons.rs",
        include_bytes!("../src/ui/smoke/rpm_lessons.rs"),
    ),
    (
        "src/ui/smoke/safe_automation_lessons.rs",
        include_bytes!("../src/ui/smoke/safe_automation_lessons.rs"),
    ),
    (
        "src/ui/smoke/script_feedback.rs",
        include_bytes!("../src/ui/smoke/script_feedback.rs"),
    ),
    (
        "src/ui/smoke/seed_teaching.rs",
        include_bytes!("../src/ui/smoke/seed_teaching.rs"),
    ),
    (
        "src/ui/smoke/spanning_tree.rs",
        include_bytes!("../src/ui/smoke/spanning_tree.rs"),
    ),
    (
        "src/ui/smoke/spanning_tree_lessons.rs",
        include_bytes!("../src/ui/smoke/spanning_tree_lessons.rs"),
    ),
    (
        "src/ui/smoke/substitution.rs",
        include_bytes!("../src/ui/smoke/substitution.rs"),
    ),
    (
        "src/ui/smoke/unit_manager.rs",
        include_bytes!("../src/ui/smoke/unit_manager.rs"),
    ),
    (
        "src/ui/smoke/unit_manager_lessons.rs",
        include_bytes!("../src/ui/smoke/unit_manager_lessons.rs"),
    ),
    (
        "src/ui/spanning_tree_workspace.rs",
        include_bytes!("../src/ui/spanning_tree_workspace.rs"),
    ),
    (
        "src/ui/storage_workspace.rs",
        include_bytes!("../src/ui/storage_workspace.rs"),
    ),
    (
        "src/ui/switching_workspace.rs",
        include_bytes!("../src/ui/switching_workspace.rs"),
    ),
    (
        "src/ui/terminal_learning.rs",
        include_bytes!("../src/ui/terminal_learning.rs"),
    ),
    ("src/ui/tests.rs", include_bytes!("../src/ui/tests.rs")),
    (
        "src/ui/tests/apache_static.rs",
        include_bytes!("../src/ui/tests/apache_static.rs"),
    ),
    (
        "src/ui/tests/apache_static_lessons.rs",
        include_bytes!("../src/ui/tests/apache_static_lessons.rs"),
    ),
    (
        "src/ui/tests/client_learning.rs",
        include_bytes!("../src/ui/tests/client_learning.rs"),
    ),
    (
        "src/ui/tests/command_vectors.rs",
        include_bytes!("../src/ui/tests/command_vectors.rs"),
    ),
    (
        "src/ui/tests/cron_lessons_native.rs",
        include_bytes!("../src/ui/tests/cron_lessons_native.rs"),
    ),
    (
        "src/ui/tests/cron_workspace.rs",
        include_bytes!("../src/ui/tests/cron_workspace.rs"),
    ),
    (
        "src/ui/tests/debian_packages.rs",
        include_bytes!("../src/ui/tests/debian_packages.rs"),
    ),
    (
        "src/ui/tests/dhcpv4.rs",
        include_bytes!("../src/ui/tests/dhcpv4.rs"),
    ),
    (
        "src/ui/tests/dhcpv4_lessons.rs",
        include_bytes!("../src/ui/tests/dhcpv4_lessons.rs"),
    ),
    (
        "src/ui/tests/etherchannel_native.rs",
        include_bytes!("../src/ui/tests/etherchannel_native.rs"),
    ),
    (
        "src/ui/tests/etherchannel_workspace.rs",
        include_bytes!("../src/ui/tests/etherchannel_workspace.rs"),
    ),
    (
        "src/ui/tests/file_backup_lessons_native.rs",
        include_bytes!("../src/ui/tests/file_backup_lessons_native.rs"),
    ),
    (
        "src/ui/tests/file_backup_workspace.rs",
        include_bytes!("../src/ui/tests/file_backup_workspace.rs"),
    ),
    (
        "src/ui/tests/getopts_history.rs",
        include_bytes!("../src/ui/tests/getopts_history.rs"),
    ),
    (
        "src/ui/tests/getopts_lessons.rs",
        include_bytes!("../src/ui/tests/getopts_lessons.rs"),
    ),
    (
        "src/ui/tests/legacy_learning.rs",
        include_bytes!("../src/ui/tests/legacy_learning.rs"),
    ),
    (
        "src/ui/tests/mac_learning.rs",
        include_bytes!("../src/ui/tests/mac_learning.rs"),
    ),
    (
        "src/ui/tests/nat.rs",
        include_bytes!("../src/ui/tests/nat.rs"),
    ),
    (
        "src/ui/tests/nat_lessons.rs",
        include_bytes!("../src/ui/tests/nat_lessons.rs"),
    ),
    (
        "src/ui/tests/nat_lessons_native.rs",
        include_bytes!("../src/ui/tests/nat_lessons_native.rs"),
    ),
    (
        "src/ui/tests/nat_native.rs",
        include_bytes!("../src/ui/tests/nat_native.rs"),
    ),
    (
        "src/ui/tests/nat_teaching.rs",
        include_bytes!("../src/ui/tests/nat_teaching.rs"),
    ),
    (
        "src/ui/tests/nfs_lessons_native.rs",
        include_bytes!("../src/ui/tests/nfs_lessons_native.rs"),
    ),
    (
        "src/ui/tests/nfs_workspace.rs",
        include_bytes!("../src/ui/tests/nfs_workspace.rs"),
    ),
    (
        "src/ui/tests/numeric_validation.rs",
        include_bytes!("../src/ui/tests/numeric_validation.rs"),
    ),
    (
        "src/ui/tests/optional_inputs.rs",
        include_bytes!("../src/ui/tests/optional_inputs.rs"),
    ),
    (
        "src/ui/tests/ospf.rs",
        include_bytes!("../src/ui/tests/ospf.rs"),
    ),
    (
        "src/ui/tests/rpm_lessons_native.rs",
        include_bytes!("../src/ui/tests/rpm_lessons_native.rs"),
    ),
    (
        "src/ui/tests/rpm_packages.rs",
        include_bytes!("../src/ui/tests/rpm_packages.rs"),
    ),
    (
        "src/ui/tests/safe_automation_lessons.rs",
        include_bytes!("../src/ui/tests/safe_automation_lessons.rs"),
    ),
    (
        "src/ui/tests/seed_learning.rs",
        include_bytes!("../src/ui/tests/seed_learning.rs"),
    ),
    (
        "src/ui/tests/spanning_tree.rs",
        include_bytes!("../src/ui/tests/spanning_tree.rs"),
    ),
    (
        "src/ui/tests/ui_consistency.rs",
        include_bytes!("../src/ui/tests/ui_consistency.rs"),
    ),
    (
        "src/ui/tests/unit_manager.rs",
        include_bytes!("../src/ui/tests/unit_manager.rs"),
    ),
    (
        "src/ui/unit_manager_workspace.rs",
        include_bytes!("../src/ui/unit_manager_workspace.rs"),
    ),
    (
        "src/ui/workspace.rs",
        include_bytes!("../src/ui/workspace.rs"),
    ),
    (
        "src/unit_manager.rs",
        include_bytes!("../src/unit_manager.rs"),
    ),
    (
        "src/unit_manager/journal.rs",
        include_bytes!("../src/unit_manager/journal.rs"),
    ),
    (
        "src/unit_manager_teaching.rs",
        include_bytes!("../src/unit_manager_teaching.rs"),
    ),
    (
        "src/unit_manager_terminal.rs",
        include_bytes!("../src/unit_manager_terminal.rs"),
    ),
    (
        "src/unit_manager_terminal/session.rs",
        include_bytes!("../src/unit_manager_terminal/session.rs"),
    ),
    (
        "src/unit_manager_workspace.rs",
        include_bytes!("../src/unit_manager_workspace.rs"),
    ),
    ("src/workspace.rs", include_bytes!("../src/workspace.rs")),
    (
        "tests/fixtures/bash-before12-source-index.json",
        include_bytes!("../tests/fixtures/bash-before12-source-index.json"),
    ),
    (
        "tests/fixtures/bash-before13-source-index.json",
        include_bytes!("../tests/fixtures/bash-before13-source-index.json"),
    ),
    (
        "tests/support/actual10_evidence.rs",
        include_bytes!("../tests/support/actual10_evidence.rs"),
    ),
    (
        "tests/support/actual11_evidence.rs",
        include_bytes!("../tests/support/actual11_evidence.rs"),
    ),
    (
        "tests/support/actual9_evidence.rs",
        include_bytes!("../tests/support/actual9_evidence.rs"),
    ),
    (
        "tools/automation_before10_oracle.py",
        include_bytes!("../tools/automation_before10_oracle.py"),
    ),
    (
        "tools/build_safety.py",
        include_bytes!("../tools/build_safety.py"),
    ),
    (
        "tools/case_bash_oracle.py",
        include_bytes!("../tools/case_bash_oracle.py"),
    ),
    ("tools/dev.py", include_bytes!("../tools/dev.py")),
    (
        "tools/native_reopen.py",
        include_bytes!("../tools/native_reopen.py"),
    ),
    (
        "tools/parameters_before11_verify.py",
        include_bytes!("../tools/parameters_before11_verify.py"),
    ),
    (
        "tools/scripting_before12_verify.py",
        include_bytes!("../tools/scripting_before12_verify.py"),
    ),
    (
        "tools/scripting_before13_verify.py",
        include_bytes!("../tools/scripting_before13_verify.py"),
    ),
    (
        "tools/tests/test_scripting_before13_verify.py",
        include_bytes!("../tools/tests/test_scripting_before13_verify.py"),
    ),
];
fn value(v: &impl serde::Serialize) -> Value {
    serde_json::to_value(v).unwrap()
}
fn write(path: &Path, raw: &[u8]) {
    assert!(raw.len() <= 16_000_000);
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    f.write_all(raw).unwrap();
    f.sync_all().unwrap();
}
fn json_file(path: &Path, data: &impl serde::Serialize) {
    write(path, &serde_json::to_vec(data).unwrap());
}
fn pin(root: &Path, path: &Path) -> Value {
    assert!(path.starts_with(root));
    let mut at = root.to_path_buf();
    for part in path.strip_prefix(root).unwrap().components() {
        at.push(part);
        assert!(!fs::symlink_metadata(&at).unwrap().file_type().is_symlink());
    }
    let file = File::open(path).unwrap();
    let info = file.metadata().unwrap();
    assert!(info.is_file() && info.len() <= 16_000_000);
    let mut bytes = Vec::new();
    file.take(16_000_001).read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes.len() as u64, info.len());
    json!({"file":path.strip_prefix(root).unwrap().to_str().unwrap(),"bytes":bytes.len(),"sha256":actual11::sha(&bytes)})
}
fn files(root: &Path, path: &Path, out: &mut Vec<Value>) {
    assert!(!fs::symlink_metadata(path).unwrap().file_type().is_symlink());
    if path.is_dir() {
        let mut paths = fs::read_dir(path)
            .unwrap()
            .map(|p| p.unwrap().path())
            .collect::<Vec<_>>();
        paths.sort();
        for p in paths {
            files(root, &p, out);
        }
    } else {
        out.push(pin(root, path));
    }
    assert!(out.len() <= 1500);
}
fn records(a: &Attempt, current: bool) -> BTreeMap<String, usize> {
    let mut rows = BTreeMap::new();
    if current && let State::Script { run: Some(r), .. } = &a.state {
        *rows.entry(serde_json::to_string(r).unwrap()).or_default() += 1;
    }
    if let Some(w) = &a.workspace {
        for r in w.script_history.iter().chain(&w.script_history_more) {
            *rows.entry(serde_json::to_string(r).unwrap()).or_default() += 1;
        }
    }
    rows
}
fn metadata(mut v: Value) -> Value {
    v.as_object_mut().unwrap().remove("state");
    if let Some(w) = v.get_mut("workspace").and_then(Value::as_object_mut) {
        for k in ["script_history", "script_history_more", "checked"] {
            w.remove(k);
        }
        if let Some(l) = w.get_mut("script_learning").and_then(Value::as_object_mut) {
            l.remove("example_revision");
            l.remove("example_result");
        }
    }
    v
}
fn index_profile_contexts() -> usize {
    let index: Value = serde_json::from_slice(INDEX).unwrap();
    let ids = index["prior_profile_ids"].as_array().unwrap();
    assert_eq!(ids.len(), 142);
    ids.len()
}
struct Observations {
    stream: File,
    bytes: usize,
    rows: usize,
}
impl Observations {
    fn new(path: &Path) -> Self {
        Self {
            stream: OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .unwrap(),
            bytes: 0,
            rows: 0,
        }
    }
    fn append(&mut self, id: &str, role: &str, case: usize, outcome: &shell::Outcome) {
        let raw = serde_json::to_vec(&json!({"id":id,"role":role,"case":case,"outcome":outcome}))
            .unwrap();
        self.bytes = self.bytes.checked_add(raw.len() + 1).unwrap();
        self.rows += 1;
        assert!(self.bytes <= 16_000_000 && self.rows <= 475);
        self.stream.write_all(&raw).unwrap();
        self.stream.write_all(b"\n").unwrap();
        self.stream.sync_data().unwrap();
    }
}
fn profile(
    root: &Path,
    destination: &Path,
    raw: &[u8],
    catalog: &[pack::Scenario],
    rows: &[Value],
    drafts: bool,
) -> Value {
    let old: Progress = serde_json::from_slice(raw).unwrap();
    let mut p = progress::decode(raw, catalog).unwrap();
    assert_eq!(value(&old), value(&p));
    assert_eq!(p.attempts.len(), index_profile_contexts());
    let index: Value = serde_json::from_slice(INDEX).unwrap();
    assert_eq!(
        json!(p.attempts.keys().collect::<Vec<_>>()),
        index["prior_profile_ids"]
    );
    assert_eq!(
        p.attempts
            .values()
            .filter(|a| matches!(a.state, State::Script { .. }))
            .count(),
        52
    );
    assert_eq!(
        p.attempts
            .values()
            .filter(|a| !matches!(a.state, State::Script { .. }))
            .count(),
        90
    );
    for id in NEW {
        assert!(
            !p.attempts.contains_key(id),
            "Only genuinely absent script fixtures may be added"
        );
    }
    assert_eq!(
        value(&p.migration_receipts),
        json!({"workspace":true,"shell":[7,8,9,10,11]})
    );
    let mut added = Vec::new();
    let mut executed = 0;
    for row in rows {
        let id = row["id"].as_str().unwrap();
        let s = catalog.iter().find(|s| s.id == id).unwrap();
        if drafts
            && old
                .attempts
                .get(id)
                .is_some_and(|a| matches!(a.state, State::Script { run: None, .. }))
        {
            assert!(DRAFTS.contains(&id));
            assert_eq!(value(&p.attempts[id]), value(&old.attempts[id]));
            continue;
        }
        p.clock += 1;
        executed += 1;
        if !p.attempts.contains_key(id) {
            assert!(NEW.contains(&id));
            added.push(id);
        }
        let a = p
            .attempts
            .entry(id.into())
            .or_insert_with(|| Attempt::new(s, p.clock));
        let before = old.attempts.get(id);
        if let Some(before) = before {
            assert!(matches!(before.state, State::Script { run: Some(_), .. }));
        }
        workspace::prepare(a);
        let record: Record = serde_json::from_value(row["record"].clone()).unwrap();
        assert_eq!(record.engine_revision, 12);
        a.state = State::Script {
            source: record.source.clone(),
            run: Some(record),
        };
        let lesson = s.teaching.as_ref().unwrap();
        let w = a.workspace.as_mut().unwrap();
        let l = w
            .script_learning
            .get_or_insert_with(|| script_teaching::Learning::new(lesson));
        assert_eq!(
            l.example_source, lesson.example,
            "Never attach worked-example results to another draft"
        );
        l.example_revision = 12;
        l.example_result = Some(serde_json::from_value(row["example_outcome"].clone()).unwrap());
        if let Some(before) = before {
            assert_eq!(
                metadata(value(a)),
                metadata(value(before)),
                "Unchanged learner metadata: {id}"
            );
            assert_eq!(
                records(a, false),
                records(before, true),
                "Every genuine prior record retained: {id}"
            );
        }
        assert!(progress::current_solved(a, s));
    }
    assert_eq!(added, NEW);
    assert_eq!(executed, if drafts { 53 } else { 55 });
    assert_eq!(p.attempts.len(), index_profile_contexts() + NEW.len());
    assert_eq!(p.favorites, old.favorites);
    for (id, a) in &old.attempts {
        if !matches!(a.state, State::Script { .. }) {
            assert_eq!(value(&p.attempts[id]), value(a));
        }
    }
    let missing = catalog
        .iter()
        .filter(|s| !p.attempts.contains_key(&s.id))
        .collect::<Vec<_>>();
    assert_eq!(missing.len(), 26);
    assert!(
        missing
            .iter()
            .all(|s| !matches!(s.initial, State::Script { .. }))
    );
    if drafts {
        for id in DRAFTS {
            assert_eq!(value(&p.attempts[id]), value(&old.attempts[id]));
        }
    }
    let dir = destination.join(if drafts {
        "draft-profile"
    } else {
        "original-profile"
    });
    fs::create_dir(&dir).unwrap();
    let path = dir.join("progress-v1.json");
    write(&path, raw);
    let saved = progress::save(&path, &p).unwrap();
    assert!(saved.durability_warning.is_none());
    let loaded = progress::load(&path, catalog).unwrap();
    assert_eq!(value(&loaded), value(&p));
    assert_eq!(
        loaded.migration_receipts.as_ref().unwrap().shell,
        vec![7, 8, 9, 10, 11, 12]
    );
    let backup = path.with_extension("before-shell-v12.json");
    assert_eq!(fs::read(&backup).unwrap(), raw);
    let saved_raw = fs::read(&path).unwrap();
    write(
        &destination.join(if drafts {
            "writer12-drafts.json"
        } else {
            "writer12-original.json"
        }),
        &saved_raw,
    );
    progress::save(&path, &loaded).unwrap();
    assert_eq!(fs::read(&path).unwrap(), saved_raw);
    assert_eq!(fs::read(&backup).unwrap(), raw);
    json!({"prior_contexts":index_profile_contexts(),"contexts":p.attempts.len(),"actual12_current_records":executed,"actual12_examples":executed,
        "retained_unfinished_assignment_and_practice":if drafts {2}else{0},"explicit_new_script_ids":added,
        "missing_non_script_ids":missing.iter().map(|s|s.id.as_str()).collect::<Vec<_>>(),
        "non_script_original_attempts_exact":90,"all_prior_records_retained":true,"raw11_backup":pin(root,&backup),
        "receipts":[7,8,9,10,11,12]})
}
fn main() {
    assert_eq!(
        std::env::args_os().len(),
        1,
        "No source/input/path/engine CLI"
    );
    assert_eq!(
        shell::EVALUATOR_REVISION,
        12,
        "Cannot label later evaluator observations as historical12"
    );
    assert_eq!(
        std::env::var("DLMS_FIXED_CAPTURE12").as_deref(),
        Ok("before13-r1")
    );
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert!(root.join(".git").is_dir(), "MAIN only");
    let root = fs::canonicalize(root).unwrap();
    assert_eq!(
        std::env::var_os("CARGO_TARGET_DIR"),
        Some(root.join("target").into_os_string())
    );
    // Bind observed semantics to the actual compiled source, not just a runtime
    // revision label. A stale same-revision ELF cannot attest newer core bytes.
    for (name, compiled) in COMPILED_CORE {
        assert_eq!(
            pin(&root, &root.join(name))["sha256"],
            actual11::sha(compiled),
            "Compiled evaluator/progress core differs from current source: {name}"
        );
    }
    assert_eq!(actual11::sha(INDEX), INDEX_SHA);
    let index: Value = serde_json::from_slice(INDEX).unwrap();
    let catalog = pack::bundled().unwrap();
    assert_eq!(catalog.len(), 171);
    assert_eq!(
        json!(
            catalog
                .iter()
                .map(|s| json!({"id":s.id,"version":s.version}))
                .collect::<Vec<_>>()
        ),
        index["catalog"]
    );
    for row in index["bundled_packs"].as_array().unwrap() {
        let path = row["path"].as_str().unwrap();
        assert!(path.starts_with("scenarios/") && Path::new(path).components().count() == 2);
        assert_eq!(pin(&root, &root.join(path))["sha256"], row["sha256"]);
    }
    let scripts = catalog
        .iter()
        .filter(|s| matches!(s.goal, Goal::Script { .. }))
        .collect::<Vec<_>>();
    assert_eq!(scripts.len(), 55);
    assert_eq!(index["scripts"].as_array().unwrap().len(), 55);
    let original = actual11::observed("writer11-original.json");
    let drafts = actual11::observed("writer11-drafts.json");
    assert_eq!(
        actual11::sha(&original),
        index["prior_original_sha256"].as_str().unwrap()
    );
    assert_eq!(
        actual11::sha(&drafts),
        index["prior_drafts_sha256"].as_str().unwrap()
    );
    let destination = root.join(DESTINATION);
    let parent = destination.parent().unwrap();
    let mut at = root.clone();
    for c in parent.strip_prefix(&root).unwrap().components() {
        at.push(c);
        match fs::symlink_metadata(&at) {
            Ok(info) => assert!(info.is_dir() && !info.file_type().is_symlink()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => fs::create_dir(&at).unwrap(),
            Err(e) => panic!("Capture parent observation failed: {e}"),
        }
    }
    fs::create_dir(&destination).expect("Fresh fixed capture only; preserve all prior evidence");
    let mut inputs = Vec::new();
    for (name, _) in COMPILED_CORE {
        inputs.push(pin(&root, &root.join(name)));
    }
    json_file(
        &destination.join("SOURCE.json"),
        &json!({"actual_evaluator_revision":12,"source_index_sha256":INDEX_SHA,"inputs":inputs}),
    );
    write(&destination.join("input11-original.json"), &original);
    write(&destination.join("input11-drafts.json"), &drafts);
    let mut rows = Vec::new();
    let mut observations = Observations::new(&destination.join("VM-OBSERVATIONS.jsonl"));
    let mut count = 0;
    for (s, expected) in scripts.iter().zip(index["scripts"].as_array().unwrap()) {
        let Goal::Script { cases } = &s.goal else {
            unreachable!()
        };
        let lesson = s.teaching.as_ref().unwrap();
        assert_eq!(
            json!({"id":s.id,"version":s.version,"source":s.solution[0],"cases":cases,
            "example_source":lesson.example,"example_case":lesson.fixture}),
            *expected
        );
        let outcomes = cases
            .iter()
            .enumerate()
            .map(|(n, c)| {
                let observed = shell::run(&s.solution[0], c);
                observations.append(&s.id, "assignment", n, &observed);
                observed
            })
            .collect::<Vec<_>>();
        assert!(cases.iter().zip(&outcomes).all(|(c, o)| c.accepts(o)));
        count += cases.len();
        let example = shell::run(&lesson.example, &lesson.fixture);
        observations.append(&s.id, "example", 0, &example);
        assert!(lesson.fixture.accepts(&example));
        rows.push(json!({"id":s.id,"source":s.solution[0],"cases":cases,
            "record":Record{engine_revision:12,source:s.solution[0].clone(),results:outcomes},
            "example_source":lesson.example,"example_case":lesson.fixture,"example_outcome":example,"example_revision":12}));
    }
    assert_eq!(count, 420);
    assert_eq!(observations.rows, 475);
    observations.stream.sync_all().unwrap();
    drop(observations);
    let mut stream = Vec::new();
    for row in &rows {
        serde_json::to_writer(&mut stream, row).unwrap();
        stream.push(b'\n');
    }
    write(&destination.join("actual12-authored.jsonl"), &stream);
    let original_profile = profile(&root, &destination, &original, &catalog, &rows, false);
    let draft_profile = profile(&root, &destination, &drafts, &catalog, &rows, true);
    let mut outputs = Vec::new();
    files(&destination, &destination, &mut outputs);
    let manifest = json!({"scope":"Synthetic development evidence: actual current12 VM reference executions, never personal progress or learner mastery",
        "actual_evaluator_revision":12,"catalog_contexts":171,"script_records":55,"authored_cases":420,"examples":55,
        "prior_actual11_manifest_sha256":"e303bdecb9f57a89debd77c8300d27b523c94630446bc918e4c5e4c171373168",
        "source_index_sha256":INDEX_SHA,"original_profile":original_profile,"draft_profile":draft_profile,
        "source":pin(&destination,&destination.join("SOURCE.json")),"outputs":outputs});
    let raw = serde_json::to_vec(&manifest).unwrap();
    write(&destination.join("MANIFEST.json"), &raw);
    write(
        &destination.join("COMPLETE"),
        format!("{}\n", actual11::sha(&raw)).as_bytes(),
    );
    println!(
        "PASS genuine actual12 before13:55 scripts/420cases/55 examples;145 derived profile contexts;2 authentic drafts retained; no network/backup/OneOff mastery"
    );
}
