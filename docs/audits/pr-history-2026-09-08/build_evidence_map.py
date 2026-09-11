#!/usr/bin/env python3
"""Build the publication evidence index from the exact captured GitHub ZIP.

Read-only input; never extracts or executes archive content. Does not turn
an evidence export into security review, external audit, or approval.
"""
import hashlib
import json
import sys
import zipfile
from pathlib import Path

EXPECTED = '4dbb5b4f8f6e7911e5f048be3bbb34b8771503d6702a288854e534e6343774cd'

def generate(archive: Path) -> bytes:
    if archive.stat().st_size != 25100909 or hashlib.sha256(archive.read_bytes()).hexdigest() != EXPECTED:
        raise ValueError('not the pinned 2026-09-08 evidence capture')
    with zipfile.ZipFile(archive) as z:
        inv = json.loads(z.read('inventory.json'))
        rows = []
        for source in inv['pull_requests']:
            raw = z.read(source['evidence_file'])
            record = json.loads(raw)
            row = dict(source)
            row['capture_record_sha256'] = hashlib.sha256(raw).hexdigest()
            row['dossier'] = 'ALL_174_PR_REVIEWS.md#pr-%04d' % row['number']
            row['files'] = [{key: f[key] for key in ['filename', 'previous_filename', 'sha', 'status', 'additions', 'deletions', 'changes', 'blob_url', 'raw_url'] if key in f} | {'api_patch_present': 'patch' in f} for f in record['files']]
            if len(row['files']) != row['changed_files']:
                raise ValueError('changed-file count mismatch')
            rows.append(row)
        result = {key: value for key, value in inv.items() if key != 'pull_requests'}
        result['publication_note'] = 'Generated from the pinned capture. review_completed remains the exporter\'s false collection marker; per-PR research is in the dossier. This map proves inventory traceability, not audit completion.'
        result['capture_artifact'] = {'run_id': 34200283626, 'artifact_id': 10045573430, 'archive_sha256': EXPECTED, 'url': 'https://github.com/mininet-labs/Mininet/actions/runs/34200283626/artifacts/10045573430', 'expires_at': '2026-09-22T07:39:32Z'}
        result['pull_requests'] = rows
        if len(rows) != 174 or len({r['number'] for r in rows}) != 174:
            raise ValueError('inventory mismatch')
        return (json.dumps(result, ensure_ascii=False, indent=2) + '\n').encode()

if __name__ == '__main__':
    if len(sys.argv) != 3:
        raise SystemExit('usage: build_evidence_map.py CAPTURE.zip OUTPUT.json')
    output = Path(sys.argv[2])
    if output.exists():
        raise SystemExit('output already exists; preserve prior evidence')
    output.write_bytes(generate(Path(sys.argv[1])))
