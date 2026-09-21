"""Record byte identity of the local candidate and preserved hearing work."""
import base64
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parents[2]
review = root/'review/art-satire'
build = Path('/private/tmp/dystrail-art-satire-build')
manifest = json.loads((build/'offline-manifest.json').read_text())
for asset in manifest['assets']:
    raw = (build/asset['path']).read_bytes()
    assert len(raw) == asset['bytes'], asset['path']
    assert 'sha256-'+base64.b64encode(hashlib.sha256(raw).digest()).decode() == asset['integrity'], asset['path']
for variant in 'abc':
    path = Path(f'static/img/scenes-v2/activities-{variant}.png')
    assert (root/'dystrail-web'/path).read_bytes() == (build/path).read_bytes()
record = dict(revision=manifest['revision'],assets_checked=len(manifest['assets']),bytes=manifest['bytes'],
              result='Every packaged asset length and SHA-256 matches the offline manifest; all activity atlas bytes match their source files.')
(review/'activity-offline-integrity.json').write_text(json.dumps(record,indent=2)+'\n')

snapshot = json.loads((review/'hearing-snapshot/manifest.json').read_text())
main = Path('/Users/vanna/Source/Dystrail')
for entry in snapshot['files']:
    assert hashlib.sha256((main/entry['path']).read_bytes()).hexdigest() == entry['sha256'], entry['path']
engine = json.loads((review/'hearing-engine-preservation.json').read_text())['checked_files']
for entry in engine:
    assert hashlib.sha256((root/entry['path']).read_bytes()).hexdigest() == entry['sha256'], entry['path']
locales = []
for path in (root/'dystrail-web/i18n').glob('*.json'):
    current = json.loads(path.read_text())
    original = json.loads((review/'hearing-snapshot/dystrail-web/i18n'/path.name).read_text())
    assert current['hearing'] == original['hearing'], path.name
    for family in ['ACT-FORAGE','ACT-GLEAN','ACT-FOODWORK','ACT-CASHWORK']:
        for variant in 'ABC':
            unit = current['visual_copy'][family+'-'+variant]
            assert set(unit) == {'title','setup','action','outcome'}
            assert all(unit.values())
    locales.append(path.stem)
record = dict(other_thread='01a09da1-4acb-7731-8029-d7885aa05c10',last_completed_turn='01a0a129-1151-7fe2-b388-5d12eb524449',
              snapshot_files_unchanged=len(snapshot['files']),engine_files_unchanged=len(engine),
              hearing_locales_unchanged=sorted(locales),main_checkout_changed_files=[])
(review/'activity-hearing-preservation.json').write_text(json.dumps(record,indent=2)+'\n')
print(f"Verified {manifest['revision']}: {len(manifest['assets'])} offline assets; preserved {len(engine)} hearing engine/tester files and {len(locales)} hearing locale subtrees.")
