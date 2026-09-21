"""Import reviewed policy copy and source bindings; no simulation-data edits."""
import json
from pathlib import Path

root = Path(__file__).resolve().parents[2]
review = root / 'review/art-satire'
web = root / 'dystrail-web'
pack = json.loads((root / 'review/satire-fresh-2026-09-13/complete-pack.json').read_text())
units = [u for u in pack['units'] if u['id'].startswith('ORDER-') and u['variant'] in ('B', 'C')]
assert len(units) == 12
translations = json.loads((review / 'policy-bc-translations.json').read_text())
english = {}
for unit in units:
    expiry = unit['expiration'].replace('his test coin', 'their test coin')
    english[unit['id']] = [unit['title'], unit['activation'], expiry] + [o['text'] for o in unit['scene']['overlays']]
translations['en'] = english
for path in (web / 'i18n').glob('*.json'):
    lang = path.stem
    rows = translations.get(lang, english)
    assert set(rows) == set(english), lang
    data = json.loads(path.read_text())
    for unit, values in rows.items():
        assert len(values) == len(english[unit]), (lang, unit)
        data.setdefault('visual_copy', {})[unit] = dict(zip(['title', 'activation', 'expiration'], values[:3]))
        data['visual_copy'][unit]['overlays'] = {str(i): value for i, value in enumerate(values[3:])}
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n')

sources = json.loads((web / 'static/assets/data/satire-sources.json').read_text())
path = web / 'static/assets/data/visual-sources.json'
visual = json.loads(path.read_text())
overrides = json.loads((review / 'policy-source-overrides.json').read_text())
for unit in units:
    if unit['id'] in overrides:
        visual[unit['id']] = overrides[unit['id']]
    else:
        original = sources[unit['family_id']]
        assert original['source'] == unit['political_basis']['source_url'], unit['id']
        visual[unit['id']] = original
path.write_text(json.dumps(visual, ensure_ascii=False, indent=2) + '\n')
print('Integrated 12 policy variants in EN/ES/IT/AR, with 12 selected-unit source bindings.')
