"""Pin the latest editorial pack separately from the partially synchronized Doc."""
import copy, hashlib, json, shutil
from pathlib import Path

root = Path(__file__).resolve().parents[2]
review = root/'review/art-satire'
incoming = Path('/Users/vanna/Source/Dystrail/review/satire-fresh-2026-09-13')
source = root/'review/satire-fresh-2026-09-13'
snapshot = review/'source-refresh-20260914'
snapshot.mkdir(exist_ok=True)
old_path = snapshot/'previous-complete-pack.json'
if not old_path.exists():
    shutil.copy2(source/'complete-pack.json', old_path)
old = {u['id']:u for u in json.loads(old_path.read_text())['units']}
new = {u['id']:u for u in json.loads((incoming/'complete-pack.json').read_text())['units']}
assert len(old) == len(new) == 644 and old.keys() == new.keys()
editorial = {'title','setup','fact','commentary','speaker','sources','source_hook',
 'evidence_status','illustration','status','scene','scene_id','political_basis',
 'continuing','onset','activation','headline','epilogue','outcome','expiration',
 'recovery','final_ally','text','transitions','before_vote','exhausted','action',
 'cast_requirements','content_gate','content_topics','news_hook_id'}
def no_logs(value):
    if isinstance(value,dict): return {k:no_logs(v) for k,v in value.items() if k!='log'}
    if isinstance(value,list): return [no_logs(v) for v in value]
    return value
def mechanics(u):
    result={k:no_logs(v) for k,v in u.items() if k not in editorial|{'choices','outcomes'}}
    if 'choices' in u:
        result['choices']=[no_logs({k:v for k,v in c.items() if k not in {'label','outcome','roadside_outcome'}}) for c in u['choices']]
        result['roadside_outcome_slots']=[i for i,c in enumerate(u['choices']) if 'roadside_outcome' in c]
    if 'outcomes' in u: result['outcome_slots']=sorted(u['outcomes'])
    return result
for ident in old:
    assert mechanics(old[ident])==mechanics(new[ident]), ('mechanics drift',ident)
    assert old[ident].get('mode')==new[ident].get('mode'), ('mode drift',ident)
report=json.loads((incoming/'validation-diversity-revision.json').read_text())
changed=set(report['changed_ids'])
native=json.loads((review/'rest-workshop-native.json').read_text())
sections=json.loads((incoming/'doc-saved-sections.json').read_text())
manifest={t['key']:t for t in json.loads((incoming/'doc-section-manifest.json').read_text())}
tabs=[]
for section in sections:
    expected=[r['text'].strip() for r in section['rows'] if r['text'].strip()]
    actual=[r['text'].strip() for r in native['paragraphs'] if r.get('tabId')==manifest[section['key']]['tabId'] and r['text'].strip()]
    assert expected==actual, ('native mismatch',section['key'])
    tabs.append(dict(key=section['key'],paragraphs=len(actual),matches=True))
names=['complete-pack.json','narrative-pack.json','roads-a.json','roads-b.json','support.json','towns.json','functional.json','political-sourcebook.json','validation-diversity-revision.json','doc-saved-sections.json','doc-section-manifest.json','doc-diversity-sync-checkpoint.json','manual-integration-parts.json']
hashes={}
for name in names:
    shutil.copy2(incoming/name,snapshot/name)
    hashes[name]=hashlib.sha256((snapshot/name).read_bytes()).hexdigest()
    if name in ['complete-pack.json','narrative-pack.json','roads-a.json','roads-b.json','support.json','towns.json','functional.json','political-sourcebook.json']:
        shutil.copy2(snapshot/name,source/name)
catalog_path=review/'asset-catalog.json'
catalog=json.loads(catalog_path.read_text())
refresh=[]
for unit in catalog['units']:
    ident=unit['id']
    unit['source_scene']=copy.deepcopy(new[ident].get('scene',{}))
    if ident in changed:
        unit['latest_editorial_revision']='diversity-20260914'
        if unit.get('asset'):
            unit.setdefault('previous_production_status',unit['production_status'])
            unit['production_status']='source_refresh_required'
            refresh.append(ident)
catalog['source_sha256']=hashes['complete-pack.json']
catalog['native_workshop_verification']='review/art-satire/source-refresh-20260914/reconciliation.json'
catalog_path.write_text(json.dumps(catalog,ensure_ascii=False,indent=2)+'\n')
result=dict(status='latest local editorial pack pinned; native document partially synchronized',
    authoritative_for_implementation='complete-pack.json in this snapshot; no wholesale runtime import',
    revisionId=native['revisionId'],paragraphs=len(native['paragraphs']),tabs=tabs,
    records=len(new),changed_narrative=report['changed_narrative_packages'],changed_functional=['MODE-D'],
    ordered_mechanics_preserved=True,mode_membership_preserved=True,
    illustrated_units_requiring_refresh=sorted(refresh),changed_ids=sorted(changed),hashes=hashes,
    document_sync_note='The native document matches all 16 saved sections, but some newer local editorial replacements are awaiting document synchronization. This task does not write to the workshop.',
    runtime_note='Existing scenes retain their verified previous copy until each replacement is integrated and reviewed. Hearing mechanics and locale subtrees are not imported from editorial files.')
(snapshot/'reconciliation.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({k:result[k] for k in ['records','changed_narrative','ordered_mechanics_preserved','illustrated_units_requiring_refresh']}))
