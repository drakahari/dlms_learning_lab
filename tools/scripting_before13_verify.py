#!/usr/bin/env python3
"""Fixed genuine12 before13 development capture/attestation, not learner input.

No source/profile/path/engine CLI. Prepare launches nothing. Capture requires
Root-reviewed exact current source/binary pins and the original sealed genuine11 data.
SEALED_MANIFEST is filled only after an actual12 capture and independent review;
until then verification cannot authorize a future evaluator13 transition.
"""
import argparse
import copy
from collections import Counter
import hashlib
import json
import os
import re
import stat
from pathlib import Path

import case_bash_oracle as safe
import scripting_before12_verify as before11
from automation_before10_oracle import pinned_executable

ROOT = Path(__file__).resolve().parents[1]
HERE = '.build-artifacts/reviews/bash-loop-recovery/captures'
DESTINATION = HERE + '/public12-171-420-r1'
PRODUCER = 'target/debug/examples/scripting_before13_capture'
SOURCE = 'examples/scripting_before13_capture.rs'
INDEX = 'tests/fixtures/bash-before13-source-index.json'
INDEX_SHA = '69efc842123cb50df55c29141f40441f804a3fe1a7e6ac80a1aaa6f9aef3fcbc'
PRIOR_SHA = 'e303bdecb9f57a89debd77c8300d27b523c94630446bc918e4c5e4c171373168'
SELF = 'tools/scripting_before13_verify.py'
TEST = 'tools/tests/test_scripting_before13_verify.py'
CAP = 16_000_000
BINARY_CAP = 512 * 1024 * 1024
TOTAL = 256 * 1024 * 1024
ATTEST_TOTAL = TOTAL + BINARY_CAP
SEALED_MANIFEST = "109d1c747c8bb9cb8f497abcc5a3e13be5de53de6447b37fccc728ec15aeb448"  # Genuine12; Root and independent raw readback.
SEALED_ATTESTATION = "0f03d3474a70fa01295b545d6c14cec26ecdf8db758773cb951621e78e630f05"
NEW = ['shell-numeric-retry', 'shell-numeric-intake', 'shell-numeric-publication']
DRAFTS = ['shell-command-plan', 'shell-handover']


def canonical(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(',', ':')).encode()


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def strict(raw):
    def pairs(items):
        value = {}
        for k, v in items:
            safe.require(k not in value, 'Duplicate JSON key')
            value[k] = v
        return value
    return json.loads(raw, object_pairs_hook=pairs)


def contained(root, name):
    safe.require(isinstance(name, str), 'Contained fixed relative path')
    path = Path(name)
    safe.require(isinstance(name, str) and not path.is_absolute() and path.parts
                 and '..' not in path.parts and str(path) == name, 'Contained fixed relative path')
    return root / path


def index_value(raw):
    safe.require(sha(raw) == INDEX_SHA, 'Fixed55 authored input index changed')
    value = strict(raw)
    for key, expected in {'actual_evaluator_revision_required': 12, 'catalog_count': 171,
                           'script_count': 55, 'authored_cases': 420, 'separate_examples': 55,
                           'prior_profile_contexts': 142, 'expected_profile_contexts': 145}.items():
        safe.require(value[key] == expected and not isinstance(value[key], bool), 'Fixed actual12 scope: ' + key)
    safe.require(value['explicit_new_script_contexts'] == NEW and value['retained_draft_ids'] == DRAFTS,
                 'Only3 explicit new script fixtures and2 authentic drafts')
    safe.require(value['prior_actual11_manifest_sha256'] == PRIOR_SHA
                 and len(value['catalog']) == 171 and len(value['scripts']) == 55
                 and len(value['prior_profile_ids']) == 142 and len(set(value['prior_profile_ids'])) == 142
                 and len(value['non_script_absent_not_invented']) == 26,
                 'Exact authored/legacy identity counts')
    safe.require(sum(len(s['cases']) for s in value['scripts']) == 420, 'All420 authored cases')
    safe.require(len(set(value['prior_script_ids'])) == 52 and len(set(value['prior_non_script_ids'])) == 90
                 and not set(value['prior_script_ids']) & set(value['prior_non_script_ids'])
                 and set(value['prior_script_ids']) | set(value['prior_non_script_ids']) == set(value['prior_profile_ids'])
                 and not set(NEW) & set(value['prior_profile_ids'])
                 and len(set(value['prior_profile_ids']) | set(NEW)) == value['expected_profile_contexts'],
                 'Derived52+90 genuine prior identities and only3 absent numeric contexts')
    return value


def load_index(root):
    return index_value(safe.read_bytes(root / INDEX, CAP))

def input11(root, index):
    safe.require(before11.SEALED_MANIFEST == PRIOR_SHA and before11.DESTINATION == (
        '.build-artifacts/reviews/bash-numeric-validation/captures/public11-164-384-r1'), 'Fixed genuine11 verifier identity')
    before11.require_seals()
    before11.verify_capture(root, PRIOR_SHA)
    attestation = safe.read_bytes(root / before11.DESTINATION / 'ATTESTATION.json', CAP)
    safe.require(sha(attestation) == before11.SEALED_ATTESTATION == index['prior_actual11_attestation_sha256'],
                 'Genuine11 independent attestation pin')
    result = {}
    for role in ['original', 'drafts']:
        raw = safe.read_bytes(root / before11.DESTINATION / ('writer11-' + role + '.json'), CAP)
        safe.require(sha(raw) == index['prior_' + role + '_sha256'], 'Original11 raw profile pin changed')
        value = strict(raw)
        safe.require(sorted(value['attempts']) == index['prior_profile_ids']
                     and value['migration_receipts'] == {'workspace': True, 'shell': [7, 8, 9, 10, 11]},
                     'Authentic genuine11 contexts/receipts unchanged')
        scripts = {k for k, a in value['attempts'].items() if a['state']['kind'] == 'Script'}
        safe.require(scripts == set(index['prior_script_ids']) and len(scripts) == 52
                     and set(value['attempts']) - scripts == set(index['prior_non_script_ids'])
                     and len(index['prior_non_script_ids']) == 90 and not set(NEW) & set(value['attempts']),
                     'Derive context union from actual52Script/90nonScript IDs; add only absent3 numeric fixtures')
        result[role] = (raw, value)
    return result


def outcome(expected, actual):
    return isinstance(actual, dict) and set(actual) == {'stdout', 'stderr', 'status', 'files', 'error'} \
        and actual['error'] is None and actual['stdout'] == expected['stdout'] \
        and actual['stderr'] == expected['stderr'] and actual['status'] == expected['status'] \
        and not isinstance(actual['status'], bool) and actual['files'] == expected['after']


def authored(raw, index):
    lines = raw.decode('utf-8', errors='strict').splitlines()
    safe.require(len(lines) == 55, 'All55 actual12 script/example observations required')
    rows = []
    for text, expected in zip(lines, index['scripts']):
        row = strict(text)
        safe.require(row['id'] == expected['id'] and row['source'] == expected['source']
                     and row['cases'] == expected['cases'] and row['example_source'] == expected['example_source']
                     and row['example_case'] == expected['example_case'] and row['example_revision'] == 12,
                     'Actual12 observation uses exact authored source/input/order')
        record = row['record']
        safe.require(set(record) == {'engine_revision', 'source', 'results'} and record['engine_revision'] == 12
                     and record['source'] == expected['source'] and len(record['results']) == len(expected['cases']),
                     'Actual12 record revision/source/case count')
        safe.require(all(outcome(c, r) for c, r in zip(expected['cases'], record['results']))
                     and outcome(expected['example_case'], row['example_outcome']),
                     'Actual12 outcomes must match every independently authored behavior')
        rows.append(row)
    return rows


def case_observations(raw, index, records):
    lines = raw.decode('utf-8', errors='strict').splitlines()
    safe.require(len(lines) == 475, 'Every420 assignment and55 example API outcome is retained')
    expected = []
    for row in records:
        expected.extend({'id': row['id'], 'role': 'assignment', 'case': n, 'outcome': result}
                        for n, result in enumerate(row['record']['results']))
        expected.append({'id': row['id'], 'role': 'example', 'case': 0, 'outcome': row['example_outcome']})
    actual = [strict(line) for line in lines]
    safe.require(actual == expected and sum(len(s['cases']) for s in index['scripts']) == 420,
                 'Durable per-API observations equal exact authored Record/example results, source order and identities')
    return actual


def records(attempt, include_current):
    values = []
    if include_current and attempt.get('state', {}).get('run') is not None:
        values.append(attempt['state']['run'])
    work = attempt.get('workspace', {})
    if work.get('script_history') is not None:
        values.append(work['script_history'])
    values.extend(work.get('script_history_more', []))
    return Counter(canonical(v) for v in values)


def stable_metadata(attempt):
    value = copy.deepcopy(attempt)
    value.pop('state')
    work = value.get('workspace', {})
    for field in ['script_history', 'script_history_more', 'checked']:
        work.pop(field, None)
    for field in ['example_revision', 'example_result']:
        work.get('script_learning', {}).pop(field, None)
    return value


def profile(old, current, index, observations, drafts):
    expected_ids = set(index['prior_profile_ids']) | set(NEW)
    safe.require(set(current['attempts']) == expected_ids and len(expected_ids) == len(index['prior_profile_ids']) + len(NEW) == 145,
                 '142 authentic contexts plus only3 explicitly executed script fixtures')
    safe.require(set(current) == set(old), 'Preserve exact profile top-level fields')
    safe.require(current['favorites'] == old['favorites'] and current['schema_version'] == old['schema_version']
                 and current['migration_receipts'] == {'workspace': True, 'shell': [7, 8, 9, 10, 11, 12]},
                 'Preserved favorites/schema and genuine12 receipt append')
    by_id = {row['id']: row for row in observations}
    skipped = set(DRAFTS) if drafts else set()
    safe.require(current['clock'] == old['clock'] + (53 if drafts else 55), 'Explicit executed fixture clock only')
    for name, before in old['attempts'].items():
        after = current['attempts'][name]
        if before['state']['kind'] != 'Script' or name in skipped:
            safe.require(after == before, 'Unchanged non-script/authentic draft: ' + name)
            continue
        row = by_id[name]
        safe.require(stable_metadata(after) == stable_metadata(before), 'All other learner metadata retained: ' + name)
        safe.require(records(after, False) == records(before, True), 'Every genuine6..11 record retained: ' + name)
        safe.require(after['state'] == {'kind': 'Script', 'source': row['source'], 'run': row['record']},
                     'Only actually executed12 reference supplies current credit: ' + name)
        work = after['workspace']
        safe.require(work['checked'] is False and work['script_learning']['example_revision'] == 12
                     and work['script_learning']['example_result'] == row['example_outcome'], 'Actually rerun12 example only')
    counter = old['clock']
    for row in observations:
        name = row['id']
        if name in skipped:
            continue
        counter += 1
        if name in NEW:
            a = current['attempts'][name]
            safe.require(a['state'] == {'kind': 'Script', 'source': row['source'], 'run': row['record']}
                         and a['scenario_version'] == next(s['version'] for s in index['scripts'] if s['id'] == name)
                         and a['step'] == 0 and a['hints'] == 0 and a['challenge'] is False
                         and a['transcript'] == [] and a['visited'] == counter,
                         'New synthetic script context follows real fixed execution only')
            work = a['workspace']
            safe.require(work['script_learning']['example_revision'] == 12
                         and work['script_learning']['example_source'] == row['example_source']
                         and work['script_learning']['example_result'] == row['example_outcome']
                         and not any(k.startswith(('cron_', 'etherchannel_', 'nfs_', 'one_off_', 'physical_link_', 'file_backup_',
                                                         'rpm_', 'apache_static_', 'switching_', 'ipv6_', 'ospf_', 'acl_',
                                                         'unit_manager_', 'mac_', 'accounts_', 'storage_', 'debian_', 'dhcpv4_', 'ipv4_routing_')) for k in work),
                         'No fabricated other-family metadata/mastery')
    safe.require(not set(index['non_script_absent_not_invented']) & set(current['attempts']), 'Never invent26 later non-script contexts')


def source_index(root, folder, *, require_retained=False):
    """Only pre-attestation validation may use the exact current static index.

    Final sealed verification requires the ordinary retained tree and every
    SOURCE input. An absent whole tree must never fall back to current source.
    """
    frozen = folder / 'compiled-source'
    try:
        info = frozen.lstat()
    except FileNotFoundError:
        safe.require(not require_retained, 'Sealed actual12 retained compiled-source directory is required')
        return index_value(safe.read_bytes(root / INDEX, CAP))
    safe.require(stat.S_ISDIR(info.st_mode) and not stat.S_ISLNK(info.st_mode),
                 'Retained compiled-source must be an ordinary directory')
    source = strict(safe.read_bytes(folder / 'SOURCE.json', CAP))
    safe.require(source['actual_evaluator_revision'] == 12 and source['source_index_sha256'] == INDEX_SHA
                 and 1 <= len(source['inputs']) <= 1500, 'Frozen actual12 source identity')
    names = set()
    for pin in source['inputs']:
        name = pin['file']
        safe.require(name not in names, 'Unique frozen source input'); names.add(name)
        observed = safe.pin(contained(frozen, name), CAP)
        safe.require({k: observed[k] for k in ['bytes', 'sha256']} == {k: pin[k] for k in ['bytes', 'sha256']},
                     'Frozen source changed: ' + name)
    return index_value(safe.read_bytes(frozen / INDEX, CAP))


def verify_capture(root, expected_manifest, *, require_retained=False):
    safe.require(isinstance(expected_manifest, str) and len(expected_manifest) == 64, 'Actual reviewed12 manifest pin required')
    folder = root / DESTINATION
    raw = safe.read_bytes(folder / 'MANIFEST.json', CAP)
    safe.require(sha(raw) == expected_manifest and safe.read_bytes(folder / 'COMPLETE', 65) == (expected_manifest + '\n').encode(),
                 'Exact actual12 manifest and completion marker')
    manifest = strict(raw)
    for key, value in {'actual_evaluator_revision': 12, 'catalog_contexts': 171, 'script_records': 55,
                       'authored_cases': 420, 'examples': 55}.items():
        safe.require(manifest[key] == value and not isinstance(manifest[key], bool), 'Closed actual12 scope')
    safe.require(manifest['prior_actual11_manifest_sha256'] == PRIOR_SHA and manifest['source_index_sha256'] == INDEX_SHA,
                 'Actual12 provenance is linked to authentic11 and static authored inputs')
    seen = set()
    for pin in manifest['outputs']:
        name = pin['file']
        safe.require(name not in seen and len(seen) < 128, 'Unique bounded capture outputs')
        seen.add(name)
        observed = safe.pin(contained(folder, name), CAP)
        safe.require({k: observed[k] for k in ['bytes', 'sha256']} == {k: pin[k] for k in ['bytes', 'sha256']},
                     'Actual12 output changed: ' + name)
    required = {'VM-OBSERVATIONS.jsonl', 'SOURCE.json', 'input11-original.json', 'input11-drafts.json', 'actual12-authored.jsonl',
                'writer12-original.json', 'writer12-drafts.json',
                'original-profile/progress-v1.json', 'original-profile/progress-v1.before-shell-v12.json',
                'draft-profile/progress-v1.json', 'draft-profile/progress-v1.before-shell-v12.json'}
    safe.require(seen == required, 'Exact closed12 capture output identities')
    index = source_index(root, folder, require_retained=require_retained)
    inputs = input11(root, index)
    observations = authored(safe.read_bytes(folder / 'actual12-authored.jsonl', CAP), index)
    case_observations(safe.read_bytes(folder / 'VM-OBSERVATIONS.jsonl', CAP), index, observations)
    for role, drafts in [('original', False), ('drafts', True)]:
        original_raw, original = inputs[role]
        safe.require(safe.read_bytes(folder / ('input11-' + role + '.json'), CAP) == original_raw, 'Actual11 inputs copied exactly')
        current_raw = safe.read_bytes(folder / ('writer12-' + role + '.json'), CAP)
        current = strict(current_raw)
        profile(original, current, index, observations, drafts)
        directory = folder / ('draft-profile' if drafts else 'original-profile')
        safe.require(safe.read_bytes(directory / 'progress-v1.json', CAP) == current_raw
                     and safe.read_bytes(directory / 'progress-v1.before-shell-v12.json', CAP) == original_raw,
                     'Current12 main/actual11 backup exact')
    return manifest


def source_set(root):
    names = ['.cargo/config.toml', 'Cargo.lock', 'Cargo.toml', 'docs/BUILD_WORKFLOW.md', 'examples/automation_before10_capture.rs', 'examples/parameters_before11_capture.rs', 'examples/scripting_before12_capture.rs', 'examples/scripting_before13_capture.rs', 'examples/support/automation_digest.rs', 'scenarios/accounts-lifecycle-v1.json', 'scenarios/acl-policy-v1.json', 'scenarios/apache-static-v1.json', 'scenarios/debian-packages-v1.json', 'scenarios/dhcpv4-client-v1.json', 'scenarios/discovery-v1.json', 'scenarios/dns-forward-v1.json', 'scenarios/dns-reverse-v1.json', 'scenarios/etherchannel-v1.json', 'scenarios/file-backup-v1.json', 'scenarios/ipv4-static-v1.json', 'scenarios/ipv6-static-v1.json', 'scenarios/linux-access-v1.json', 'scenarios/linux-cron-v1.json', 'scenarios/linux-files-v1.json', 'scenarios/linux-find-v1.json', 'scenarios/linux-processes-v1.json', 'scenarios/mac-learning-v1.json', 'scenarios/nat-basic-v1.json', 'scenarios/nfs-v1.json', 'scenarios/ospf-p2p-v1.json', 'scenarios/rpm-packages-v1.json', 'scenarios/shell-arrays-v1.json', 'scenarios/shell-capture-v1.json', 'scenarios/shell-case-v1.json', 'scenarios/shell-command-vectors-v1.json', 'scenarios/shell-composition-v1.json', 'scenarios/shell-counts-v1.json', 'scenarios/shell-file-batches-v1.json', 'scenarios/shell-getopts-v1.json', 'scenarios/shell-numeric-validation-v1.json', 'scenarios/shell-optional-inputs-v1.json', 'scenarios/shell-reusable-v1.json', 'scenarios/shell-safe-automation-v1.json', 'scenarios/shell-scope-v1.json', 'scenarios/shell-text-v1.json', 'scenarios/shell-unit-v1.json', 'scenarios/spanning-tree-v1.json', 'scenarios/starter-v1.json', 'scenarios/storage-v1.json', 'scenarios/unit-manager-v1.json', 'scenarios/units-v1.json', 'scenarios/vlan-data-v1.json', 'src/accounts.rs', 'src/accounts_teaching.rs', 'src/acl.rs', 'src/acl_editor.rs', 'src/acl_routing.rs', 'src/acl_teaching.rs', 'src/acl_workspace.rs', 'src/address_teaching.rs', 'src/apache_static.rs', 'src/apache_static_teaching.rs', 'src/apache_static_workspace.rs', 'src/backup.rs', 'src/catalog.rs', 'src/client_teaching.rs', 'src/course.rs', 'src/cron.rs', 'src/cron_teaching.rs', 'src/cron_workspace.rs', 'src/cron_workspace/budget.rs', 'src/debian_packages.rs', 'src/debian_packages_teaching.rs', 'src/debian_packages_terminal.rs', 'src/debian_packages_terminal/session.rs', 'src/debian_packages_workspace.rs', 'src/dhcpv4.rs', 'src/dhcpv4_teaching.rs', 'src/dhcpv4_workspace.rs', 'src/discovery.rs', 'src/discovery_teaching.rs', 'src/discovery_workspace.rs', 'src/dns.rs', 'src/dns_teaching.rs', 'src/dns_workspace.rs', 'src/engine.rs', 'src/etherchannel.rs', 'src/etherchannel_teaching.rs', 'src/etherchannel_workspace.rs', 'src/file_backup.rs', 'src/file_backup_teaching.rs', 'src/file_backup_terminal.rs', 'src/file_backup_workspace.rs', 'src/file_backup_workspace/presentation.rs', 'src/filesystem.rs', 'src/filesystem/find.rs', 'src/filesystem/permissions.rs', 'src/ipv4.rs', 'src/ipv4_routing.rs', 'src/ipv4_routing_teaching.rs', 'src/ipv4_routing_workspace.rs', 'src/ipv6.rs', 'src/ipv6_teaching.rs', 'src/ipv6_workspace.rs', 'src/legacy_access_teaching.rs', 'src/legacy_logs_teaching.rs', 'src/lib.rs', 'src/local_file.rs', 'src/mac_learning.rs', 'src/mac_learning_teaching.rs', 'src/mac_learning_workspace.rs', 'src/main.rs', 'src/nat.rs', 'src/nat_teaching.rs', 'src/nat_workspace.rs', 'src/nfs.rs', 'src/nfs_teaching.rs', 'src/nfs_workspace.rs', 'src/ospf.rs', 'src/ospf_teaching.rs', 'src/ospf_workspace.rs', 'src/pack.rs', 'src/process_teaching.rs', 'src/processes.rs', 'src/progress.rs', 'src/rpm_packages.rs', 'src/rpm_packages_teaching.rs', 'src/rpm_packages_terminal.rs', 'src/rpm_packages_workspace.rs', 'src/script_teaching.rs', 'src/seed_dns_teaching.rs', 'src/seed_mount_teaching.rs', 'src/service_teaching.rs', 'src/shell.rs', 'src/shell/arithmetic.rs', 'src/shell/capture.rs', 'src/shell/case.rs', 'src/shell/cron.rs', 'src/shell/execution.rs', 'src/shell/getopts.rs', 'src/shell/globbing.rs', 'src/shell/parameters.rs', 'src/shell/preparation.rs', 'src/shell/substitution.rs', 'src/shell/text_tools.rs', 'src/shell/values.rs', 'src/spanning_tree.rs', 'src/spanning_tree_teaching.rs', 'src/spanning_tree_workspace.rs', 'src/startup_trace.rs', 'src/storage.rs', 'src/storage_teaching.rs', 'src/storage_workspace.rs', 'src/switching.rs', 'src/switching_teaching.rs', 'src/switching_workspace.rs', 'src/terminal_teaching.rs', 'src/theme.rs', 'src/ui.rs', 'src/ui/accounts_workspace.rs', 'src/ui/acl_workspace.rs', 'src/ui/address_learning.rs', 'src/ui/apache_static_workspace.rs', 'src/ui/client_learning.rs', 'src/ui/controls.rs', 'src/ui/cron_workspace.rs', 'src/ui/cron_workspace/memo.rs', 'src/ui/data.rs', 'src/ui/debian_packages_workspace.rs', 'src/ui/dhcpv4_workspace.rs', 'src/ui/discovery_workspace.rs', 'src/ui/dns_workspace.rs', 'src/ui/etherchannel_workspace.rs', 'src/ui/etherchannel_workspace/memo.rs', 'src/ui/file_backup_workspace.rs', 'src/ui/ipv4_routing_workspace.rs', 'src/ui/ipv4_workspace.rs', 'src/ui/ipv6_workspace.rs', 'src/ui/legacy_learning.rs', 'src/ui/mac_learning_workspace.rs', 'src/ui/nat_workspace.rs', 'src/ui/nat_workspace/memo.rs', 'src/ui/nfs_workspace.rs', 'src/ui/nfs_workspace/memo.rs', 'src/ui/ospf_workspace.rs', 'src/ui/process_learning.rs', 'src/ui/rpm_packages.rs', 'src/ui/rpm_packages/memo.rs', 'src/ui/script_learning.rs', 'src/ui/scripting.rs', 'src/ui/seed_learning.rs', 'src/ui/service_learning.rs', 'src/ui/smoke.rs', 'src/ui/smoke/accounts.rs', 'src/ui/smoke/accounts_lessons.rs', 'src/ui/smoke/acl.rs', 'src/ui/smoke/acl_lessons.rs', 'src/ui/smoke/apache_static_lessons.rs', 'src/ui/smoke/capture_lessons.rs', 'src/ui/smoke/case_history.rs', 'src/ui/smoke/client_learning.rs', 'src/ui/smoke/command_vectors.rs', 'src/ui/smoke/cron_lessons.rs', 'src/ui/smoke/debian_packages_lessons.rs', 'src/ui/smoke/dhcpv4.rs', 'src/ui/smoke/dhcpv4_lessons.rs', 'src/ui/smoke/dns_reverse.rs', 'src/ui/smoke/etherchannel_lessons.rs', 'src/ui/smoke/file_backup_lessons.rs', 'src/ui/smoke/find_lessons.rs', 'src/ui/smoke/legacy_terminal.rs', 'src/ui/smoke/mac_layout.rs', 'src/ui/smoke/mac_learning.rs', 'src/ui/smoke/mac_lessons.rs', 'src/ui/smoke/nat.rs', 'src/ui/smoke/nat_lessons.rs', 'src/ui/smoke/nfs_alignment.rs', 'src/ui/smoke/nfs_lessons.rs', 'src/ui/smoke/numeric_validation.rs', 'src/ui/smoke/optional_inputs.rs', 'src/ui/smoke/ospf.rs', 'src/ui/smoke/ospf_lessons.rs', 'src/ui/smoke/reopen.rs', 'src/ui/smoke/reopen/dhcpv4.rs', 'src/ui/smoke/reopen/nfs.rs', 'src/ui/smoke/rpm_lessons.rs', 'src/ui/smoke/safe_automation_lessons.rs', 'src/ui/smoke/script_feedback.rs', 'src/ui/smoke/seed_teaching.rs', 'src/ui/smoke/spanning_tree.rs', 'src/ui/smoke/spanning_tree_lessons.rs', 'src/ui/smoke/substitution.rs', 'src/ui/smoke/unit_manager.rs', 'src/ui/smoke/unit_manager_lessons.rs', 'src/ui/spanning_tree_workspace.rs', 'src/ui/storage_workspace.rs', 'src/ui/switching_workspace.rs', 'src/ui/terminal_learning.rs', 'src/ui/tests.rs', 'src/ui/tests/apache_static.rs', 'src/ui/tests/apache_static_lessons.rs', 'src/ui/tests/client_learning.rs', 'src/ui/tests/command_vectors.rs', 'src/ui/tests/cron_lessons_native.rs', 'src/ui/tests/cron_workspace.rs', 'src/ui/tests/debian_packages.rs', 'src/ui/tests/dhcpv4.rs', 'src/ui/tests/dhcpv4_lessons.rs', 'src/ui/tests/etherchannel_native.rs', 'src/ui/tests/etherchannel_workspace.rs', 'src/ui/tests/file_backup_lessons_native.rs', 'src/ui/tests/file_backup_workspace.rs', 'src/ui/tests/getopts_history.rs', 'src/ui/tests/getopts_lessons.rs', 'src/ui/tests/legacy_learning.rs', 'src/ui/tests/mac_learning.rs', 'src/ui/tests/nat.rs', 'src/ui/tests/nat_lessons.rs', 'src/ui/tests/nat_lessons_native.rs', 'src/ui/tests/nat_native.rs', 'src/ui/tests/nat_teaching.rs', 'src/ui/tests/nfs_lessons_native.rs', 'src/ui/tests/nfs_workspace.rs', 'src/ui/tests/numeric_validation.rs', 'src/ui/tests/optional_inputs.rs', 'src/ui/tests/ospf.rs', 'src/ui/tests/rpm_lessons_native.rs', 'src/ui/tests/rpm_packages.rs', 'src/ui/tests/safe_automation_lessons.rs', 'src/ui/tests/seed_learning.rs', 'src/ui/tests/spanning_tree.rs', 'src/ui/tests/ui_consistency.rs', 'src/ui/tests/unit_manager.rs', 'src/ui/unit_manager_workspace.rs', 'src/ui/workspace.rs', 'src/unit_manager.rs', 'src/unit_manager/journal.rs', 'src/unit_manager_teaching.rs', 'src/unit_manager_terminal.rs', 'src/unit_manager_terminal/session.rs', 'src/unit_manager_workspace.rs', 'src/workspace.rs', 'tests/fixtures/bash-before12-source-index.json', 'tests/fixtures/bash-before13-source-index.json', 'tests/support/actual10_evidence.rs', 'tests/support/actual11_evidence.rs', 'tests/support/actual9_evidence.rs', 'tools/automation_before10_oracle.py', 'tools/build_safety.py', 'tools/case_bash_oracle.py', 'tools/dev.py', 'tools/native_reopen.py', 'tools/parameters_before11_verify.py', 'tools/scripting_before12_verify.py', 'tools/scripting_before13_verify.py', 'tools/tests/test_scripting_before13_verify.py']
    return [(root / n, CAP) for n in names] + [(root / PRODUCER, BINARY_CAP)]


def pins(paths):
    rows = [safe.pin(p, cap) for p, cap in paths]
    safe.require(sum(p['bytes'] for p in rows) <= BINARY_CAP + TOTAL, 'Fixed source/binary aggregate')
    return rows


def current12(root):
    raw = safe.read_bytes(root / 'src/shell.rs', 512 * 1024)
    safe.require(re.findall(rb'^pub const EVALUATOR_REVISION: u32 = ([0-9]+);$', raw, re.MULTILINE) == [b'12'],
                 'New before13 capture requires actual current evaluator12 source; never relabel later semantics')


def approved(raw, control):
    value = strict(raw)
    safe.require(value.get('root_reviewed') is True and value == {**control, 'root_reviewed': True}, 'Exact Root-reviewed current12 control required')


def retained_parent(destination, name):
    path = contained(destination, name)
    parent = destination
    for position, component in enumerate(Path(name).parts[:-1]):
        if component == '.cargo':
            safe.require(position == 0 and name == '.cargo/config.toml', 'Only exact compiled .cargo/config.toml retention')
            current = parent / component
            try:
                current.mkdir(mode=0o700)
            except FileExistsError:
                pass
            info = current.lstat()
            safe.require(stat.S_ISDIR(info.st_mode) and not stat.S_ISLNK(info.st_mode)
                         and stat.S_IMODE(info.st_mode) == 0o700 and current.resolve().parent == parent.resolve(),
                         'Private regular retained .cargo directory')
            parent = current
        else:
            try:
                parent = safe.private_dir(parent, component)
            except FileExistsError:
                parent = safe.private_dir(parent, component, existing=True)
    return path


def attest(root, manifest_sha, before, executable_pin):
    folder = root / DESTINATION
    destination = safe.private_dir(folder, 'compiled-source')
    budget = safe.Budget(ATTEST_TOTAL)
    source = strict(safe.read_bytes(folder / 'SOURCE.json', CAP))
    safe.require(source['actual_evaluator_revision'] == 12 and source['source_index_sha256'] == INDEX_SHA, 'Actual producer source statement')
    for pin in source['inputs']:
        name = pin['file']
        original = contained(root, name)
        raw = safe.read_bytes(original, CAP)
        safe.require(sha(raw) == pin['sha256'] and len(raw) == pin['bytes'], 'Source changed before attestation')
        target = retained_parent(destination, name)
        safe.write_new(target, raw, budget, CAP)
    # Keep the actual12 executable itself; never rebuild an old producer under13.
    with safe.ordinary(root / PRODUCER, BINARY_CAP) as (stream, info):
        safe.require(safe.pin(root / PRODUCER, BINARY_CAP) == executable_pin, 'Actual12 producer changed')
        out = folder / 'actual12-producer'
        budget.charge(info.st_size)
        with safe.new_file(out) as target:
            left = info.st_size
            while left:
                data = stream.read(min(left, 1024 * 1024)); safe.require(data, 'Producer shrank'); target.write(data); left -= len(data)
            target.flush(); os.fsync(target.fileno())
        out.chmod(0o700)
        safe.require(safe.pin(out, BINARY_CAP)['sha256'] == executable_pin['sha256'], 'Retained actual12 producer differs')
    report = {'actual12_manifest_sha256': manifest_sha, 'producer': executable_pin, 'reviewed_pins': before,
              'records_regenerated': False, 'scope': 'Current12 actual capture; not yet sealed for future13 until Root independent review'}
    safe.json_new(folder / 'ATTESTATION.json', report, safe.Budget(CAP), CAP)


def require_seals():
    safe.require(all(isinstance(value, str) and re.fullmatch(r'[0-9a-f]{64}', value)
                     for value in [SEALED_MANIFEST, SEALED_ATTESTATION]),
                 'No genuine12 capture has been sealed yet; do not advance evaluator')


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--mode', required=True, choices=['prepare', 'capture', 'verify'])
    args = parser.parse_args(argv)
    root = ROOT.resolve(strict=True)
    safe.require((root / '.git').is_dir() and not (root / '.git').is_symlink(), 'MAIN checkout only')
    inherited = safe.require_guard(root)
    safe.require(os.environ.get('CARGO_TARGET_DIR') == str(root / 'target'), 'MAIN guarded target required')
    if args.mode == 'verify':
        require_seals()
        verify_capture(root, SEALED_MANIFEST, require_retained=True)
        attestation_raw = safe.read_bytes(root / DESTINATION / 'ATTESTATION.json', CAP)
        safe.require(sha(attestation_raw) == SEALED_ATTESTATION, 'Reviewed actual12 attestation changed')
        report = strict(attestation_raw)
        safe.require(report['actual12_manifest_sha256'] == SEALED_MANIFEST and report['records_regenerated'] is False,
                     'Exact actual12 provenance')
        retained = safe.pin(root / DESTINATION / 'actual12-producer', BINARY_CAP)
        safe.require({k: retained[k] for k in ['bytes', 'sha256']} == {k: report['producer'][k] for k in ['bytes', 'sha256']},
                     'Original actual12 executable changed')
        print(json.dumps({'status': 'SEALED-ACTUAL12-VERIFIED', 'manifest_sha256': SEALED_MANIFEST}))
        return 0
    current12(root)
    index = load_index(root)
    input11(root, index)
    paths = source_set(root); before = pins(paths); head = safe.source_identity(root)
    from build_safety import owned_directory
    base = owned_directory(root, root / HERE)
    safe.require(not (root / DESTINATION).exists(), 'Preserve existing genuine capture; no overwrite')
    control = {'schema_version': 1, 'root_reviewed': False, 'source_head': head, 'pins': before,
               'actual_evaluator_revision': 12, 'index_sha256': INDEX_SHA, 'prior_actual11_manifest_sha256': PRIOR_SHA,
               'profile_contexts': 145, 'new_script_contexts': NEW, 'claims': 'Synthetic actual12 observations, never personal progress or learner mastery'}
    if args.mode == 'prepare':
        safe.json_new(base / 'PROPOSED-before13-r1.json', control, safe.Budget(CAP), CAP)
        print(json.dumps({'status': 'PROPOSED-NO-EXECUTION'})); return 0
    approved(safe.read_bytes(base / 'APPROVED-before13-r1.json', CAP), control)
    run = safe.private_dir(base, 'before13-run-r2'); budget = safe.Budget(TOTAL)
    temporary = safe.private_dir(run, 'tmp')
    environment = safe.closed_environment({}, temporary)
    environment.update(DLMS_FIXED_CAPTURE12='before13-r1', CARGO_TARGET_DIR=str(root / 'target'))
    executable_pin = next(p for p in before if p['path'] == str(root / PRODUCER))
    with pinned_executable(root / PRODUCER, executable_pin, BINARY_CAP) as (executable, fd):
        result = safe.capture_command([executable], run, root, environment, b'', budget, 65536,
            timeout=90, inherited=(*inherited, fd))
    safe.json_new(run / 'RESULT.json', result, budget, CAP)
    safe.quiet_group()
    safe.require(result['status'] == 0 and not result['stderr'] and result['stdout'].startswith('PASS genuine actual12 before13:'),
                 'Actual current12 producer did not finish; preserve partial evidence')
    safe.require(pins(paths) == before and safe.source_identity(root) == head, 'Compiled source/binary changed during actual12 capture')
    manifest_sha = sha(safe.read_bytes(root / DESTINATION / 'MANIFEST.json', CAP))
    verify_capture(root, manifest_sha)
    attest(root, manifest_sha, before, executable_pin)
    print(json.dumps({'status': 'ACTUAL12-CAPTURED-REVIEW-AND-SEAL-REQUIRED', 'manifest_sha256': manifest_sha,
                      'records': 55, 'cases': 420, 'examples': 55, 'contexts': 145, 'draft_current_records': 53}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
