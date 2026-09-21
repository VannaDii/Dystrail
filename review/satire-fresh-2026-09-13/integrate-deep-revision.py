"""Integrate the bounded rights/health and news-diversity editorial pass."""
import collections
import hashlib
import json
import re
from pathlib import Path
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parent
BEFORE = Path('/private/tmp/dystrail-before-deep-20260914')
NAMES = ['roads-a', 'roads-b', 'towns', 'support', 'functional']
INPUTS = {'deep-rights-surveillance': 30, 'deep-rights-reproductive': 27,
          'news-diversity-light': 12, 'shared-boundary-fixes': 8,
          'shared-town-boundary-fixes': 14}
parts = {n: json.loads((ROOT / (n + '.json')).read_text()) for n in NAMES}
old = {u['id']: u for n in NAMES for u in json.loads((BEFORE / (n + '.json')).read_text())}
replacements = {}
for name, count in INPUTS.items():
    rows = json.loads((ROOT / (name + '.json')).read_text())
    assert len(rows) == count, (name, len(rows), count)
    for u in rows:
        assert u['id'] in old and u['id'] not in replacements, u['id']
        for source in u.get('sources', []):
            if not source.get('title'):
                source['title'] = u['political_basis']['subject']
                if len(u['sources']) > 1:
                    source['title'] += ' — ' + urlsplit(source['url']).netloc.removeprefix('www.')
        replacements[u['id']] = u
for name, rows in parts.items():
    parts[name] = [replacements.get(u['id'], u) for u in rows]

AUTHOR_FIELDS = {'title', 'setup', 'fact', 'commentary', 'speaker', 'sources',
    'source_hook', 'evidence_status', 'illustration', 'status', 'scene', 'scene_id',
    'political_basis', 'continuing', 'onset', 'activation', 'headline', 'epilogue',
    'outcome', 'expiration', 'recovery', 'final_ally', 'text', 'transitions',
    'before_vote', 'exhausted', 'action', 'cast_requirements', 'content_gate', 'content_topics'}

def no_logs(v):
    if isinstance(v, dict): return {k: no_logs(x) for k, x in v.items() if k != 'log'}
    if isinstance(v, list): return [no_logs(x) for x in v]
    return v

def mechanics(u):
    d = {k: no_logs(v) for k, v in u.items() if k not in AUTHOR_FIELDS | {'choices', 'outcomes'}}
    if 'choices' in u:
        d['choices'] = [no_logs({k: v for k, v in c.items() if k not in {'label', 'outcome', 'roadside_outcome'}}) for c in u['choices']]
        d['roadside_outcome_slots'] = [i for i, c in enumerate(u['choices']) if 'roadside_outcome' in c]
    if 'outcomes' in u: d['outcome_slots'] = sorted(u['outcomes'])
    return d

def playable(u):
    fields = AUTHOR_FIELDS - {'sources', 'source_hook', 'evidence_status', 'status',
        'scene', 'scene_id', 'illustration', 'political_basis', 'cast_requirements',
        'content_gate', 'content_topics'}
    values = [u[k] for k in fields if k in u]
    values += [{k: c[k] for k in ('label', 'outcome', 'roadside_outcome') if k in c} for c in u.get('choices', [])]
    values += [u.get('outcomes', {})]
    return json.dumps(values, ensure_ascii=False)

units = [u for rows in parts.values() for u in rows]
narrative = [u for n in NAMES[:-1] for u in parts[n]]
assert len(units) == 644 and len({u['id'] for u in units}) == 644
assert {u['id'] for u in units} == set(old)
assert len(narrative) == 567 and len(parts['functional']) == 77
by = {u['id']: u for u in units}
changed = {u['id'] for u in units if u != old[u['id']]}
expected_changes = set(replacements) | {'CROSS-02D-' + v for v in 'ABC'} | {'HEARING-D-' + v for v in 'ABC'} | {'MODE-D'}
assert changed == expected_changes, {'unexpected': sorted(changed - expected_changes), 'missing': sorted(expected_changes - changed)}
for u in units:
    assert mechanics(u) == mechanics(old[u['id']]), ('mechanics', u['id'])
    if u['id'].startswith('CARE-') or u['id'] in {'ENC-C01-A', 'ENC-S13-A'}:
        assert u == old[u['id']], ('locked copy', u['id'])
    for c in u.get('choices', []):
        if 'log' in c.get('effects', {}): assert c['effects']['log'] == c['outcome'], ('stale log', u['id'])
    assert u.get('scene', {}).get('asset_notes'), ('scene guard', u['id'])
    assert not re.search(r'\b(?:water|food|supply|supplies) packs?\b|\b(?:TODO|TBD|insert joke|lorem ipsum)\b', playable(u), re.I), ('unfinished prose', u['id'])
    if u['id'] in changed and u in narrative:
        assert u.get('sources') and u.get('political_basis', {}).get('real_hook'), ('source', u['id'])
deep = [u for u in narrative if u['mode'] == 'Deep End']
assert len(deep) == 63
assert all(u.get('content_gate') == 'deep_only' and u.get('content_topics') for u in deep)
assert not any(u.get('content_gate') == 'deep_only' and u['mode'] != 'Deep End' for u in narrative)
assert collections.Counter(u['mode'] for u in narrative if u['category'] == 'Road encounters') == {'Classic': 72, 'Shared': 102, 'Deep End': 57}
assert 'opt in' in by['MODE-D']['text'] and 'abortion restrictions' in by['MODE-D']['text']
assert all(u == old[u['id']] for u in parts['functional'] if u['id'] != 'MODE-D')

# This is a bounded manual-review flag list, not an automated rating of humor.
heavy = re.compile(r'abortion|miscarriage|mifepristone|misoprostol|pregnan|\bIVF\b|embryo|\bFlock\b|stalk|fertility data|clinic.location|gender.affirm|massacre|internment|slavery', re.I)
boundary_flags = [{'id': u['id'], 'matches': sorted(set(m.group(0) for m in heavy.finditer(playable(u))))} for u in narrative if u['mode'] != 'Deep End' and heavy.search(playable(u))]
assert not boundary_flags, ('mode boundary requires review', boundary_flags)

for name, rows in parts.items():
    (ROOT / (name + '.json')).write_text(json.dumps(rows, ensure_ascii=False, indent=2) + '\n')
sourcebook_path = ROOT / 'political-sourcebook.json'
sourcebook = json.loads(sourcebook_path.read_text())
known_urls = {s['url'] for s in sourcebook['sources']}
for u in replacements.values():
    basis = u['political_basis']
    url = basis['source_url']
    if url in known_urls: continue
    sourcebook['sources'].append({'id': 'news-' + u['id'].lower(), 'subject': basis['subject'],
        'fact_and_limits': basis['real_hook'], 'url': url, 'checked': '2026-09-14',
        'editorial_mode': 'Deep End only' if u.get('content_gate') == 'deep_only' else 'Classic or Shared'})
    known_urls.add(url)
sourcebook_path.write_text(json.dumps(sourcebook, ensure_ascii=False, indent=2) + '\n')
topics_before = collections.Counter(u.get('political_basis', {}).get('subject', 'unspecified') for u in old.values() if u.get('political_basis'))
topics_after = collections.Counter(u.get('political_basis', {}).get('subject', 'unspecified') for u in narrative)
report = {'status': 'verified editorial pack; not a production deployment',
    'changed_narrative_packages': len(changed - {'MODE-D'}), 'changed_functional_records': ['MODE-D'],
    'counts': {'narrative': 567, 'functional': 77, 'deep_only_narrative': 63},
    'changed_ids': sorted(changed), 'ordered_mechanics_preserved': True,
    'locked_approved_samples_preserved': True, 'shared_heavy_theme_flags': boundary_flags,
    'leading_topics_before': topics_before.most_common(10), 'leading_topics_after': topics_after.most_common(10),
    'runtime_note': 'The existing road mode selector treats an empty mode list as shared. Integration must retain Deep End eligibility, must not use heavy material as a Classic/shared fallback, and must apply the six support gates. This editorial pass does not deploy those changes.',
    'hashes': {n: hashlib.sha256((ROOT / (n + '.json')).read_bytes()).hexdigest() for n in NAMES}}
(ROOT / 'validation-deep-revision.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({k: report[k] for k in ['status', 'counts', 'changed_narrative_packages', 'shared_heavy_theme_flags']}))
