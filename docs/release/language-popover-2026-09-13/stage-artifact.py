"""Stage a verified frozen game build over the preserved production Pages payload."""
import argparse
import base64
import hashlib
import json
import re
import shutil
import subprocess
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('--artifact', required=True, type=Path)
parser.add_argument('--revision', required=True)
parser.add_argument('--manifest-sha256', required=True)
parser.add_argument('--source-commit', required=True)
parser.add_argument('--source-ci-run', required=True, type=int)
args = parser.parse_args()
artifact = args.artifact.resolve(strict=True)
work = Path('/tmp/dystrail-language-popover-20260913/release-prep')
root = Path('/tmp/dystrail-source-handoff-publication')
payload = root/'release-site'
baseline = json.loads(Path('/tmp/dystrail-language-popover-20260913/release-prep/baseline-release-manifest.json').read_text())
verified = json.loads((work/'production-baseline-verification.json').read_text())
assert verified['passed'] and verified['revision'] == baseline['revision']
assert re.fullmatch(r'[0-9a-f]{20}', args.revision)
assert re.fullmatch(r'[0-9a-f]{40}', args.source_commit)
manifest_bytes = (artifact/'offline-manifest.json').read_bytes()
assert hashlib.sha256(manifest_bytes).hexdigest() == args.manifest_sha256, 'Frozen manifest hash changed'
manifest = json.loads(manifest_bytes)
assert manifest['revision'] == args.revision, 'Frozen revision changed'
worker = (artifact/'sw.js').read_text()
worker_manifest = json.loads(re.search(r'^const BUILD = (.*);$',worker,re.M).group(1))
assert worker_manifest == manifest, 'Worker and offline manifest disagree'

artifact_files = {p.relative_to(artifact).as_posix():p for p in artifact.rglob('*') if p.is_file()}
assert all(not p.is_symlink() for p in artifact.rglob('*')), 'Artifact contains symlinks'
expected_paths = {entry['path'] for entry in manifest['assets']} | {'sw.js','offline-manifest.json'}
assert set(artifact_files) == expected_paths, 'Artifact has unexpected or missing payload files'
assert sum(entry['bytes'] for entry in manifest['assets']) == manifest['bytes']
for entry in manifest['assets']:
    relative = Path(entry['path'])
    assert not relative.is_absolute() and '..' not in relative.parts
    data = artifact_files[entry['path']].read_bytes()
    assert len(data) == entry['bytes'], entry['path']
    assert 'sha256-'+base64.b64encode(hashlib.sha256(data).digest()).decode() == entry['integrity'], entry['path']

before = {p.relative_to(payload).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in payload.rglob('*') if p.is_file()}
assert before == baseline['files'], 'Release workspace no longer equals verified production baseline'
# Retain old hashed game assets for visitors completing an existing installation.
shutil.copytree(artifact, payload/'play', dirs_exist_ok=True)
shutil.copyfile(artifact/'index.html', payload/'404.html')
source_root = Path('/Users/vanna/Source/Dystrail')
for source_path, destination in [('site/index.html','index.html'),('site/assets/gameplay-current.jpg','assets/gameplay-current.jpg')]:
    data = subprocess.check_output(['git','show',f'{args.source_commit}:{source_path}'],cwd=source_root)
    (payload/destination).write_bytes(data)
after = {p.relative_to(payload).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(payload.rglob('*')) if p.is_file()}
protected = {name:sha for name,sha in before.items() if not name.startswith('play/') and name not in {'404.html','index.html'}}
assert all(after.get(name) == sha for name,sha in protected.items()), 'A homepage or documentation file changed'
old_hashed = {name:sha for name,sha in before.items() if name.startswith('play/') and re.search(r'-[0-9a-f]{8,}(?:_bg)?\.(?:js|css|wasm)$', name)}
assert all(after.get(name) == sha for name,sha in old_hashed.items()), 'An existing hashed game asset changed'
assert (payload/'CNAME').read_text().strip() == 'dystrail.com'
changed = sorted(name for name in set(before)&set(after) if before[name] != after[name])
added = sorted(set(after)-set(before))
removed = sorted(set(before)-set(after))
assert not removed
assert all(name.startswith('play/') or name in {'404.html','index.html','assets/gameplay-current.jpg'} for name in changed + added)
release = {'revision':args.revision,'previous_revision':baseline['revision'],'files':after,'game_assets':len(manifest['assets']),'game_bytes':manifest['bytes'],'preserved_files':protected,'preserved_hashed_assets':old_hashed,'artifact_manifest_sha256':args.manifest_sha256}
release['source_commit'] = args.source_commit
release['source_branch'] = 'release/complete-game-source-2026-09-13'
release['source_ci_run'] = args.source_ci_run
release['build_origin'] = 'Exact release artifact downloaded from source CI'
(root/'release-manifest.json').write_text(json.dumps(release,indent=2)+'\n')
report = {'artifact':str(artifact),'revision':args.revision,'artifact_manifest_sha256':args.manifest_sha256,'game_assets':len(manifest['assets']),'total_files':len(after),'protected_files':len(protected),'preserved_hashed_assets':len(old_hashed),'changed':changed,'added':added,'removed':removed,'passed':True}
(work/'staging-verification.json').write_text(json.dumps(report,indent=2)+'\n')
verifier = root/'scripts/verify-release-payload.py'
verifier.write_text('''"""Verify exact published bytes, offline integrity, and preserved website files."""
import base64
import hashlib
import json
import re
from pathlib import Path

root = Path(__file__).resolve().parents[1]
payload = root / "release-site"
manifest = json.loads((root / "release-manifest.json").read_text())
assert manifest["revision"] == "'''+args.revision+'''", "Unexpected preview revision"
files = {str(path.relative_to(payload)): path for path in payload.rglob("*") if path.is_file()}
assert set(files) == set(manifest["files"]), "Payload inventory differs from reviewed artifact"
assert all(not path.is_symlink() for path in payload.rglob("*")), "Unexpected symlink"
for name, path in files.items():
    assert hashlib.sha256(path.read_bytes()).hexdigest() == manifest["files"][name], name
for name, expected in manifest["preserved_files"].items():
    assert manifest["files"][name] == expected, name
for name, expected in manifest["preserved_hashed_assets"].items():
    assert manifest["files"][name] == expected, name
assert (payload / "CNAME").read_text().strip() == "dystrail.com"
assert (payload / "404.html").read_bytes() == (payload / "play/index.html").read_bytes()
offline_bytes = (payload / "play/offline-manifest.json").read_bytes()
assert hashlib.sha256(offline_bytes).hexdigest() == manifest["artifact_manifest_sha256"]
offline = json.loads(offline_bytes)
assert offline["revision"] == manifest["revision"]
worker = (payload / "play/sw.js").read_text()
assert json.loads(re.search(r"^const BUILD = (.*);$", worker, re.M).group(1)) == offline
for asset in offline["assets"]:
    data = (payload / "play" / asset["path"]).read_bytes()
    assert len(data) == asset["bytes"], asset["path"]
    assert "sha256-" + base64.b64encode(hashlib.sha256(data).digest()).decode() == asset["integrity"], asset["path"]
print(f"Verified {len(files)} files and {len(offline['assets'])} offline assets for preview {manifest['revision']}")
''')
print(json.dumps(report,indent=2))
