"""Build bounded editorial assignments without modifying canonical copy."""
import collections, hashlib, json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
BASE = Path('/private/tmp/dystrail-before-diversity-20260914')
units = sum([json.loads((BASE / (name + '.json')).read_text()) for name in ['roads-a', 'roads-b', 'towns', 'support']], [])

def hook_id(u):
    b = u['political_basis']
    s = b['subject'].lower()
    if 'tariff' in s and 'tourism decline' not in s:
        return 'existing-2025-tariff-incidence'
    if ('park' in s and ('staff' in s or 'workforce' in s)):
        return 'existing-nps-firings'
    if 'gulf' in s and 'renam' in s:
        return 'existing-gulf-renaming'
    if 'history' in s and ('white house' in s or 'trump' in s or 'federal control' in s):
        return 'existing-history-directive'
    if 'nih' in s and ('cap' in s or 'overhead' in s):
        return 'existing-nih-overhead-cap'
    if 'maha' in s and ('citation' in s or 'nonexistent' in s):
        return 'existing-maha-false-citations'
    if any(t in s for t in ['local-food purchasing', 'local-food purchasing funds', 'cancellation of harvesters', 'cancellation of ozarks', 'canceled illinois food purchasing']):
        return 'existing-usda-local-food-cancellations'
    if 'medicaid' in s and ('work requirements' in s or 'community-engagement' in s):
        return 'existing-medicaid-snap-work-requirements'
    if 'penny' in s or 'pennies' in s:
        return 'existing-penny-production-end'
    if 'maryland' in s and 'paid-family-leave' in s:
        return 'existing-maryland-leave-delay'
    if 'no-tax-on-tips' in s or 'no tax on tips' in s:
        return 'existing-tip-overtime-deductions'
    if 'weather-service firings' in s or 'national weather service staffing' in s:
        return 'existing-nws-staffing-losses'
    return 'existing-' + hashlib.sha1(b['source_url'].encode()).hexdigest()[:12]

groups = collections.defaultdict(list)
for u in units:
    groups[hook_id(u)].append(u)

locked = {'ENC-C01-A', 'ENC-S13-A', 'CARE-01-A', 'CARE-02-A'}
targets = []
for key, group in groups.items():
    if len(group) <= 6:
        continue
    keep = [u for u in group if u['id'] in locked or u['category'] == 'Executive-order bulletins']
    assert len(keep) <= 6, key
    # Preserve scarce local sourced facts and a spread of scenes. The fixed
    # executive-order slots must continue describing their actual game order.
    rest = sorted([u for u in group if u not in keep], key=lambda u: (u['category'] != 'Town conversations', u['category'] != 'Road encounters', u['id']))
    families = {u['family_id'] for u in keep}
    categories = {u['category'] for u in keep}
    for u in rest:
        if len(keep) >= 6:
            break
        if u['family_id'] not in families and u['category'] not in categories:
            keep.append(u)
            families.add(u['family_id'])
            categories.add(u['category'])
    for u in rest:
        if len(keep) >= 6:
            break
        if u not in keep:
            keep.append(u)
    targets.extend(u for u in group if u not in keep)

target_ids = {u['id'] for u in targets}
remaining = collections.Counter(hook_id(u) for u in units if u['id'] not in target_ids)
registry = [{'news_hook_id': k, 'subject': group[0]['political_basis']['subject'], 'before_count': len(group), 'retained_count': remaining[k], 'member_ids': [u['id'] for u in group], 'source_urls': sorted({u['political_basis']['source_url'] for u in group})} for k, group in groups.items()]
(ROOT / 'diversity-hook-registry.json').write_text(json.dumps(registry, ensure_ascii=False, indent=2) + '\n')
(ROOT / 'diversity-overused-targets.json').write_text(json.dumps(sorted(targets, key=lambda u: u['id']), ensure_ascii=False, indent=2) + '\n')
for name, group in [('roads', [u for u in targets if u['category'] == 'Road encounters']), ('support', [u for u in targets if u['category'] != 'Road encounters'])]:
    (ROOT / f'diversity-{name}-targets.json').write_text(json.dumps(group, ensure_ascii=False, indent=2) + '\n')

# Writers may choose at most two additional uses of these existing hooks.
# Heavy hooks and headline families that were overused are not offered.
available = []
for key, group in groups.items():
    if len(group) <= 3 and all(u['mode'] != 'Deep End' for u in group):
        u = group[0]
        available.append({'news_hook_id': key, 'additional_budget': 2, 'existing_count': len(group), 'political_basis': u['political_basis'], 'sources': u['sources'], 'example_setup': u.get('setup',u.get('activation',u.get('onset',''))), 'town': u.get('town'), 'mode': u['mode']})
(ROOT / 'diversity-existing-hook-budget.json').write_text(json.dumps(available, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({'targets': len(targets), 'categories': dict(collections.Counter(u['category'] for u in targets)), 'available_existing_hooks': len(available), 'max_retained_hook': max(remaining.values())}))
