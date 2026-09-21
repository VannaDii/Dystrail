import collections
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
BEFORE = Path('/private/tmp/dystrail-political-before-20260913')
NAMES = ['roads-a', 'roads-b', 'towns', 'support', 'functional']
parts = {n: json.loads((ROOT / (n + '.json')).read_text()) for n in NAMES}
old = {u['id']: u for n in NAMES for u in json.loads((BEFORE / (n + '.json')).read_text())}
units = [u for n in NAMES for u in parts[n]]
narrative = [u for n in NAMES[:-1] for u in parts[n]]
errors = []

# Only authored prose, source notes, scene direction and editorial status may
# differ. Every remaining value, including nested effects, stays authoritative.
prose_fields = {
    'title', 'setup', 'fact', 'commentary', 'speaker', 'sources', 'source_hook',
    'evidence_status', 'illustration', 'status', 'scene', 'scene_id',
    'political_basis', 'continuing', 'onset', 'activation', 'headline', 'epilogue',
    'outcome', 'expiration', 'recovery', 'final_ally', 'text', 'transitions',
    'before_vote', 'exhausted', 'action', 'cast_requirements'
}

def without_logs(value):
    if isinstance(value, dict):
        return {k: without_logs(v) for k, v in value.items() if k != 'log'}
    if isinstance(value, list):
        return [without_logs(v) for v in value]
    return value

def mechanical(u):
    result = {k: without_logs(v) for k, v in u.items() if k not in prose_fields | {'choices', 'outcomes'}}
    if 'choices' in u:
        result['choices'] = [without_logs({k: v for k, v in c.items() if k not in {'label', 'outcome', 'roadside_outcome'}}) for c in u['choices']]
        result['roadside_outcome_slots'] = [i for i, c in enumerate(u['choices']) if 'roadside_outcome' in c]
    if 'outcomes' in u:
        result['outcome_keys'] = sorted(u['outcomes'])
    return result

ids = [u['id'] for u in units]
if len(ids) != 644 or len(set(ids)) != 644 or set(ids) != set(old):
    errors.append({'identity': '644 unique original IDs required'})
mechanical_diffs = []
for u in units:
    a, b = mechanical(old[u['id']]), mechanical(u)
    if a != b:
        mechanical_diffs.append({'id': u['id'], 'fields': [k for k in a.keys() | b.keys() if a.get(k) != b.get(k)]})
    if u['category'] == 'Crew-care scenarios':
        for k in ['companion_lost', 'player_lost']:
            if u['outcomes'].get(k) != old[u['id']]['outcomes'].get(k):
                errors.append({'id': u['id'], 'changed_care_terminal_copy': k})
if mechanical_diffs:
    errors.append({'mechanical_differences': mechanical_diffs})
if parts['functional'] != json.loads((BEFORE / 'functional.json').read_text()):
    errors.append({'functional': 'The 77 reference records changed'})

approved = {
    'ENC-C01-A': {
        'title': 'Smaller government',
        'setup': 'DOGE fired the park’s cleaners. The remaining ranger hasn’t had lunch. She offers your crew gloves beside overflowing bins. “Congratulations. You’re the smaller government.”',
        'choices': [
            ['Share some food and help clear the bins', 'You share lunch and haul bags until your back aches. She hands you a bin liner. “Uniform allowance.”'],
            ['Photograph the mess for your D.C. case', 'You photograph the overflowing bins. She points at the flies. “Only department still recruiting.”']
        ]
    },
    'ENC-S13-A': {
        'title': 'China’s card declined',
        'setup': 'Trump says China pays the tariffs. At a roadside shop, your supplies cost $12. The cashier taps a Chinese flag against the card reader. Declined.',
        'choices': [
            ['Pay the $12', 'Your payment works. “Congratulations,” she says. “You’re China now.”'],
            ['Get the itemized quote', 'She circles the tariff. “China’s very generous with your money.”'],
            ['Keep the $12', 'You leave empty-handed. The tariff has successfully protected the supplies from Americans.']
        ]
    }
}
by = {u['id']: u for u in units}
for id, expected in approved.items():
    u = by[id]
    got = {'title': u['title'], 'setup': u['setup'], 'choices': [[c['label'], c['outcome']] for c in u['choices']]}
    if got != expected:
        errors.append({'id': id, 'approved_sample_changed': got})
for id in ['CARE-01-A', 'CARE-02-A']:
    if by[id]['setup'] != old[id]['setup']:
        errors.append({'id': id, 'approved_care_leadin_changed': True})

def prose(u):
    values = [u[k] for k in prose_fields - {'sources', 'scene', 'political_basis', 'source_hook', 'evidence_status', 'status', 'illustration', 'scene_id'} if k in u]
    values += [{k: c[k] for k in ['label', 'outcome', 'roadside_outcome'] if k in c} for c in u.get('choices', [])]
    values += [u.get('outcomes', {})]
    return json.dumps(values, ensure_ascii=False)

politics = re.compile(r'Trump|DOGE|MAHA|Congress|White House|tariff|Gulf of (?:Mexico|America)|Sharpie|\bNIH\b|Medicaid|SNAP|super.?PAC|campaign|billionaire|Department of War|Education Department|education department|DEI|bombing.plan|airstrike|voter|voting|ballot|legislatur|legislat|governor|federal|USDA|RFK|HHS|ICE|DHS|CBP|border|Reagan|DeSantis|Polis|Musk|Medicare|Supreme Court|politic|immigra|Senate|senator|school voucher|school vouchers|HB \d|SB \d|executive order|constitutional|abortion|Citizens United|Dobbs|congestion pric|congestion toll|Nixon|state ban', re.I)
unrevised = []
political_review_flags = []
for u in narrative:
    text = prose(u)
    basis = u.get('political_basis')
    if not basis or not basis.get('source_url') or not basis.get('real_hook'):
        unrevised.append(u['id'])
    if not politics.search(text):
        political_review_flags.append({'id': u['id'], 'lead': u.get('setup') or u.get('onset') or u.get('epilogue') or u.get('activation')})
    if re.search(r'\b(?:water|food|road|supply|supplies)\s+packs?\b', text, re.I):
        errors.append({'id': u['id'], 'pack_language': True})
    if re.search(r'\b(?:TODO|TBD|lorem ipsum|insert joke|writing placeholder)\b', text, re.I):
        errors.append({'id': u['id'], 'placeholder': True})
    if re.search(r'\b(?:fuck(?:ing|ers?)?|shit|bullshit|asshole)\b', text, re.I):
        errors.append({'id': u['id'], 'profanity': True})
    if not u.get('scene') or not u['scene'].get('asset_notes'):
        errors.append({'id': u['id'], 'missing_scene_or_text_guard': True})
    for c in u.get('choices', []):
        if c.get('effects', {}).get('log', c.get('outcome')) != c.get('outcome'):
            errors.append({'id': u['id'], 'stale_effect_log': c.get('label')})

report = {
    'status': 'complete structural verification' if not errors and not unrevised else 'in progress or corrections required',
    'editorial_status': 'Political satire revision for user review; humor is not automatically certified',
    'narrative_count': len(narrative), 'functional_count': len(parts['functional']),
    'politically_rewritten': len(narrative) - len(unrevised),
    'mechanical_differences': mechanical_diffs,
    'errors': errors, 'unrevised': unrevised,
    'political_review_flags': political_review_flags,
    'mode_counts': dict(collections.Counter(u['mode'] for u in narrative if u['category'] == 'Road encounters')),
    'category_counts': dict(collections.Counter(u['category'] for u in narrative)),
    'source_hashes': {n: hashlib.sha256((ROOT / (n + '.json')).read_bytes()).hexdigest() for n in NAMES}
}
(ROOT / 'validation-political-revision.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({'status': report['status'], 'politically_rewritten': report['politically_rewritten'], 'mechanical_differences': mechanical_diffs, 'errors': errors, 'political_review_flag_count': len(political_review_flags)}, ensure_ascii=False))
