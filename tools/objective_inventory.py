#!/usr/bin/env python3
"""Index local copies of official objectives, without copying their prose into the repo.
No network access or subprocesses. PDF text must be extracted separately in development.
"""
from datetime import date
import argparse
import hashlib
import html
from html.parser import HTMLParser
import json
from pathlib import Path
import re

SOURCES = {
    'CCNA v1.1': ('https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf', '1.1'),
    'LPIC-1 v5.0': ('https://www.lpi.org/our-certifications/exam-101-102-objectives/', '5.0'),
    'LPIC-2 v4.5': ('https://www.lpi.org/our-certifications/exam-201-202-objectives/', '4.5'),
}


def normalized(text):
    return ' '.join(html.unescape(text).split())


def digest(text):
    return hashlib.sha256(normalized(text).encode()).hexdigest()


def plain(text):
    return normalized(re.sub(r'<[^>]+>', ' ', text))


class Lists(HTMLParser):
    def __init__(self):
        super().__init__()
        self.depth = 0
        self.items = []
        self.current = None

    def handle_starttag(self, tag, attrs):
        if tag == 'ul':
            self.depth += 1
        elif tag == 'li' and self.depth == 1:
            if self.current is not None:
                raise ValueError('Unclosed knowledge/term item')
            self.current = []

    def handle_data(self, text):
        if self.current is not None:
            self.current.append(text)

    def handle_endtag(self, tag):
        if tag == 'li' and self.depth == 1:
            if self.current is None:
                raise ValueError('Unmatched knowledge/term item')
            item = normalized(''.join(self.current))
            if not item:
                raise ValueError('Empty knowledge/term item')
            self.items.append(item)
            self.current = None
        elif tag == 'ul':
            self.depth -= 1
            if self.depth < 0:
                raise ValueError('Unmatched objective list')


def list_items(fragment):
    parser = Lists()
    parser.feed(fragment)
    parser.close()
    if parser.depth or parser.current is not None or not parser.items:
        raise ValueError('Incomplete or absent objective list')
    return parser.items


def item(reference, parent, kind, position, wording, anchor=''):
    return dict(reference=reference, parent=parent, kind=kind,
                position=position, normalized_sha256=digest(wording), anchor=anchor)


def lpi_items(source, version):
    if not re.search(r'Exam Objectives Version:\s*' + re.escape(version) + r'\b', plain(source)):
        raise ValueError('Unexpected LPI version')
    headers = list(re.finditer(r'<h([234])\b[^>]*>(.*?)</h\1>', source, re.S))
    result = []
    for n, match in enumerate(headers):
        title = plain(match[2])
        number = re.match(r'(\d{3}\.\d+)\b', title)
        if match[1] != '4' or not number:
            continue
        if title == '104.4 Removed':
            continue
        reference = number[1]
        anchor = re.search(r'\bid="([^"]+)"', match[0])
        if not anchor:
            raise ValueError('Missing objective anchor')
        anchor = anchor[1]
        end = headers[n + 1].start() if n + 1 < len(headers) else len(source)
        body = source[match.end():end]
        knowledge = re.search(r'Key Knowledge Areas:', body, re.I)
        terms = re.search(r'The following is a partial list', body, re.I)
        if not knowledge or not terms or knowledge.end() >= terms.start():
            raise ValueError('Missing or reordered objective sections: ' + reference)
        cells = [plain(cell) for cell in re.findall(r'<td\b[^>]*>(.*?)</td>', body, re.S)]
        if len(cells) != 4 or cells[0] != 'Weight' or cells[2] != 'Description' or not cells[1].isdigit() or not cells[3]:
            raise ValueError('Missing or unsupported objective weight/description: ' + reference)
        weight, description = int(cells[1]), cells[3]
        parent = item(reference, reference, 'objective', 0, title + '\n' + description + f'\nWeight:{weight}', anchor)
        parent.update(description_sha256=digest(description), weight=weight)
        result.append(parent)
        for kind, values in [('K', list_items(body[knowledge.end():terms.start()])),
                             ('T', list_items(body[terms.end():]))]:
            for position, wording in enumerate(values, 1):
                result.append(item(f'{reference}.{kind}{position}', reference,
                                   'knowledge' if kind == 'K' else 'term', position, wording, anchor))
    return checked(result)


def cisco_items(source):
    if 'CCNA Exam v1.1 (200-301)' not in source:
        raise ValueError('Unexpected Cisco version')
    result = []
    active = None
    for line in source.splitlines():
        line = line.replace('\f', '')
        found = re.match(r'\s*(\d\.\d+(?:\.[a-z])?)\s+(.+)', line)
        if found:
            if active:
                result.append(active)
            reference, wording = found.groups()
            active = [reference, wording]
        elif not line.strip() or re.match(r'\s*(?:\d+%|2024 Cisco|CCNA Exam)', line):
            if active:
                result.append(active)
                active = None
        elif active:
            active[1] += ' ' + line.strip()
    if active:
        result.append(active)
    items = []
    for reference, wording in result:
        parent = '.'.join(reference.split('.')[:2])
        items.append(item(reference, parent, 'objective' if reference == parent else 'subobjective',
                          0, wording))
    return checked(items)


def checked(items):
    keys = [i['reference'] for i in items]
    if not keys or len(keys) != len(set(keys)):
        raise ValueError('Missing or duplicate objective references')
    parents = {i['reference'] for i in items if i['kind'] == 'objective'}
    if any(i['parent'] not in parents for i in items):
        raise ValueError('Subobjective without a parent')
    return items


def inventory(ccna_text, lpic1_html, lpic2_html, checked_date):
    sources = []
    for blueprint, raw, items in [
        ('CCNA v1.1', ccna_text, cisco_items(ccna_text)),
        ('LPIC-1 v5.0', lpic1_html, lpi_items(lpic1_html, '5.0')),
        ('LPIC-2 v4.5', lpic2_html, lpi_items(lpic2_html, '4.5')),
    ]:
        url, version = SOURCES[blueprint]
        sources.append(dict(blueprint=blueprint, version=version, checked=checked_date,
                            url=url, input_sha256=hashlib.sha256(raw.encode()).hexdigest(), items=items))
    return dict(schema_version=1, sources=sources)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--ccna-text', type=Path, required=True)
    parser.add_argument('--lpic1-html', type=Path, required=True)
    parser.add_argument('--lpic2-html', type=Path, required=True)
    parser.add_argument('--checked', required=True)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument('--output', type=Path)
    group.add_argument('--check', type=Path)
    args = parser.parse_args()
    if not re.fullmatch(r"\d{4}-\d{2}-\d{2}", args.checked):
        raise ValueError("Use an explicit YYYY-MM-DD source check date")
    date.fromisoformat(args.checked)
    result = inventory(args.ccna_text.read_text(encoding="utf-8"), args.lpic1_html.read_text(encoding="utf-8"),
                       args.lpic2_html.read_text(encoding="utf-8"), args.checked)
    if args.output:
        args.output.write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
    else:
        original = json.loads(args.check.read_text(encoding="utf-8"))
        # Changes in surrounding HTML or PDF text extraction do not change semantic entries.
        old = {s['blueprint']: s['items'] for s in original['sources']}
        new = {s['blueprint']: s['items'] for s in result['sources']}
        if old != new:
            raise SystemExit('Objective inventory changed; review source items before updating coverage')
    print('Indexed', sum(len(s['items']) for s in result['sources']), 'current objective references')


if __name__ == '__main__':
    main()
