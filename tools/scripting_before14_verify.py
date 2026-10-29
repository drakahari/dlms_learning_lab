#!/usr/bin/env python3
"""Fixed genuine13 before14 development capture/attestation, not learner input.

No source/profile/path/engine CLI. Prepare launches nothing. Capture requires
Root-reviewed exact current source/binary pins and the original sealed genuine12 data.
SEALED_MANIFEST is filled only after an actual13 capture and independent review;
until then verification cannot authorize a future evaluator14 transition.
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
import scripting_before13_verify as before12
from automation_before10_oracle import pinned_executable

ROOT = Path(__file__).resolve().parents[1]
HERE = '.build-artifacts/reviews/bash-ere-integration/captures'
DESTINATION = HERE + '/public13-182-442-r1'
PRODUCER = 'target/debug/examples/scripting_before14_capture'
SOURCE = 'examples/scripting_before14_capture.rs'
INDEX = 'tests/fixtures/bash-before14-source-index.json'
INDEX_SHA = 'e44861c14f96ccb573273ea893ccee23fab9a7f96ee1af33022612f2871fc497'
PRIOR_SHA = '109d1c747c8bb9cb8f497abcc5a3e13be5de53de6447b37fccc728ec15aeb448'
SELF = 'tools/scripting_before14_verify.py'
TEST = 'tools/tests/test_scripting_before14_verify.py'
CAP = 16_000_000
BINARY_CAP = 512 * 1024 * 1024
TOTAL = 256 * 1024 * 1024
ATTEST_TOTAL = TOTAL + BINARY_CAP
SEALED_MANIFEST = '8ea1df74fdc71db3f7290e2908467c6ec4162eff7231d91440ad4131bc983441'
SEALED_ATTESTATION = '0855fd6df6c1ad0260fddc8a5c6d1f04c6e065e16d0ce8a176d44581a53b49d1'
NEW = ['shell-loop-intake', 'shell-loop-fallback']
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
    safe.require(sha(raw) == INDEX_SHA, 'Fixed57 authored input index changed')
    value = strict(raw)
    for key, expected in {'actual_evaluator_revision_required': 13, 'catalog_count': 182,
                           'script_count': 57, 'authored_cases': 442, 'separate_examples': 57,
                           'prior_profile_contexts': 145, 'expected_profile_contexts': 147}.items():
        safe.require(value[key] == expected and not isinstance(value[key], bool), 'Fixed actual13 scope: ' + key)
    safe.require(value['explicit_new_script_contexts'] == NEW and value['retained_draft_ids'] == DRAFTS,
                 'Only2 explicit new loop fixtures and2 authentic drafts')
    safe.require(value['prior_actual12_manifest_sha256'] == PRIOR_SHA
                 and len(value['catalog']) == 182 and len(value['scripts']) == 57
                 and len(value['prior_profile_ids']) == 145 and len(set(value['prior_profile_ids'])) == 145
                 and len(value['non_script_absent_not_invented']) == 35,
                 'Exact authored/legacy identity counts')
    safe.require(sum(len(s['cases']) for s in value['scripts']) == 442, 'All442 authored cases')
    safe.require(len(set(value['prior_script_ids'])) == 55 and len(set(value['prior_non_script_ids'])) == 90
                 and not set(value['prior_script_ids']) & set(value['prior_non_script_ids'])
                 and set(value['prior_script_ids']) | set(value['prior_non_script_ids']) == set(value['prior_profile_ids'])
                 and not set(NEW) & set(value['prior_profile_ids'])
                 and len(set(value['prior_profile_ids']) | set(NEW)) == value['expected_profile_contexts'],
                 'Derived55+90 genuine prior identities and only2 absent loop contexts')
    return value


def load_index(root):
    return index_value(safe.read_bytes(root / INDEX, CAP))

def input12(root, index):
    safe.require(before12.SEALED_MANIFEST == PRIOR_SHA and before12.DESTINATION == (
        '.build-artifacts/reviews/bash-loop-recovery/captures/public12-171-420-r1'), 'Fixed genuine12 verifier identity')
    before12.require_seals()
    before12.verify_capture(root, PRIOR_SHA, require_retained=True)
    prior_attestation = strict(safe.read_bytes(root / before12.DESTINATION / 'ATTESTATION.json', CAP))
    safe.require(prior_attestation['actual12_manifest_sha256'] == PRIOR_SHA
                 and prior_attestation['records_regenerated'] is False, 'Actual12 attested provenance')
    original_elf = safe.pin(root / before12.DESTINATION / 'actual12-producer', BINARY_CAP)
    safe.require({k:original_elf[k] for k in ['bytes','sha256']} ==
                 {k:prior_attestation['producer'][k] for k in ['bytes','sha256']},
                 'Exact ordinary retained actual12 producer required')
    attestation = safe.read_bytes(root / before12.DESTINATION / 'ATTESTATION.json', CAP)
    safe.require(sha(attestation) == before12.SEALED_ATTESTATION == index['prior_actual12_attestation_sha256'],
                 'Genuine12 independent attestation pin')
    result = {}
    for role in ['original', 'drafts']:
        raw = safe.read_bytes(root / before12.DESTINATION / ('writer12-' + role + '.json'), CAP)
        safe.require(sha(raw) == index['prior_' + role + '_sha256'], 'Original12 raw profile pin changed')
        value = strict(raw)
        safe.require(sorted(value['attempts']) == index['prior_profile_ids']
                     and value['migration_receipts'] == {'workspace': True, 'shell': [7, 8, 9, 10, 11, 12]},
                     'Authentic genuine12 contexts/receipts unchanged')
        scripts = {k for k, a in value['attempts'].items() if a['state']['kind'] == 'Script'}
        safe.require(scripts == set(index['prior_script_ids']) and len(scripts) == 55
                     and set(value['attempts']) - scripts == set(index['prior_non_script_ids'])
                     and len(index['prior_non_script_ids']) == 90 and not set(NEW) & set(value['attempts']),
                     'Derive context union from actual55Script/90nonScript IDs; add only absent2 loop fixtures')
        result[role] = (raw, value)
    return result


def outcome(expected, actual):
    return isinstance(actual, dict) and set(actual) == {'stdout', 'stderr', 'status', 'files', 'error'} \
        and actual['error'] is None and actual['stdout'] == expected['stdout'] \
        and actual['stderr'] == expected['stderr'] and actual['status'] == expected['status'] \
        and not isinstance(actual['status'], bool) and actual['files'] == expected['after']


def jsonl(raw):
    safe.require(raw.endswith(b'\n'), 'Complete LF-terminated observation stream')
    safe.require(b'\r' not in raw, 'Raw CR/CRLF framing is unsupported; escaped string data remains valid')
    rows = raw[:-1].decode('utf-8', errors='strict').split('\n')
    safe.require(all(rows), 'No empty observation records')
    return rows


def authored(raw, index):
    lines = jsonl(raw)
    safe.require(len(lines) == 57, 'All57 actual13 script/example observations required')
    rows = []
    for text, expected in zip(lines, index['scripts']):
        row = strict(text)
        safe.require(row['id'] == expected['id'] and row['source'] == expected['source']
                     and row['cases'] == expected['cases'] and row['example_source'] == expected['example_source']
                     and row['example_case'] == expected['example_case'] and row['example_revision'] == 13,
                     'Actual13 observation uses exact authored source/input/order')
        record = row['record']
        safe.require(set(record) == {'engine_revision', 'source', 'results'} and record['engine_revision'] == 13
                     and record['source'] == expected['source'] and len(record['results']) == len(expected['cases']),
                     'Actual13 record revision/source/case count')
        safe.require(all(outcome(c, r) for c, r in zip(expected['cases'], record['results']))
                     and outcome(expected['example_case'], row['example_outcome']),
                     'Actual13 outcomes must match every independently authored behavior')
        rows.append(row)
    return rows


def case_observations(raw, index, records):
    lines = jsonl(raw)
    safe.require(len(lines) == 499, 'Every442 assignment and57 example API outcome is retained')
    expected = []
    for row in records:
        expected.extend({'id': row['id'], 'role': 'assignment', 'case': n, 'outcome': result}
                        for n, result in enumerate(row['record']['results']))
        expected.append({'id': row['id'], 'role': 'example', 'case': 0, 'outcome': row['example_outcome']})
    actual = [strict(line) for line in lines]
    safe.require(actual == expected and sum(len(s['cases']) for s in index['scripts']) == 442,
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
    safe.require(set(current['attempts']) == expected_ids and len(expected_ids) == len(index['prior_profile_ids']) + len(NEW) == 147,
                 '145 authentic contexts plus only2 explicitly executed script fixtures')
    safe.require(set(current) == set(old), 'Preserve exact profile top-level fields')
    safe.require(current['favorites'] == old['favorites'] and current['schema_version'] == old['schema_version']
                 and current['migration_receipts'] == {'workspace': True, 'shell': [7, 8, 9, 10, 11, 12, 13]},
                 'Preserved favorites/schema and genuine13 receipt append')
    by_id = {row['id']: row for row in observations}
    skipped = set(DRAFTS) if drafts else set()
    safe.require(current['clock'] == old['clock'] + (55 if drafts else 57), 'Explicit executed fixture clock only')
    for name, before in old['attempts'].items():
        after = current['attempts'][name]
        if before['state']['kind'] != 'Script' or name in skipped:
            safe.require(after == before, 'Unchanged non-script/authentic draft: ' + name)
            continue
        row = by_id[name]
        safe.require(before['workspace']['script_learning']['example_source'] == row['example_source'],
                     'Never attach practice credit to a different preserved example draft: ' + name)
        safe.require(before['state']['source'] == row['source'],
                     'Never replace another authentic draft with reference credit: ' + name)
        safe.require(stable_metadata(after) == stable_metadata(before), 'All other learner metadata retained: ' + name)
        safe.require(records(after, False) == records(before, True), 'Every genuine6..12 record retained: ' + name)
        safe.require(after['state'] == {'kind': 'Script', 'source': row['source'], 'run': row['record']},
                     'Only actually executed13 reference supplies current credit: ' + name)
        work = after['workspace']
        safe.require(work['checked'] is False and work['script_learning']['example_revision'] == 13
                     and work['script_learning']['example_result'] == row['example_outcome'], 'Actually rerun13 example only')
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
            safe.require(work['script_learning']['example_revision'] == 13
                         and work['script_learning']['example_source'] == row['example_source']
                         and work['script_learning']['example_result'] == row['example_outcome']
                         and not any(k.startswith(('cron_', 'etherchannel_', 'nfs_', 'one_off_', 'physical_link_', 'file_backup_',
                                                         'rpm_', 'apache_static_', 'switching_', 'ipv6_', 'ospf_', 'acl_',
                                                         'unit_manager_', 'mac_', 'accounts_', 'storage_', 'debian_', 'dhcpv4_', 'ipv4_routing_')) for k in work),
                         'No fabricated other-family metadata/mastery')
    safe.require(not set(index['non_script_absent_not_invented']) & set(current['attempts']), 'Never invent35 later non-script contexts')


def source_index(root, folder, *, require_retained=False):
    """Only pre-attestation validation may use the exact current static index.

    Final sealed verification requires the ordinary retained tree and every
    SOURCE input. An absent whole tree must never fall back to current source.
    """
    frozen = folder / 'compiled-source'
    try:
        info = frozen.lstat()
    except FileNotFoundError:
        safe.require(not require_retained, 'Sealed actual13 retained compiled-source directory is required')
        return index_value(safe.read_bytes(root / INDEX, CAP))
    safe.require(stat.S_ISDIR(info.st_mode) and not stat.S_ISLNK(info.st_mode),
                 'Retained compiled-source must be an ordinary directory')
    source = strict(safe.read_bytes(folder / 'SOURCE.json', CAP))
    safe.require(source['actual_evaluator_revision'] == 13 and source['source_index_sha256'] == INDEX_SHA
                 and 1 <= len(source['inputs']) <= 1500, 'Frozen actual13 source identity')
    safe.require([p['file'] for p in source['inputs']] == SOURCE_INPUTS, 'Exact complete compiled-source inventory')
    names = set()
    for pin in source['inputs']:
        name = pin['file']
        safe.require(name not in names, 'Unique frozen source input'); names.add(name)
        observed = safe.pin(contained(frozen, name), CAP)
        safe.require({k: observed[k] for k in ['bytes', 'sha256']} == {k: pin[k] for k in ['bytes', 'sha256']},
                     'Frozen source changed: ' + name)
    return index_value(safe.read_bytes(frozen / INDEX, CAP))


def verify_capture(root, expected_manifest, *, require_retained=False):
    safe.require(isinstance(expected_manifest, str) and len(expected_manifest) == 64, 'Actual reviewed13 manifest pin required')
    folder = root / DESTINATION
    raw = safe.read_bytes(folder / 'MANIFEST.json', CAP)
    safe.require(sha(raw) == expected_manifest and safe.read_bytes(folder / 'COMPLETE', 65) == (expected_manifest + '\n').encode(),
                 'Exact actual13 manifest and completion marker')
    manifest = strict(raw)
    for key, value in {'actual_evaluator_revision': 13, 'catalog_contexts': 182, 'script_records': 57,
                       'authored_cases': 442, 'examples': 57}.items():
        safe.require(manifest[key] == value and not isinstance(manifest[key], bool), 'Closed actual13 scope')
    safe.require(manifest['prior_actual12_manifest_sha256'] == PRIOR_SHA and manifest['source_index_sha256'] == INDEX_SHA,
                 'Actual13 provenance is linked to authentic12 and static authored inputs')
    seen = set()
    for pin in manifest['outputs']:
        name = pin['file']
        safe.require(name not in seen and len(seen) < 128, 'Unique bounded capture outputs')
        seen.add(name)
        observed = safe.pin(contained(folder, name), CAP)
        safe.require({k: observed[k] for k in ['bytes', 'sha256']} == {k: pin[k] for k in ['bytes', 'sha256']},
                     'Actual13 output changed: ' + name)
    required = {'VM-OBSERVATIONS.jsonl', 'SOURCE.json', 'input12-original.json', 'input12-drafts.json', 'actual13-authored.jsonl',
                'writer13-original.json', 'writer13-drafts.json',
                'original-profile/progress-v1.json', 'original-profile/progress-v1.before-shell-v13.json',
                'draft-profile/progress-v1.json', 'draft-profile/progress-v1.before-shell-v13.json'}
    safe.require(seen == required, 'Exact closed13 capture output identities')
    index = source_index(root, folder, require_retained=require_retained)
    inputs = input12(root, index)
    observations = authored(safe.read_bytes(folder / 'actual13-authored.jsonl', CAP), index)
    case_observations(safe.read_bytes(folder / 'VM-OBSERVATIONS.jsonl', CAP), index, observations)
    for role, drafts in [('original', False), ('drafts', True)]:
        original_raw, original = inputs[role]
        safe.require(safe.read_bytes(folder / ('input12-' + role + '.json'), CAP) == original_raw, 'Actual12 inputs copied exactly')
        current_raw = safe.read_bytes(folder / ('writer13-' + role + '.json'), CAP)
        current = strict(current_raw)
        profile(original, current, index, observations, drafts)
        directory = folder / ('draft-profile' if drafts else 'original-profile')
        safe.require(safe.read_bytes(directory / 'progress-v1.json', CAP) == current_raw
                     and safe.read_bytes(directory / 'progress-v1.before-shell-v13.json', CAP) == original_raw,
                     'Current13 main/actual12 backup exact')
    return manifest


SOURCE_INPUTS = ['.cargo/config.toml', 'Cargo.lock', 'Cargo.toml', 'assets/themes/LICENSE.dlms', 'assets/themes/LICENSE.omarchy', 'assets/themes/dlms-stock.json', 'assets/themes/omarchy-catppuccin-latte.toml', 'assets/themes/omarchy-catppuccin.toml', 'assets/themes/omarchy-ethereal.toml', 'assets/themes/omarchy-everforest.toml', 'assets/themes/omarchy-flexoki-light.toml', 'assets/themes/omarchy-gruvbox.toml', 'assets/themes/omarchy-hackerman.toml', 'assets/themes/omarchy-kanagawa.toml', 'assets/themes/omarchy-last-horizon.toml', 'assets/themes/omarchy-lumon.toml', 'assets/themes/omarchy-lupine.toml', 'assets/themes/omarchy-manifest.json', 'assets/themes/omarchy-matte-black.toml', 'assets/themes/omarchy-miasma.toml', 'assets/themes/omarchy-nord.toml', 'assets/themes/omarchy-osaka-jade.toml', 'assets/themes/omarchy-retro-82.toml', 'assets/themes/omarchy-ristretto.toml', 'assets/themes/omarchy-rose-pine.toml', 'assets/themes/omarchy-solitude.toml', 'assets/themes/omarchy-tokyo-night.toml', 'assets/themes/omarchy-vantablack.toml', 'assets/themes/omarchy-white.toml', 'docs/ACCESSIBILITY.md', 'docs/ACCOUNTS_BACKEND.md', 'docs/ACCOUNTS_SUBSET.md', 'docs/ACCOUNT_LESSONS.md', 'docs/ACL_EDITOR_SUBSET.md', 'docs/ACL_LESSONS.md', 'docs/ACL_ROUTING_SUBSET.md', 'docs/ACL_SUBSET.md', 'docs/ACL_WORKSPACE.md', 'docs/ADDRESSING_TEACHING.md', 'docs/APACHE_STATIC_NATIVE.md', 'docs/APACHE_STATIC_NATIVE_ACCEPTANCE.md', 'docs/APACHE_STATIC_SUBSET.md', 'docs/APACHE_STATIC_WORKSPACE.md', 'docs/ARCHITECTURE.md', 'docs/AUTOMATION_BEFORE10_ORACLE.md', 'docs/BEFORE14_CAPTURE.md', 'docs/BUILD_SAFETY_EVIDENCE.md', 'docs/BUILD_WORKFLOW.md', 'docs/CATALOG164_MILESTONE.md', 'docs/CATALOG168_MILESTONE.md', 'docs/CATALOG171_MILESTONE.md', 'docs/CATALOG176_MILESTONE.md', 'docs/CATALOG180_MILESTONE.md', 'docs/CHECKPOINT.md', 'docs/CLIENT_NATIVE.md', 'docs/CLIENT_TEACHING.md', 'docs/COMPLETION.md', 'docs/CONTENT.md', 'docs/CONTROL_LAYOUT.md', 'docs/CRON_SUBSET.md', 'docs/CURRICULUM.csv', 'docs/CURRICULUM.md', 'docs/CURRICULUM_RECONCILIATION.md', 'docs/DEBIAN_PACKAGES_NATIVE.md', 'docs/DEBIAN_PACKAGES_NATIVE_ACCEPTANCE.md', 'docs/DEBIAN_PACKAGES_SUBSET.md', 'docs/DEBIAN_PACKAGES_TERMINAL.md', 'docs/DEBIAN_PACKAGES_WORKSPACE.md', 'docs/DEBIAN_PACKAGE_LESSONS.md', 'docs/DESIGN_REVIEW.md', 'docs/DHCPV4_LESSONS.md', 'docs/DHCPV4_LESSONS_NATIVE.md', 'docs/DHCPV4_NATIVE.md', 'docs/DHCPV4_SUBSET.md', 'docs/DHCPV4_WORKSPACE.md', 'docs/DISCOVERY_LESSONS.md', 'docs/DISCOVERY_SUBSET.md', 'docs/DISCOVERY_WORKSPACE.md', 'docs/DNS_LESSONS.md', 'docs/DNS_NO_ANSWER_SUBSET.md', 'docs/DNS_REVERSE_LESSONS.md', 'docs/DNS_REVERSE_NATIVE.md', 'docs/DNS_REVERSE_SUBSET.md', 'docs/DNS_SUBSET.md', 'docs/DNS_WORKSPACE.md', 'docs/ERE_SUBSET.md', 'docs/ETHERCHANNEL_SUBSET.md', 'docs/ETHERCHANNEL_WORKSPACE.md', 'docs/EVALUATOR8_MIGRATION_ACCEPTANCE.md', 'docs/FEDORA_065032D_VERIFICATION.md', 'docs/FEDORA_C683_VERIFICATION.md', 'docs/FILESYSTEM_SUBSET.md', 'docs/FILE_BACKUP_LESSONS.md', 'docs/FILE_BACKUP_SUBSET.md', 'docs/FIND_LESSONS.md', 'docs/FIND_SUBSET.md', 'docs/GETOPTS_LESSONS_NATIVE.md', 'docs/GETOPTS_ORACLE.md', 'docs/GETOPTS_SCOPE_ORACLE.md', 'docs/IMPLEMENTATION_LEDGER.md', 'docs/INSTALLATION.md', 'docs/IPV4_LESSONS.md', 'docs/IPV4_PRIVATE_PLANNING_LESSONS.md', 'docs/IPV4_ROUTING_SUBSET.md', 'docs/IPV6_LESSONS.md', 'docs/IPV6_SUBSET.md', 'docs/LEARNER_GUIDE.md', 'docs/LEGACY_ACCESS_LOGS_TEACHING.md', 'docs/LEGACY_TERMINAL_NATIVE.md', 'docs/LINUX_ACCESS.md', 'docs/LINUX_FILES.md', 'docs/LPIC_RECONCILIATION.md', 'docs/LVM_SUBSET.md', 'docs/MAC_LEARNING_SUBSET.md', 'docs/MAC_LESSONS.md', 'docs/NATIVE_REOPEN_DHCP.md', 'docs/NATIVE_STARTUP.md', 'docs/NAT_LESSONS.md', 'docs/NAT_NATIVE.md', 'docs/NAT_SUBSET.md', 'docs/NAT_TEACHING.md', 'docs/NAT_WORKSPACE_SUBSET.md', 'docs/NFS_LESSONS.md', 'docs/NFS_SUBSET.md', 'docs/NFS_WORKSPACE.md', 'docs/NTP_SUBSET.md', 'docs/OBJECTIVE_MAPPING.md', 'docs/OBJECTIVE_SOURCES.json', 'docs/ONE_OFF_LESSONS.md', 'docs/ONE_OFF_SUBSET.md', 'docs/ONE_OFF_WORKSPACE.md', 'docs/OSPF_LESSONS.md', 'docs/OSPF_SUBSET.md', 'docs/OSPF_WORKSPACE.md', 'docs/PHYSICAL_LINK_BACKEND.md', 'docs/PHYSICAL_LINK_LESSONS.md', 'docs/PHYSICAL_LINK_MODEL.md', 'docs/PROCESS_LESSONS.md', 'docs/PROCESS_SUBSET.md', 'docs/PROGRESS_RECOVERY.md', 'docs/RPM_NATIVE.md', 'docs/RPM_PACKAGES_WORKSPACE.md', 'docs/RPM_PACKAGE_SUBSET.md', 'docs/RPM_VERSION_REFERENCE.md', 'docs/SCRIPT_TEACHING.md', 'docs/SEED_DNS_MOUNT_TEACHING.md', 'docs/SERVICE_TEACHING.md', 'docs/SHELL_ARRAYS.md', 'docs/SHELL_ARRAY_LESSONS.md', 'docs/SHELL_CAPTURE_LESSONS.md', 'docs/SHELL_CASE.md', 'docs/SHELL_CASE_CLASSES.md', 'docs/SHELL_CASE_LESSONS.md', 'docs/SHELL_CASE_REFERENCE.md', 'docs/SHELL_COMMAND_VECTORS.md', 'docs/SHELL_COMMAND_VECTOR_LESSONS.md', 'docs/SHELL_COMPOSITION.md', 'docs/SHELL_COUNTS.md', 'docs/SHELL_EVALUATOR12_BASELINE.md', 'docs/SHELL_EXPANSION_LIMITS.md', 'docs/SHELL_FILE_BATCHES.md', 'docs/SHELL_GETOPTS.md', 'docs/SHELL_GETOPTS_LESSONS.md', 'docs/SHELL_GLOBBING.md', 'docs/SHELL_LOOP_RECOVERY_LESSONS.md', 'docs/SHELL_NUMERIC_VALIDATION_LESSONS.md', 'docs/SHELL_OPTIONAL_INPUTS_LESSONS.md', 'docs/SHELL_PARAMETER_DEFAULTS.md', 'docs/SHELL_REUSABLE.md', 'docs/SHELL_SAFE_AUTOMATION_LESSONS.md', 'docs/SHELL_SAFE_AUTOMATION_NATIVE_ACCEPTANCE.md', 'docs/SHELL_SCOPE.md', 'docs/SHELL_SUBSET.md', 'docs/SHELL_SUBSTITUTION.md', 'docs/SHELL_TEACHING_AMENDMENTS.md', 'docs/SHELL_TEXT.md', 'docs/SKILL_COVERAGE.csv', 'docs/SPANNING_TREE_LESSONS.md', 'docs/SPANNING_TREE_LESSONS_NATIVE.md', 'docs/SPANNING_TREE_SUBSET.md', 'docs/SPANNING_TREE_WORKSPACE.md', 'docs/STATIC_WEB_LESSONS.md', 'docs/STORAGE_LESSONS.md', 'docs/STORAGE_SUBSET.md', 'docs/STORAGE_WORKSPACE.md', 'docs/SWITCHING_SUBSET.md', 'docs/TEACHING_REFINEMENTS.md', 'docs/THEMES.md', 'docs/UNIT_MANAGER_INTEGRATION.md', 'docs/UNIT_MANAGER_LESSONS.md', 'docs/UNIT_MANAGER_NATIVE.md', 'docs/UNIT_MANAGER_SUBSET.md', 'docs/UNIT_MANAGER_TERMINAL.md', 'docs/UNIT_MANAGER_WORKSPACE.md', 'docs/VERIFICATION.md', 'docs/VLAN_LESSONS.md', 'docs/decisions/LICENSE.md', 'examples/automation_before10_capture.rs', 'examples/file_backup_probe.rs', 'examples/filesystem_probe.rs', 'examples/getopts_before9_capture.rs', 'examples/ipv4_route_probe.rs', 'examples/ipv6_probe.rs', 'examples/loop_control_reference.rs', 'examples/optional_inputs_reference.rs', 'examples/parameters_before11_capture.rs', 'examples/scripting_before12_capture.rs', 'examples/scripting_before13_capture.rs', 'examples/scripting_before14_capture.rs', 'examples/shell_probe.rs', 'examples/storage_probe.rs', 'examples/support/automation_attestation.rs', 'examples/support/automation_digest.rs', 'examples/support/getopts_attestation.rs', 'examples/support/getopts_digest.rs', 'scenarios/accounts-lifecycle-v1.json', 'scenarios/acl-policy-v1.json', 'scenarios/apache-static-v1.json', 'scenarios/debian-packages-v1.json', 'scenarios/dhcpv4-client-v1.json', 'scenarios/discovery-v1.json', 'scenarios/dns-forward-v1.json', 'scenarios/dns-reverse-v1.json', 'scenarios/etherchannel-v1.json', 'scenarios/file-backup-v1.json', 'scenarios/ipv4-private-planning-v1.json', 'scenarios/ipv4-static-v1.json', 'scenarios/ipv6-static-v1.json', 'scenarios/linux-access-v1.json', 'scenarios/linux-cron-v1.json', 'scenarios/linux-files-v1.json', 'scenarios/linux-find-v1.json', 'scenarios/linux-one-off-v1.json', 'scenarios/linux-processes-v1.json', 'scenarios/mac-learning-v1.json', 'scenarios/nat-basic-v1.json', 'scenarios/nfs-v1.json', 'scenarios/ospf-p2p-v1.json', 'scenarios/physical-links-v1.json', 'scenarios/rpm-packages-v1.json', 'scenarios/shell-arrays-v1.json', 'scenarios/shell-capture-v1.json', 'scenarios/shell-case-v1.json', 'scenarios/shell-command-vectors-v1.json', 'scenarios/shell-composition-v1.json', 'scenarios/shell-counts-v1.json', 'scenarios/shell-file-batches-v1.json', 'scenarios/shell-getopts-v1.json', 'scenarios/shell-loop-recovery-v1.json', 'scenarios/shell-numeric-validation-v1.json', 'scenarios/shell-optional-inputs-v1.json', 'scenarios/shell-reusable-v1.json', 'scenarios/shell-safe-automation-v1.json', 'scenarios/shell-scope-v1.json', 'scenarios/shell-text-v1.json', 'scenarios/shell-unit-v1.json', 'scenarios/spanning-tree-v1.json', 'scenarios/starter-v1.json', 'scenarios/storage-v1.json', 'scenarios/unit-manager-v1.json', 'scenarios/units-v1.json', 'scenarios/vlan-data-v1.json', 'src/accounts.rs', 'src/accounts_teaching.rs', 'src/acl.rs', 'src/acl_editor.rs', 'src/acl_routing.rs', 'src/acl_teaching.rs', 'src/acl_workspace.rs', 'src/address_teaching.rs', 'src/apache_static.rs', 'src/apache_static_teaching.rs', 'src/apache_static_workspace.rs', 'src/backup.rs', 'src/catalog.rs', 'src/client_teaching.rs', 'src/course.rs', 'src/cron.rs', 'src/cron_teaching.rs', 'src/cron_workspace.rs', 'src/cron_workspace/budget.rs', 'src/debian_packages.rs', 'src/debian_packages_teaching.rs', 'src/debian_packages_terminal.rs', 'src/debian_packages_terminal/session.rs', 'src/debian_packages_workspace.rs', 'src/dhcpv4.rs', 'src/dhcpv4_teaching.rs', 'src/dhcpv4_workspace.rs', 'src/discovery.rs', 'src/discovery_teaching.rs', 'src/discovery_workspace.rs', 'src/dns.rs', 'src/dns_teaching.rs', 'src/dns_workspace.rs', 'src/engine.rs', 'src/ere.rs', 'src/etherchannel.rs', 'src/etherchannel_teaching.rs', 'src/etherchannel_workspace.rs', 'src/file_backup.rs', 'src/file_backup_teaching.rs', 'src/file_backup_terminal.rs', 'src/file_backup_workspace.rs', 'src/file_backup_workspace/presentation.rs', 'src/filesystem.rs', 'src/filesystem/find.rs', 'src/filesystem/permissions.rs', 'src/ipv4.rs', 'src/ipv4_routing.rs', 'src/ipv4_routing_teaching.rs', 'src/ipv4_routing_workspace.rs', 'src/ipv6.rs', 'src/ipv6_teaching.rs', 'src/ipv6_workspace.rs', 'src/legacy_access_teaching.rs', 'src/legacy_logs_teaching.rs', 'src/lib.rs', 'src/local_file.rs', 'src/lvm.rs', 'src/mac_learning.rs', 'src/mac_learning_teaching.rs', 'src/mac_learning_workspace.rs', 'src/main.rs', 'src/nat.rs', 'src/nat_teaching.rs', 'src/nat_workspace.rs', 'src/native_observations.rs', 'src/nfs.rs', 'src/nfs_teaching.rs', 'src/nfs_workspace.rs', 'src/ntp.rs', 'src/one_off.rs', 'src/one_off_teaching.rs', 'src/one_off_workspace.rs', 'src/one_off_workspace/budget.rs', 'src/one_off_workspace/decode.rs', 'src/ospf.rs', 'src/ospf_teaching.rs', 'src/ospf_workspace.rs', 'src/pack.rs', 'src/physical_link.rs', 'src/physical_link_assignment.rs', 'src/physical_link_teaching.rs', 'src/physical_link_workspace.rs', 'src/process_teaching.rs', 'src/processes.rs', 'src/progress.rs', 'src/rpm_packages.rs', 'src/rpm_packages_teaching.rs', 'src/rpm_packages_terminal.rs', 'src/rpm_packages_workspace.rs', 'src/script_teaching.rs', 'src/seed_dns_teaching.rs', 'src/seed_mount_teaching.rs', 'src/service_teaching.rs', 'src/shell.rs', 'src/shell/arithmetic.rs', 'src/shell/capture.rs', 'src/shell/case.rs', 'src/shell/cron.rs', 'src/shell/execution.rs', 'src/shell/getopts.rs', 'src/shell/globbing.rs', 'src/shell/loop_control.rs', 'src/shell/one_off.rs', 'src/shell/parameters.rs', 'src/shell/preparation.rs', 'src/shell/substitution.rs', 'src/shell/text_tools.rs', 'src/shell/values.rs', 'src/spanning_tree.rs', 'src/spanning_tree_teaching.rs', 'src/spanning_tree_workspace.rs', 'src/startup_trace.rs', 'src/storage.rs', 'src/storage_teaching.rs', 'src/storage_workspace.rs', 'src/switching.rs', 'src/switching_teaching.rs', 'src/switching_workspace.rs', 'src/terminal_teaching.rs', 'src/theme.rs', 'src/ui.rs', 'src/ui/accounts_workspace.rs', 'src/ui/acl_workspace.rs', 'src/ui/address_learning.rs', 'src/ui/apache_static_workspace.rs', 'src/ui/client_learning.rs', 'src/ui/completion_index.rs', 'src/ui/completion_index/tests.rs', 'src/ui/controls.rs', 'src/ui/cron_workspace.rs', 'src/ui/cron_workspace/memo.rs', 'src/ui/data.rs', 'src/ui/debian_packages_workspace.rs', 'src/ui/dhcpv4_workspace.rs', 'src/ui/discovery_workspace.rs', 'src/ui/dns_workspace.rs', 'src/ui/etherchannel_workspace.rs', 'src/ui/etherchannel_workspace/memo.rs', 'src/ui/file_backup_workspace.rs', 'src/ui/ipv4_routing_workspace.rs', 'src/ui/ipv4_workspace.rs', 'src/ui/ipv6_workspace.rs', 'src/ui/legacy_learning.rs', 'src/ui/mac_learning_workspace.rs', 'src/ui/nat_workspace.rs', 'src/ui/nat_workspace/memo.rs', 'src/ui/nfs_workspace.rs', 'src/ui/nfs_workspace/memo.rs', 'src/ui/one_off_boundary.rs', 'src/ui/one_off_workspace.rs', 'src/ui/one_off_workspace/input.rs', 'src/ui/one_off_workspace/memo.rs', 'src/ui/ospf_workspace.rs', 'src/ui/physical_link_workspace.rs', 'src/ui/physical_link_workspace/memo.rs', 'src/ui/physical_link_workspace/tests.rs', 'src/ui/process_learning.rs', 'src/ui/rpm_packages.rs', 'src/ui/rpm_packages/memo.rs', 'src/ui/script_learning.rs', 'src/ui/scripting.rs', 'src/ui/seed_learning.rs', 'src/ui/service_learning.rs', 'src/ui/smoke.rs', 'src/ui/smoke/accounts.rs', 'src/ui/smoke/accounts_lessons.rs', 'src/ui/smoke/acl.rs', 'src/ui/smoke/acl_lessons.rs', 'src/ui/smoke/apache_static_lessons.rs', 'src/ui/smoke/capture_lessons.rs', 'src/ui/smoke/case_history.rs', 'src/ui/smoke/client_learning.rs', 'src/ui/smoke/command_vectors.rs', 'src/ui/smoke/cron_lessons.rs', 'src/ui/smoke/debian_packages_lessons.rs', 'src/ui/smoke/dhcpv4.rs', 'src/ui/smoke/dhcpv4_lessons.rs', 'src/ui/smoke/dns_reverse.rs', 'src/ui/smoke/etherchannel_lessons.rs', 'src/ui/smoke/file_backup_lessons.rs', 'src/ui/smoke/find_lessons.rs', 'src/ui/smoke/legacy_terminal.rs', 'src/ui/smoke/loop_recovery.rs', 'src/ui/smoke/mac_layout.rs', 'src/ui/smoke/mac_learning.rs', 'src/ui/smoke/mac_lessons.rs', 'src/ui/smoke/nat.rs', 'src/ui/smoke/nat_lessons.rs', 'src/ui/smoke/nfs_alignment.rs', 'src/ui/smoke/nfs_lessons.rs', 'src/ui/smoke/numeric_validation.rs', 'src/ui/smoke/one_off_lessons.rs', 'src/ui/smoke/optional_inputs.rs', 'src/ui/smoke/ospf.rs', 'src/ui/smoke/ospf_lessons.rs', 'src/ui/smoke/physical_link_lessons.rs', 'src/ui/smoke/private_office.rs', 'src/ui/smoke/reopen.rs', 'src/ui/smoke/reopen/dhcpv4.rs', 'src/ui/smoke/reopen/nfs.rs', 'src/ui/smoke/rpm_lessons.rs', 'src/ui/smoke/safe_automation_lessons.rs', 'src/ui/smoke/script_feedback.rs', 'src/ui/smoke/seed_teaching.rs', 'src/ui/smoke/spanning_tree.rs', 'src/ui/smoke/spanning_tree_lessons.rs', 'src/ui/smoke/substitution.rs', 'src/ui/smoke/unit_manager.rs', 'src/ui/smoke/unit_manager_lessons.rs', 'src/ui/spanning_tree_workspace.rs', 'src/ui/storage_workspace.rs', 'src/ui/switching_workspace.rs', 'src/ui/terminal_learning.rs', 'src/ui/tests.rs', 'src/ui/tests/apache_static.rs', 'src/ui/tests/apache_static_lessons.rs', 'src/ui/tests/client_learning.rs', 'src/ui/tests/command_vectors.rs', 'src/ui/tests/completion_index.rs', 'src/ui/tests/cron_lessons_native.rs', 'src/ui/tests/cron_workspace.rs', 'src/ui/tests/debian_packages.rs', 'src/ui/tests/dhcpv4.rs', 'src/ui/tests/dhcpv4_lessons.rs', 'src/ui/tests/etherchannel_native.rs', 'src/ui/tests/etherchannel_workspace.rs', 'src/ui/tests/file_backup_lessons_native.rs', 'src/ui/tests/file_backup_workspace.rs', 'src/ui/tests/getopts_history.rs', 'src/ui/tests/getopts_lessons.rs', 'src/ui/tests/legacy_learning.rs', 'src/ui/tests/loop_recovery.rs', 'src/ui/tests/mac_learning.rs', 'src/ui/tests/nat.rs', 'src/ui/tests/nat_lessons.rs', 'src/ui/tests/nat_lessons_native.rs', 'src/ui/tests/nat_native.rs', 'src/ui/tests/nat_teaching.rs', 'src/ui/tests/nfs_lessons_native.rs', 'src/ui/tests/nfs_workspace.rs', 'src/ui/tests/numeric_validation.rs', 'src/ui/tests/one_off_admission.rs', 'src/ui/tests/one_off_lessons_native.rs', 'src/ui/tests/one_off_physical.rs', 'src/ui/tests/one_off_workspace.rs', 'src/ui/tests/optional_inputs.rs', 'src/ui/tests/ospf.rs', 'src/ui/tests/physical_native_controller.rs', 'src/ui/tests/physical_normal_catalog.rs', 'src/ui/tests/physical_workspace.rs', 'src/ui/tests/private_office.rs', 'src/ui/tests/rpm_lessons_native.rs', 'src/ui/tests/rpm_packages.rs', 'src/ui/tests/safe_automation_lessons.rs', 'src/ui/tests/seed_learning.rs', 'src/ui/tests/spanning_tree.rs', 'src/ui/tests/ui_consistency.rs', 'src/ui/tests/unit_manager.rs', 'src/ui/unit_manager_workspace.rs', 'src/ui/workspace.rs', 'src/unit_manager.rs', 'src/unit_manager/journal.rs', 'src/unit_manager_teaching.rs', 'src/unit_manager_terminal.rs', 'src/unit_manager_terminal/session.rs', 'src/unit_manager_workspace.rs', 'src/workspace.rs', 'tests/fixtures/accounts-content-review.json', 'tests/fixtures/accounts-workspace-preview.json', 'tests/fixtures/acl-content-review.json', 'tests/fixtures/acl-workspace-preview.json', 'tests/fixtures/apache-static-workspace-preview.json', 'tests/fixtures/automation-bash539-observations.json', 'tests/fixtures/automation-before10-authored-inputs.json', 'tests/fixtures/automation-before10-source-pins.json', 'tests/fixtures/automation-oracle-inputs.json', 'tests/fixtures/automation-vm-boundaries.json', 'tests/fixtures/bash-before12-source-index.json', 'tests/fixtures/bash-before13-source-index.json', 'tests/fixtures/bash-before14-source-index.json', 'tests/fixtures/bash-cat-reference.json', 'tests/fixtures/bash-colon-reference.json', 'tests/fixtures/bash-command-vector-lessons-reference.json', 'tests/fixtures/bash-command-vectors-reference.json', 'tests/fixtures/bash-parameter-arithmetic-amendment.json', 'tests/fixtures/bash-parameter-arithmetic-reference.json', 'tests/fixtures/bash-parameter-defaults-reference.json', 'tests/fixtures/bash-reference.json', 'tests/fixtures/catalog133-original-index.json', 'tests/fixtures/catalog157-original154-boundary.json', 'tests/fixtures/catalog160-registration.json', 'tests/fixtures/catalog164-registration.json', 'tests/fixtures/catalog168-registration.json', 'tests/fixtures/catalog171-registration.json', 'tests/fixtures/catalog175-registration.json', 'tests/fixtures/catalog176-registration.json', 'tests/fixtures/catalog180-registration.json', 'tests/fixtures/catalog182-registration.json', 'tests/fixtures/client-original-seeds.json', 'tests/fixtures/cron-lessons-review-v1.json', 'tests/fixtures/cron-lessons-rows-v1.json', 'tests/fixtures/cron-lessons-v1.json', 'tests/fixtures/cron-original154-source-index.json', 'tests/fixtures/cron-registration-v1.json', 'tests/fixtures/cron-workbench-v1.json', 'tests/fixtures/debian-packages-declared.json', 'tests/fixtures/debian-packages-native-review.json', 'tests/fixtures/debian-packages-original125.json', 'tests/fixtures/debian-packages-workspace-preview.json', 'tests/fixtures/dhcpv4-content-review.json', 'tests/fixtures/dhcpv4-original119.json', 'tests/fixtures/dhcpv4-original122.json', 'tests/fixtures/dhcpv4-workspace-preview.json', 'tests/fixtures/discovery-workspace-preview.json', 'tests/fixtures/dns-reverse-content-proposal.json', 'tests/fixtures/dns-reverse-content-review.json', 'tests/fixtures/dns-reverse-original116.json', 'tests/fixtures/dns-workspace-preview.json', 'tests/fixtures/ere-fixed-reference.json', 'tests/fixtures/etherchannel-content-review.json', 'tests/fixtures/etherchannel-vectors-v1.json', 'tests/fixtures/etherchannel-workbench-v1.json', 'tests/fixtures/file-backup-content-review.json', 'tests/fixtures/file-backup-native-rows-v1.json', 'tests/fixtures/filesystem-find-reference.json', 'tests/fixtures/filesystem-reference.json', 'tests/fixtures/getopts-bash539-observations.json', 'tests/fixtures/getopts-before9-authored-inputs.json', 'tests/fixtures/getopts-before9-source-pins.json', 'tests/fixtures/getopts-oracle-inputs.json', 'tests/fixtures/getopts-scope-oracle-inputs.json', 'tests/fixtures/ipv4-private-planning-review.json', 'tests/fixtures/ipv4-routing-preview.json', 'tests/fixtures/legacy-access-logs-original116.json', 'tests/fixtures/linux-find-content-review.json', 'tests/fixtures/linux-find-original-catalog.json', 'tests/fixtures/linux-find-unit-proposal.json', 'tests/fixtures/loop-units-before180.json', 'tests/fixtures/mac-content-review.json', 'tests/fixtures/mac-learning-preview.json', 'tests/fixtures/mac-original-catalog.json', 'tests/fixtures/mac-unit-proposal.json', 'tests/fixtures/nat-address-vectors.json', 'tests/fixtures/nat-basic-outcomes.json', 'tests/fixtures/nat-basic-review.json', 'tests/fixtures/nat-original140-source-index.json', 'tests/fixtures/nat-teaching-preview.json', 'tests/fixtures/nat-workspace-preview.json', 'tests/fixtures/nfs-original147-source-index.json', 'tests/fixtures/nfs-review.json', 'tests/fixtures/nfs-workbench-v1.json', 'tests/fixtures/one-off-lessons-review-v1.json', 'tests/fixtures/one-off-unit-proposal.json', 'tests/fixtures/one-off-units-before176.json', 'tests/fixtures/ospf-content-review.json', 'tests/fixtures/ospf-workspace-preview.json', 'tests/fixtures/physical-link-author-review.json', 'tests/fixtures/private-office-units-before171.json', 'tests/fixtures/private-office-units-before175.json', 'tests/fixtures/process-content-review.json', 'tests/fixtures/process-workspace-preview.json', 'tests/fixtures/progress-v1.json', 'tests/fixtures/rpm-original143-source-index.json', 'tests/fixtures/rpm-package-vectors.json', 'tests/fixtures/rpm-packages-review.json', 'tests/fixtures/rpm-workspace-preview.json', 'tests/fixtures/rpmsort-version-inputs.json', 'tests/fixtures/rpmsort-version-nvr-inputs.json', 'tests/fixtures/seed-teaching-original-nine.json', 'tests/fixtures/shell-capture-content-review.json', 'tests/fixtures/shell-capture-unit-proposal.json', 'tests/fixtures/shell-case-bash539.json', 'tests/fixtures/shell-case-class-amendment-v12.json', 'tests/fixtures/shell-case-class-inputs-v12.json', 'tests/fixtures/shell-case-content-review.json', 'tests/fixtures/shell-case-oracle-inputs.json', 'tests/fixtures/shell-case-original-catalog.json', 'tests/fixtures/shell-case-unit-proposal.json', 'tests/fixtures/shell-command-vectors-review.json', 'tests/fixtures/shell-content-review.json', 'tests/fixtures/shell-getopts-content-review.json', 'tests/fixtures/shell-getopts-original-catalog.json', 'tests/fixtures/shell-getopts-unit-proposal.json', 'tests/fixtures/shell-loop-control-fixed.json', 'tests/fixtures/shell-loop-recovery-native-review.json', 'tests/fixtures/shell-loop-recovery-review.json', 'tests/fixtures/shell-numeric-validation-native-review.json', 'tests/fixtures/shell-numeric-validation-review.json', 'tests/fixtures/shell-optional-inputs-native-review.json', 'tests/fixtures/shell-optional-inputs-reference117.json', 'tests/fixtures/shell-optional-inputs-registration.json', 'tests/fixtures/shell-optional-inputs-review.json', 'tests/fixtures/shell-safe-automation-native-review.json', 'tests/fixtures/shell-substitution-excluded-diagnostic.json', 'tests/fixtures/shell-substitution-reference.json', 'tests/fixtures/shell-substitution-vm-only.json', 'tests/fixtures/shell-teaching-amendment-v1.json', 'tests/fixtures/shell-v6-authored-records.json', 'tests/fixtures/shell-v6-baseline-manifest.json', 'tests/fixtures/shell-v6-resource-before-substitution.json', 'tests/fixtures/shell-v6-substitution-before.json', 'tests/fixtures/shell-v7-fixture-provenance.json', 'tests/fixtures/shell-v7-synthetic-progress-original.json', 'tests/fixtures/shell-v7-synthetic-writer-progress.json', 'tests/fixtures/shell-v8-synthetic-writer-progress.json', 'tests/fixtures/shell-v8-synthetic-writer-provenance.json', 'tests/fixtures/spanning-tree-native-review.json', 'tests/fixtures/spanning-tree-original125.json', 'tests/fixtures/spanning-tree-workspace-preview.json', 'tests/fixtures/storage-content-cases.json', 'tests/fixtures/storage-reference.json', 'tests/fixtures/storage-workspace-preview.json', 'tests/fixtures/switching-preview.json', 'tests/fixtures/unit-manager-content-review.json', 'tests/fixtures/unit-manager-pack-preview.json', 'tests/fixtures/unit-manager-prior-98-ids.json', 'tests/fixtures/unit-manager-workspace-preview.json', 'tests/support/actual10_evidence.rs', 'tests/support/actual11_evidence.rs', 'tests/support/actual12_evidence.rs', 'tests/support/actual9_evidence.rs', 'tests/support/automation_comparison.rs', 'tests/support/combined_catalog.rs', 'tests/support/cron_lessons.rs', 'tests/support/cron_registration.rs', 'tests/support/file_backup_authored.rs', 'tests/support/loop_registration.rs', 'tests/support/mod.rs', 'tests/support/one_off_registration.rs', 'tests/support/parameter_arithmetic_amendment.rs', 'tests/support/physical_assignment_fixture.rs', 'tests/support/physical_scenario.rs', 'tests/support/switching.rs', 'tests/support/teaching_amendment.rs', 'tools/automation_before10_oracle.py', 'tools/build_safety.py', 'tools/case_bash_oracle.py', 'tools/dev.py', 'tools/native_reopen.py', 'tools/parameters_before11_verify.py', 'tools/scripting_before12_verify.py', 'tools/scripting_before13_verify.py', 'tools/scripting_before14_verify.py', 'tools/tests/test_scripting_before13_verify.py', 'tools/tests/test_scripting_before14_verify.py']

def source_set(root):
    result = [(root / name, CAP) for name in SOURCE_INPUTS]
    parent = root / before12.DESTINATION
    manifest = strict(safe.read_bytes(parent / 'MANIFEST.json', CAP))
    source = strict(safe.read_bytes(parent / 'SOURCE.json', CAP))
    safe.require(sha(safe.read_bytes(parent / 'MANIFEST.json', CAP)) == PRIOR_SHA,
                 'Sealed parent manifest before source inventory')
    for name in ['MANIFEST.json','COMPLETE','ATTESTATION.json']:
        result.append((parent / name, CAP))
    for row in manifest['outputs']:
        result.append((contained(parent,row['file']),CAP))
    for row in source['inputs']:
        result.append((contained(parent / 'compiled-source',row['file']),CAP))
    result.append((parent / 'actual12-producer',BINARY_CAP))
    toolchain=root / '.build-artifacts/toolchains/rustup/toolchains/stable-x86_64-unknown-linux-gnu'
    result.extend([(toolchain / 'bin/rustc',128*1024*1024),
                   (toolchain / 'lib/rustlib/multirust-channel-manifest.toml',CAP),
                   (root / '.build-artifacts/toolchains/READY.json',CAP),
                   (Path(__import__('sys').executable).resolve(strict=True),128*1024*1024),
                   (root / PRODUCER,BINARY_CAP)])
    safe.require(len(result) <= 2500 and len({str(p) for p,_ in result}) == len(result),
                 'Bounded unique current/parent/compiler/probe source inventory')
    return result



def pins(paths):
    rows = [safe.pin(p, cap) for p, cap in paths]
    safe.require(sum(p['bytes'] for p in rows) <= BINARY_CAP + TOTAL, 'Fixed source/binary aggregate')
    return rows


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
        loose = safe.read_bytes(contained(root / '.git', requested), 4096)
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



def current13(root):
    raw = safe.read_bytes(root / 'src/shell.rs', 512 * 1024)
    safe.require(re.findall(rb'^pub const EVALUATOR_REVISION: u32 = ([0-9]+);$', raw, re.MULTILINE) == [b'13'],
                 'New before14 capture requires actual current evaluator13 source; never relabel later semantics')


def approved(raw, control):
    value = strict(raw)
    safe.require(value.get('root_reviewed') is True and value == {**control, 'root_reviewed': True}, 'Exact Root-reviewed current13 control required')


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


def attest(root, manifest_sha, before, executable_pin, held_fd):
    folder = root / DESTINATION
    destination = safe.private_dir(folder, 'compiled-source')
    budget = safe.Budget(ATTEST_TOTAL)
    source = strict(safe.read_bytes(folder / 'SOURCE.json', CAP))
    safe.require(source['actual_evaluator_revision'] == 13 and source['source_index_sha256'] == INDEX_SHA,
                 'Actual producer source statement')
    safe.require([p['file'] for p in source['inputs']] == SOURCE_INPUTS,
                 'Every source input retained, no omitted source directory')
    for pin in source['inputs']:
        name = pin['file']; raw = safe.read_bytes(contained(root,name),CAP)
        safe.require(sha(raw)==pin['sha256'] and len(raw)==pin['bytes'],'Source changed before attestation')
        safe.write_new(retained_parent(destination,name),raw,budget,CAP)
    out=folder / 'actual13-producer'; seen=hashlib.sha256(); count=0
    with safe.new_file(out) as target:
        while count<executable_pin['bytes']:
            block=os.pread(held_fd,min(65536,executable_pin['bytes']-count),count)
            safe.require(block,'Held producer shrank'); budget.charge(len(block))
            target.write(block); seen.update(block); count+=len(block)
        safe.require(not os.pread(held_fd,1,count) and seen.hexdigest()==executable_pin['sha256'],
                     'Exact executed descriptor retained')
        target.flush(); os.fsync(target.fileno())
    report={'actual13_manifest_sha256':manifest_sha,'producer':executable_pin,'reviewed_pins':before,
            'records_regenerated':False,'scope':'Synthetic genuine13 observations; independent sealing still required'}
    safe.json_new(folder/'ATTESTATION.json',report,safe.Budget(CAP),CAP)



def require_seals():
    safe.require(all(isinstance(value, str) and re.fullmatch(r'[0-9a-f]{64}', value)
                     for value in [SEALED_MANIFEST, SEALED_ATTESTATION]),
                 'No genuine13 capture has been sealed yet; do not advance evaluator')


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
        safe.require(sha(attestation_raw) == SEALED_ATTESTATION, 'Reviewed actual13 attestation changed')
        report = strict(attestation_raw)
        safe.require(report['actual13_manifest_sha256'] == SEALED_MANIFEST and report['records_regenerated'] is False,
                     'Exact actual13 provenance')
        retained = safe.pin(root / DESTINATION / 'actual13-producer', BINARY_CAP)
        safe.require({k: retained[k] for k in ['bytes', 'sha256']} == {k: report['producer'][k] for k in ['bytes', 'sha256']},
                     'Original actual13 executable changed')
        print(json.dumps({'status': 'SEALED-ACTUAL13-VERIFIED', 'manifest_sha256': SEALED_MANIFEST}))
        return 0
    current13(root)
    index = load_index(root)
    input12(root, index)
    paths = source_set(root); before = pins(paths); head = source_identity(root)
    from build_safety import owned_directory
    base = owned_directory(root, root / HERE)
    safe.require(not (root / DESTINATION).exists(), 'Preserve existing genuine capture; no overwrite')
    control = {'schema_version': 1, 'root_reviewed': False, 'source_head': head, 'pins': before,
               'actual_evaluator_revision': 13, 'index_sha256': INDEX_SHA, 'prior_actual12_manifest_sha256': PRIOR_SHA,
               'profile_contexts': 147, 'new_script_contexts': NEW, 'claims': 'Synthetic actual13 observations, never personal progress or learner mastery'}
    if args.mode == 'prepare':
        safe.json_new(base / 'PROPOSED-before14-r1.json', control, safe.Budget(CAP), CAP)
        print(json.dumps({'status': 'PROPOSED-NO-EXECUTION'})); return 0
    approved(safe.read_bytes(base / 'APPROVED-before14-r1.json', CAP), control)
    run = safe.private_dir(base, 'before14-run-r1'); budget = safe.Budget(TOTAL)
    temporary = safe.private_dir(run, 'tmp')
    environment = safe.closed_environment({}, temporary)
    environment.update(DLMS_FIXED_CAPTURE13='before14-r1', CARGO_TARGET_DIR=str(root / 'target'))
    executable_pin = next(p for p in before if p['path'] == str(root / PRODUCER))
    with pinned_executable(root / PRODUCER, executable_pin, BINARY_CAP) as (executable, fd):
        try:
            result = safe.capture_command([executable], run, root, environment, b'', budget, 65536,
                timeout=120, inherited=(*inherited, fd))
        except Exception as error:
            safe.json_new(run/'STATUS.json',{'status':None,'complete':False,'error':str(error)[:2000]},budget,CAP)
            raise
        safe.json_new(run/'STATUS.json',{'status':result['status'],'complete':True},budget,CAP)
        safe.json_new(run/'RESULT.json',result,budget,CAP)
        safe.quiet_group()
        safe.require(result['status']==0 and not result['stderr'] and result['stdout'].startswith('PASS genuine actual13 before14:'),
                     'Actual13 producer did not finish; preserve partial evidence')
        safe.require(pins(paths)==before and source_identity(root)==head,
                     'Compiled source/binary changed during actual13 capture')
        manifest_sha=sha(safe.read_bytes(root/DESTINATION/'MANIFEST.json',CAP))
        verify_capture(root,manifest_sha)
        attest(root,manifest_sha,before,executable_pin,fd)
        verify_capture(root,manifest_sha,require_retained=True)
    print(json.dumps({'status': 'ACTUAL13-CAPTURED-REVIEW-AND-SEAL-REQUIRED', 'manifest_sha256': manifest_sha,
                      'records': 57, 'cases': 442, 'examples': 57, 'contexts': 147, 'draft_current_records': 55}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
