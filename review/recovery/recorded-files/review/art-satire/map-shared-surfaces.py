"""Map the entire brief to implementation surfaces without claiming art acceptance."""
import collections
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parents[2]
review = root / 'review/art-satire'
pack_path = review / 'source-refresh-20260914/complete-pack.json'
pack = json.loads(pack_path.read_text())
catalog = json.loads((review / 'asset-catalog.json').read_text())
source = {u['id']: u for u in pack['units']}
# A category-level implementation map, not a semantic claim that any generic
# painting satisfies its individual source description. Preserve that description.
surfaces = {
 'Gathering, work, barter, and rest': ('app/activities.rs', 'activity', 'Reviewed activity atlases; see unit acceptance.'),
 'Ally-departure vignettes': ('app/ally_loss.rs', 'ally_departure', 'External ally role/prop layers; never remove a traveler for ally-stat loss.'),
 'Crew-care scenarios': ('app/crew_care.rs', 'care', 'Shared clinic/shelter and active affected traveler poses; incident-specific vendor/prop scene still required.'),
 'Weather, illness, hunger, and exposure': ('app/workshop_events.rs', 'conditions', 'Reuse clock/weather masks and actual road; add the exact condition motif only where the source requires it.'),
 'Endpoint-context records': ('app/town_facts.rs', 'endpoint_context', 'Reuse profile/route context; endpoint vignette must agree with recorded origin or arrival.'),
 'Crossing packages': ('app/workshop.rs', 'crossing', 'Shared crossing structure plus exact offer/payment/refusal state and variant props.'),
 'Road encounters': ('components/ui/journey_scene/encounters.rs', 'road_encounter', 'Existing setting is only a fallback; compare source action/props before approving reuse.'),
 'Ending packages': ('components/ui/result_screen/layout.rs', 'ending', 'Actual location, cause and survivors; reusable van/interior motifs, with distinct typed ending props.'),
 'Hearing introductions and transitions': ('app/workshop.rs', 'hearing', 'Reuse integrated hearing room and participants; wording must follow the committed hearing report.'),
 'Mode descriptions': ('components/ui/mode_select.rs', 'mode', 'Shared road/van art; mode-specific source composition still needs comparison.'),
 'Character departure introductions': ('app/workshop.rs', 'opening', 'Reuse canonical cast, actual origin and van; persona-specific source props remain to be mapped.'),
 'Executive-order bulletins': ('app/policy_bulletin.rs', 'policy', 'Reviewed bulletin atlases and persistent acknowledgement; see unit acceptance.'),
 'Persona descriptions': ('components/ui/persona_select/tile.rs', 'persona', 'Reuse canonical cast atlas, profile stats and origin; no additional full scene implied.'),
 'Town profiles': ('app/town_profile.rs', 'town_profile', 'Reuse sourced profile UI; regional backdrop alone does not satisfy the attraction vignette.'),
 'Breakdown and repair packages': ('app/repair.rs', 'repair', 'Shared vehicle/workshop with matching failed/replaced part and offer/completed/cashless states.'),
 'Shop-item descriptions': ('components/ui/outfitting_store/view/item_card.rs', 'inventory', 'Reuse matching inventory cutout with native item name, price and quantity.'),
 'Town conversations': ('app/town_facts.rs', 'town_conversation', 'Shared town setting only where source agrees; resident occupation, gesture and prop require explicit mapping.'),
}
# Locate mode component from the real tree (the module may be a directory).
mode = root / 'dystrail-web/src/components/ui/mode_select.rs'
if not mode.exists():
    matches = list((root/'dystrail-web/src').rglob('*mode*'))
    raise AssertionError(f'Locate actual mode surface: {matches}')
units = []
for u in catalog['units']:
    path, surface, reuse = surfaces[u['category']]
    path = 'dystrail-web/src/' + path
    assert (root/path).is_file(), path
    assets = []
    if surface == 'persona':
        assets = [f"dystrail-web/static/img/cast-v2/{u['runtime_key']}.png"]
    elif surface == 'inventory':
        assets = [f"dystrail-web/static/img/items/{u['runtime_key']}-v1.png"]
    for asset in assets:
        assert (root/asset).is_file(), asset
    inactive = u['disposition'] == 'mechanics_dependency'
    accepted = u['production_status'] in ('integrated_render_reviewed','announcement_integrated_rendered_review')
    units.append({
      'id':u['id'], 'family':u['family'], 'runtime_key':u['runtime_key'],
      'disposition':u['disposition'], 'surface':surface, 'implementation':path,
      'current_acceptance':u['production_status'], 'accepted_asset_binding':u['asset'],
      'existing_reuse_candidates':assets, 'reuse_rule':reuse,
      'required_source_scene':u['source_scene'],
      'next_gate': 'mechanics_dependency_keep_inactive' if inactive else 'combined_release_review' if accepted else 'source_staging_and_runtime_review',
    })
assert len(units)==len(source)==644
assert {u['id'] for u in units}==set(source)
assert sum(u['disposition']=='mechanics_dependency' for u in units)==36
profiles={u['town']:u for u in json.loads((root/'dystrail-web/static/assets/data/town-profiles.json').read_text())}
facts={u['town']:u for u in json.loads((root/'dystrail-web/static/assets/data/town-facts.json').read_text())}
personas=json.loads((root/'dystrail-web/static/assets/data/personas.json').read_text())
store=json.loads((root/'dystrail-web/static/assets/data/store.json').read_text())
items={i['id']:i for c in store['categories'] for i in c['items']}
references=[]
for u in catalog['units']:
    if u['disposition']!='functional_reference': continue
    s=source[u['id']]; key=u['runtime_key']; category=u['category']; checks={}
    if category=='Town profiles':
        current=profiles[key]
        checks={field:current.get(field)==value for field,value in s['profile'].items()}
    elif category=='Endpoint-context records':
        checks={'english_context':facts[key]['text']['en']==s['text']}
    elif category=='Persona descriptions':
        checks={field:personas[key].get(field)==s.get(field) for field in ['start','mods','score_mult']}
    elif category=='Shop-item descriptions':
        checks={field:items[key].get(field)==s.get(field) for field in ['grants','tags','max_qty','unique']}
        checks['base_price_cents']=items[key]['price_cents']==s['base_price_cents']
    else:
        checks={'mode_identity':key in ['mode.classic','mode.deep']}
    references.append({'id':u['id'],'checks':checks,'remaining':'Narrative wording and rendered source-scene compliance require review; field matches are not visual acceptance.'})
result={'source_sha256':hashlib.sha256(pack_path.read_bytes()).hexdigest(),
 'method':'Source IDs and implementation surfaces mapped; reuse candidates are not accepted paintings.',
 'surface_counts':dict(collections.Counter(u['surface'] for u in units)),
 'units':units, 'functional_reference_checks':references,
 'reference_mismatches':[{'id':r['id'],'fields':[k for k,v in r['checks'].items() if not v]} for r in references if not all(r['checks'].values())]}
(review/'shared-surface-map.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'mapped':len(units),'reference_records_checked':len(references),'reference_mismatches':result['reference_mismatches'],'surface_counts':result['surface_counts']},indent=2))
