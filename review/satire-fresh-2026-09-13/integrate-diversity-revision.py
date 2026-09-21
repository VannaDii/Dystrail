"""Integrate reviewed replacements and validate against this revision's baseline."""
import collections, copy, hashlib, json, re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
BASE = Path('/private/tmp/dystrail-before-diversity-20260914')
NAMES = ['roads-a', 'roads-b', 'towns', 'support', 'functional']
old_parts = {n: json.loads((BASE / (n + '.json')).read_text()) for n in NAMES}
old = {u['id']: u for rows in old_parts.values() for u in rows}
parts = {n: json.loads((ROOT / (n + '.json')).read_text()) for n in NAMES}
targets = sum([json.loads((ROOT / f'diversity-{n}-targets.json').read_text()) for n in ['roads', 'support', 'deep']], [])
target_ids = {u['id'] for u in targets}
assert len(targets) == len(target_ids)
replacements = {}
for name in ['roads', 'support', 'deep']:
    rows = json.loads((ROOT / f'diversity-{name}-replacements.json').read_text())
    wanted = {u['id'] for u in json.loads((ROOT / f'diversity-{name}-targets.json').read_text())}
    assert len(rows) == len({u['id'] for u in rows})
    assert {u['id'] for u in rows} == wanted, (name, 'incomplete assignment')
    for u in rows:
        replacements[u['id']] = copy.deepcopy(u)

registry = json.loads((ROOT / 'diversity-hook-registry.json').read_text())
old_hooks = {ident: r['news_hook_id'] for r in registry for ident in r['member_ids']}
for name, rows in parts.items():
    parts[name] = [replacements.get(u['id'], u) for u in rows]
    if name != 'functional':
        for u in parts[name]:
            u.setdefault('news_hook_id', old_hooks[u['id']])
            # Only logs are narrative; all other effects remain immutable.
            for c in u.get('choices', []):
                if 'log' in c.get('effects', {}):
                    c['effects']['log'] = c['outcome']

AUTHOR_FIELDS = {'title', 'setup', 'fact', 'commentary', 'speaker', 'sources',
    'source_hook', 'evidence_status', 'illustration', 'status', 'scene', 'scene_id',
    'political_basis', 'continuing', 'onset', 'activation', 'headline', 'epilogue',
    'outcome', 'expiration', 'recovery', 'final_ally', 'text', 'transitions',
    'before_vote', 'exhausted', 'action', 'cast_requirements', 'content_gate',
    'content_topics', 'news_hook_id'}

def no_logs(v):
    if isinstance(v, dict):
        return {k: no_logs(x) for k, x in v.items() if k != 'log'}
    if isinstance(v, list):
        return [no_logs(x) for x in v]
    return v

def mechanics(u):
    result = {k: no_logs(v) for k, v in u.items() if k not in AUTHOR_FIELDS | {'choices', 'outcomes'}}
    if 'choices' in u:
        result['choices'] = [no_logs({k: v for k, v in c.items() if k not in {'label', 'outcome', 'roadside_outcome'}}) for c in u['choices']]
        result['roadside_outcome_slots'] = [i for i, c in enumerate(u['choices']) if 'roadside_outcome' in c]
    if 'outcomes' in u:
        result['outcome_slots'] = sorted(u['outcomes'])
    return result

def playable(u):
    fields = AUTHOR_FIELDS - {'sources', 'source_hook', 'evidence_status', 'status',
        'scene', 'scene_id', 'illustration', 'political_basis', 'cast_requirements',
        'content_gate', 'content_topics', 'news_hook_id'}
    values = [u[k] for k in fields if k in u]
    values += [{k: c[k] for k in ('label', 'outcome', 'roadside_outcome') if k in c} for c in u.get('choices', [])]
    values += [u.get('outcomes', {})]
    return json.dumps(values, ensure_ascii=False, sort_keys=True)

units = sum(parts.values(), [])
narrative = sum([parts[n] for n in NAMES[:-1]], [])
by = {u['id']: u for u in units}
assert len(units) == 644 and len(by) == 644 and set(by) == set(old)
assert len(narrative) == 567 and len(parts['functional']) == 77
changed = {u['id'] for u in units if playable(u) != playable(old[u['id']])}
assert changed == target_ids | {'MODE-D'}, {'unexpected': sorted(changed - target_ids - {'MODE-D'}), 'missing': sorted(target_ids - changed)}

for u in units:
    assert mechanics(u) == mechanics(old[u['id']]), ('mechanics', u['id'])
    assert u.get('scene', {}).get('asset_notes'), ('scene guard', u['id'])
    assert not re.search(r'\b(?:water|food|supply|supplies) packs?\b|\b(?:TODO|TBD|insert joke|lorem ipsum)\b', playable(u), re.I), ('unfinished prose', u['id'])
    if u['id'] in target_ids:
        assert u.get('sources') and u.get('political_basis', {}).get('real_hook') and u.get('news_hook_id'), ('missing basis', u['id'])
        assert set(re.findall(r'\{[A-Za-z_][A-Za-z_0-9]*\}', playable(u))) == set(re.findall(r'\{[A-Za-z_][A-Za-z_0-9]*\}', playable(old[u['id']]))), ('interpolation', u['id'])
    if u['id'].startswith('CARE-'):
        assert u['critical'] == old[u['id']]['critical'], ('critical warning', u['id'])
        for key in ['player_lost', 'companion_lost']:
            assert u['outcomes'][key] == old[u['id']]['outcomes'][key], ('death condition', u['id'], key)
    for c in u.get('choices', []):
        if 'log' in c.get('effects', {}):
            assert c['effects']['log'] == c['outcome'], ('stale log', u['id'])

for ident in ['ENC-C01-A', 'ENC-S13-A', 'CARE-01-A', 'CARE-02-A']:
    current = {k: v for k, v in by[ident].items() if k != 'news_hook_id'}
    assert current == old[ident], ('approved copy changed', ident)
assert all(u == old[u['id']] for u in parts['functional'] if u['id'] != 'MODE-D')

deep = [u for u in narrative if u['mode'] == 'Deep End']
deep_roads = [u for u in deep if u['category'] == 'Road encounters']
assert len(deep) == 63 and len(deep_roads) == 57
assert all(u.get('content_gate') == 'deep_only' and u.get('content_topics') for u in deep)
assert not any(u.get('content_gate') == 'deep_only' and u['mode'] != 'Deep End' for u in narrative)
assert collections.Counter(u['mode'] for u in narrative if u['category'] == 'Road encounters') == {'Classic': 72, 'Shared': 102, 'Deep End': 57}
assert len({u['news_hook_id'] for u in deep_roads}) == 57, 'repeated Deep End hook'
counts = collections.Counter(u['news_hook_id'] for u in narrative)
assert max(counts.values()) <= 6, ('hook budget', counts.most_common(10))
assert len([u for u in deep_roads if u['news_hook_id'].startswith('TRANS-RESEARCH-')]) == 13
assert 'trans rights' in by['MODE-D']['text'] and 'opt in' in by['MODE-D']['text']

# Flags require an editorial decision; they do not score humor or ban words.
heavy = re.compile(r'abortion|miscarriage|mifepristone|misoprostol|pregnan|\bIVF\b|embryo|\bFlock\b|stalk|fertility data|clinic.location|gender.affirm|massacre|internment|slavery|transgender|terroris', re.I)
boundary_flags = [{'id': u['id'], 'matches': sorted(set(m.group(0) for m in heavy.finditer(playable(u))))} for u in narrative if u['mode'] != 'Deep End' and heavy.search(playable(u))]
boundary_reviews = {
    'TOWN-33-C': 'Retained, unchanged Salt Lake City flag-adoption story: a city responds to a flag restriction by making Pride, trans and Juneteenth flags official. This lighter civic workaround fits Shared. Trans identity and representation are not themselves heavy content.'
}
for flag in boundary_flags:
    if flag['id'] in boundary_reviews:
        assert flag['matches'] == ['transgender']
        assert playable(by[flag['id']]) == playable(old[flag['id']]), ('reviewed boundary copy changed', flag['id'])
unresolved_boundary_flags = [f for f in boundary_flags if f['id'] not in boundary_reviews]
assert not unresolved_boundary_flags, ('mode boundary needs review', unresolved_boundary_flags)

for name, rows in parts.items():
    (ROOT / (name + '.json')).write_text(json.dumps(rows, ensure_ascii=False, indent=2) + '\n')

sourcebook_path = ROOT / 'political-sourcebook.json'
sourcebook = json.loads(sourcebook_path.read_text())
known_urls = {s['url'] for s in sourcebook['sources']}
for u in replacements.values():
    basis = u['political_basis']
    if basis['source_url'] not in known_urls:
        sourcebook['sources'].append({'id': u['news_hook_id'], 'subject': basis['subject'], 'fact_and_limits': basis['real_hook'], 'url': basis['source_url'], 'checked': '2026-09-14', 'editorial_mode': 'Deep End only' if u['mode'] == 'Deep End' else 'Classic or Shared'})
        known_urls.add(basis['source_url'])
sourcebook_path.write_text(json.dumps(sourcebook, ensure_ascii=False, indent=2) + '\n')

report = {
    'status': 'verified editorial pack; not a runtime deployment',
    'changed_narrative_packages': len(target_ids),
    'changed_functional_records': ['MODE-D'],
    'changed_ids': sorted(changed),
    'counts': {'narrative': 567, 'functional': 77, 'deep_only_narrative': 63},
    'distinct_specific_news_hooks': len(counts),
    'maximum_packages_per_specific_news_hook': max(counts.values()),
    'deep_road_distinct_hooks': len({u['news_hook_id'] for u in deep_roads}),
    'new_trans_hooks': 13,
    'ordered_mechanics_preserved': True,
    'locked_approved_samples_preserved': True,
    'shared_heavy_theme_flags': unresolved_boundary_flags,
    'reviewed_shared_identity_mentions': [{'id': f['id'], 'decision': boundary_reviews[f['id']]} for f in boundary_flags if f['id'] in boundary_reviews],
    'leading_topics_before': sorted([(r['subject'], r['before_count']) for r in registry], key=lambda x: -x[1])[:12],
    'hook_counts': dict(sorted(counts.items())),
    'hashes': {n: hashlib.sha256((ROOT / (n + '.json')).read_bytes()).hexdigest() for n in NAMES},
    'review_note': 'Specific hooks group aliases for the same event; broad areas such as labor or health can contain many distinct actions. Humor and factual framing were reviewed editorially, not machine-scored.'
}
(ROOT / 'validation-diversity-revision.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({k: report[k] for k in ['status', 'changed_narrative_packages', 'distinct_specific_news_hooks', 'maximum_packages_per_specific_news_hook', 'deep_road_distinct_hooks', 'new_trans_hooks']}))
