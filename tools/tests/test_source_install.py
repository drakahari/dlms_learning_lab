"""Source unit authoring/retention guards; typed behavior has Rust/C comparisons."""
import hashlib
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
IDS = ['lpic2-source-project-build', 'lpic2-source-stage-install', 'lpic2-source-stale-handover']


def read(name):
    return json.loads((ROOT / name).read_text())


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()


class SourceInstallTeaching(unittest.TestCase):
    def test_preserves_actual406_scenario_presentation_and_unit_origins(self):
        old = read('scenarios/boot-entry-lessons-v18.json')['scenarios']
        current = read('scenarios/boot-entry-lessons-v19.json')['scenarios']
        self.assertEqual(len(old), 33)
        self.assertEqual(digest(old), 'fca07e6fa28af072651a3411c34e116f0b1fe50e042a024eec126ef3c425fa95')
        self.assertEqual(current[:33], old)
        self.assertEqual([s['id'] for s in current[33:]], IDS)
        rows = read('scenarios/presentation/lpic2.json')
        self.assertEqual(len(rows), 135)
        self.assertEqual(digest(rows[:100]), 'f849d2bf7b0f1221f2b593bc117d85a49a40696af5959f96f9d7686791f0a27a')
        metadata = (ROOT / 'src/course/metadata.rs').read_bytes().split(b'\npub(super) fn current409_index', 1)[0]
        self.assertEqual(hashlib.sha256(metadata).hexdigest(), '76afed19bebe1603e6d9d696bf3b04c23358bd19ca395e86b33cdbefa8b8f5b8')

    def test_build_stage_and_stale_handover_are_distinct_assignments_with_real_prerequisites(self):
        unit = read('scenarios/source-installation-units-v1.json')['units'][0]
        self.assertEqual(unit['labs'], IDS)
        self.assertEqual(unit['prerequisites'], ['backup-and-recovery', 'debian-package-administration'])
        labs = read('scenarios/boot-entry-lessons-v19.json')['scenarios'][-3:]
        tasks = [s['goal']['definition']['initial'] for s in labs]
        self.assertEqual([t['destination']['kind'] for t in tasks], ['build', 'staged', 'staged'])
        self.assertEqual(tasks[0]['starter'], [])
        self.assertEqual(tasks[1]['starter'], [])
        self.assertIn('make DESTDIR=/review/release install', tasks[2]['starter'])
        self.assertTrue(tasks[2]['starter'][-1].startswith('./configure '))
        self.assertNotIn('make check', tasks[2]['starter'][-1:])
        for lab in labs:
            definition = lab['goal']['definition']
            self.assertNotEqual(definition['initial']['required'], definition['example']['required'])
            self.assertNotEqual(definition['initial']['facts'], definition['example']['facts'])
            for mode in ['assignment', 'practice']:
                draft = lab['initial']['simulation'][mode]
                self.assertFalse(draft['checked'])
                self.assertFalse(draft['attempted'])
                self.assertEqual(draft['input'], '')
                self.assertIsNone(draft['last_run'])
            self.assertEqual(lab['solution'][-1], 'source-action {"kind":"check"}')
            self.assertEqual(len(definition['initial']['cases']), 4)

    def test_teaches_commands_beside_actions_without_showing_assignment_method_by_default(self):
        rows = read('scenarios/presentation/lpic2.json')[100:103]
        labs = read('scenarios/boot-entry-lessons-v19.json')['scenarios'][-3:]
        for lab, row in zip(labs, rows):
            self.assertEqual(row['id'], lab['id'])
            self.assertEqual(' '.join(row['requirements']), lab['objective'])
            self.assertEqual(len(row['practice']), 3)
            actions = ' '.join(p['action'] for p in row['practice'])
            self.assertIn('Run command', actions)
            self.assertIn('Check example', actions)
            self.assertNotIn('source-action', str(row))
            self.assertNotIn('make ', ' '.join(row['requirements']), 'Requirements state outcomes rather than an exact solution sequence')
            self.assertIn('not a general C compiler', lab['simplifications'])
            self.assertIn('remains incomplete', lab['simplifications'])
        self.assertIn('prefix from a staging root', ' '.join(c['title'] for c in rows[1]['concepts']))
        self.assertIn('staged copy', str(rows[2]))

    def test_runtime_model_has_no_host_io_and_development_reference_is_explicitly_separate(self):
        for name in ['src/source_install.rs', 'src/source_install/command.rs', 'src/source_install_workspace.rs']:
            text = (ROOT / name).read_text()
            for forbidden in ['std::process', 'std::fs', 'Command::new', 'TcpStream', 'UdpSocket']:
                self.assertNotIn(forbidden, text)
        reference = (ROOT / 'tests/source_install_reference.rs').read_text()
        self.assertIn('#[ignore =', reference)
        self.assertIn('cfg(target_os = "linux")', reference)
        self.assertIn('env_clear()', reference)
        self.assertIn('"--jobs=1"', reference)
        self.assertIn('No learner data or personal files', reference)
        self.assertIn('.build-artifacts/tmp', reference)
        page = (ROOT / 'src/ui/source_install_page.rs').read_text()
        self.assertIn('worked_command_rows', page)
        self.assertIn('singleline_fitting', page)
        self.assertIn('fit_popup_contents', page)
        self.assertIn('task.check_label(practice)', page)
