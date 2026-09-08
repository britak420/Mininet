#!/usr/bin/env python3
"""Verify publication bytes and coverage, not protocol security or audit quality."""
from __future__ import annotations
import hashlib
from html.parser import HTMLParser
import json
from pathlib import Path
import re
import shutil
import sys
import tempfile

ROOT = Path(__file__).resolve().parent
LABELS = (
    "Contribution to the founder's idea.", "Mechanism and evidence.",
    "What remains weaker than the intended claim.",
    "Recommended improvement and rationale.", "Concrete example.",
    "Acceptance tests to implement.", "History, supersession and integration.",
    "Source entry points.", "Review boundary.",
)

class Anchors(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.ids: list[str] = []
        self.links: list[str] = []
        self.remote_scripts: list[str] = []
    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        values = dict(attrs)
        if values.get('id'):
            self.ids.append(str(values['id']))
        if tag == 'a' and str(values.get('href', '')).startswith('#'):
            self.links.append(str(values['href'])[1:])
        if tag == 'script' and values.get('src'):
            self.remote_scripts.append(str(values['src']))

def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)

def verify(root: Path) -> dict:
    manifest = json.loads((root / 'PUBLICATION_MANIFEST.json').read_text())
    for entry in manifest['files']:
        path = root / entry['path']
        require(path.parent == root, 'manifest path escaped report directory')
        data = path.read_bytes()
        require(len(data) == entry['bytes'], entry['path'] + ': length mismatch')
        require(hashlib.sha256(data).hexdigest() == entry['sha256'], entry['path'] + ': SHA-256 mismatch')
        sha = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
        require(sha == entry['git_blob_sha'], entry['path'] + ': Git blob mismatch')
    evidence = json.loads((root / 'EVIDENCE_MAP.json').read_text())
    text = (root / 'ALL_174_PR_REVIEWS.md').read_text()
    pieces = re.split(r'^## PR #(\d+):', text, flags=re.M)
    numbers = [int(pieces[i]) for i in range(1, len(pieces), 2)]
    require(len(numbers) == len(set(numbers)) == 174, 'duplicate or missing dossiers')
    rows = evidence['pull_requests']
    require(len(rows) == 174 and set(numbers) == {r['number'] for r in rows}, 'inventory disagreement')
    blocks = {int(pieces[i]): pieces[i+1] for i in range(1, len(pieces), 2)}
    for row in rows:
        block = blocks[row['number']]
        require(row['head_sha'] in block and row['base_sha'] in block, 'PR head/base mismatch')
        require(all('**' + label + '**' in block for label in LABELS), 'missing PR section')
        require(len(row['files']) == row['changed_files'], 'incomplete changed-file inventory')
        require(('**' + row['outcome'] + '**') in block, 'PR outcome mismatch')
    require(sum(r['outcome'] == 'merged' for r in rows) == 166, 'merged-count mismatch')
    require(sum(r['outcome'] == 'closed' for r in rows) == 8, 'closed-unmerged count mismatch')
    html = Anchors(); html.feed((root / 'EXHAUSTIVE_REVIEW.html').read_text()); html.close()
    require(len(html.ids) == len(set(html.ids)), 'duplicate HTML anchors')
    require(set(html.links).issubset(html.ids), 'broken internal HTML navigation')
    require({f'pr-{n:04d}' for n in numbers}.issubset(html.ids), 'missing HTML dossier')
    require(not html.remote_scripts, 'external script dependency added')
    return {'files_verified': len(manifest['files']), 'dossiers': len(numbers),
            'changed_file_records': sum(r['changed_files'] for r in rows),
            'integrity_and_coverage': 'PASS', 'security_or_external_audit': 'NOT_CLAIMED'}

def self_test(root: Path) -> None:
    # Each check must reject corruption independently of the normal success path.
    with tempfile.TemporaryDirectory() as directory:
        copy = Path(directory) / 'report'; shutil.copytree(root, copy)
        verify(copy)
        target = copy / 'ALL_174_PR_REVIEWS.md'
        original = target.read_bytes()
        target.write_bytes(original[:-1])
        try:
            verify(copy)
        except ValueError:
            pass
        else:
            raise ValueError('truncation was not rejected')
        target.write_bytes(original)
        target = copy / 'EXHAUSTIVE_REVIEW.html'; target.unlink()
        try:
            verify(copy)
        except FileNotFoundError:
            pass
        else:
            raise ValueError('missing companion was not rejected')

if __name__ == '__main__':
    try:
        result = verify(ROOT)
        if sys.argv[1:] == ['--self-test']:
            self_test(ROOT); result['negative_integrity_tests'] = 'PASS (2)'
        elif sys.argv[1:]:
            raise ValueError('usage: verify_publication.py [--self-test]')
        print(json.dumps(result, sort_keys=True))
    except (ValueError, OSError, KeyError, TypeError) as error:
        raise SystemExit('FAIL: ' + str(error)) from None
