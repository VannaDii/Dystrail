import json
from pathlib import Path
ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[1]
def read(p):return json.loads((REPO/p).read_text())
brief=read('review/satire-fresh-2026-09-13/functional-brief.json')
profiles={x['town']:x for x in read('dystrail-web/static/assets/data/town-profiles.json')}
facts={x['town']:x for x in read('dystrail-web/static/assets/data/town-facts.json')}
store=read('dystrail-web/static/assets/data/store.json')
items={i['id']:i for c in store['categories'] for i in c['items']}
people=read('dystrail-web/static/assets/data/personas.json')
descriptions={
 'rations':'Adds three supplies per pack. A practical contribution to the argument that people need to eat.',
 'water':'Adds two supplies and carries the water-jug protection tag. Water prevents the extra sanity and health costs of prolonged heat.',
 'spare_tire':'One spare tire. Can be fitted for a tire breakdown; fitting consumes the spare.',
 'battery':'One spare battery. Can be fitted when the battery fails; it is not an alternator.',
 'alternator':'One spare alternator. Restores the failed charging component when fitted during its matching breakdown.',
 'fuel_pump':'One spare fuel pump. Fits a fuel-pump breakdown; it does not add fuel or repair unrelated faults.',
 'masks':'Carried masks protect against the smoke sanity cost and provide the game’s illness-resistance tag. Other smoke costs still apply.',
 'coats':'Warm coats protect against cold’s sanity cost and health damage from prolonged cold exposure.',
 'ponchos':'Ponchos protect against storm sanity loss. Other storm costs still apply.',
 'press_pass':'A unique permit item. Where the crossing accepts its tag, it can authorize passage without consuming a Receipt.',
 'legal_fund':'Adds one credibility. Protection from a tariff bulletin depends on the game actually carrying its legal-fund protection tag.'
}
portraits={
 'journalist':'Resourceful and credible, with a better chance of finding Receipts. Starts in Seattle.',
 'organizer':'Starts with stronger morale and more outside allies; also receives a small bribe discount. Starts in Portland.',
 'whistleblower':'Finds more Receipts but draws extra executive-order pressure. Starts in San Francisco.',
 'lobbyist':'Receives shop and bribe discounts through contacts. Starts in Los Angeles; this does not grant staff or official authority.',
 'staffer':'A balanced start without special bonuses or penalties. Starts in Sacramento.',
 'satirist':'Humor helps sustain sanity. Starts in San Diego.'
}
poses={
 'journalist':'An ordinary road traveler with a notebook and practical shoulder bag; blank press credential optional.',
 'organizer':'An ordinary traveler arranging a small stack of unlettered contact cards beside communal supplies.',
 'whistleblower':'An ordinary traveler holding a plain evidence folder close, with backup folders in a practical bag.',
 'lobbyist':'An ordinary traveler with a battered portfolio and a plain contacts notebook; no bodyguard, aide, private car or luxury office.',
 'staffer':'An ordinary traveler with a blank checklist and tangled charger, ready to do practical work.',
 'satirist':'An ordinary traveler with a blank notebook and amused expression beside the modest van.'
}
units=[]
for b in brief:
    u={**b,'status':'draft for user review','accepted':False}
    cat=b['category'];key=b['runtime_key'];sources=[]
    if cat=='Town profiles':
        x=profiles[key]
        u.update(text=f"{x['town']}, {x['state']}. Population: {x['population']:,} ({x['population_year']}). {x['attraction']['en']}",profile=x,
          evidence_status='Existing dated reference retained; population/source fields checked for completeness against the repository. No new live population claim.',
          source_checked=x.get('population_checked'))
        sources=[{'url':x['population_source'],'title':f"{x['town']} population ({x['population_year']})",'date':x.get('population_as_of',str(x['population_year'])),'qualification':'Dated population reference, not a claim about today’s population.'},{'url':x['attraction_source'],'title':f"{x['town']} attraction reference",'qualification':'Attraction description retained from the sourced project profile.'}]
        scene=f"A compact {x['town']} place vignette based on the profile’s sourced attraction: {x['attraction']['en']} Keep signs, maps, plaques and building lettering blank for overlays. Do not invent a disaster or closure."
    elif cat=='Endpoint-context records':
        x=facts[key]
        u.update(text=x['text']['en'],evidence_status='Existing dated endpoint context retained from the source pack; not counted as a playable intermediate-town conversation.',source_checked=x.get('checked'))
        sources=[{'url':x['source'],'title':f'{key} context source','date':x.get('checked'),'qualification':x.get('evidence_status','Existing dated source; distinguish proposals from realized changes.')}]
        scene=f"A restrained origin or destination establishing shot for {key}, using its sourced town-profile attraction where appropriate. An origin shows the six departing travelers; D.C. shows only actual arrivals. Keep all lettering separate."
    elif cat=='Shop-item descriptions':
        x=items[key]
        u.update(text=descriptions[key],base_price_cents=x['price_cents'],grants=x['grants'],tags=x['tags'],max_qty=x['max_qty'],unique=x['unique'],evidence_status='Item data and effect prose checked against local configuration and current simulation; effective price remains data-driven.')
        scene=f"Clean inventory cutout of {x['name']} on a plain background, matching the game’s pixel-art inventory style. No purchase, installation or consumption is implied; labels and quantity are editable overlays."
        if key=='legal_fund':u['integration_note']='store.json grants credibility but its tags array is empty. The tariff rule checks inventory.has_tag("legal_fund"). Do not promise purchase-based protection unless integration confirms the tag is carried.'
    elif cat=='Persona descriptions':
        x=people[key];u.update(text=portraits[key],start=x['start'],mods=x['mods'],score_mult=x['score_mult'],evidence_status='Checked against current persona configuration; actual user-supplied name and appearance remain authoritative.')
        scene=poses[key]
    else:
        deep=b['id']=='MODE-D'
        u.update(text=('The same road, a worse reality. Stranger encounters, harsher satire, and higher volatility. For a second trip into the mess.' if deep else 'Broken systems. Dubious advice. A fighting chance. The original trail, with plenty of political absurdity.'),evidence_status='Mode distinction retained from the user’s supplied game screen; detailed numeric effects stay with live game data.')
        scene=('The familiar van and road in the Deep End palette, with a recognizably stranger accumulation of official barriers and distorted civic scenery; no promise of a specific event.' if deep else 'The familiar ordinary van on a readable road with a modest supply stop and open horizon; political absurdity suggested through an oversized empty official sign.')
    u['sources']=sources
    u['scene']={'scene_id':b['id'].lower()+'-scene','description':scene,'overlays':[],
        'asset_notes':'No embedded text, lettering, logos, numbers or UI words. Any label, town name, quantity or caption is a separate localized overlay.'}
    units.append(u)
assert len(units)==77
(ROOT/'functional.json').write_text(json.dumps(units,ensure_ascii=False,indent=2)+'\n')

support=json.loads((ROOT/'support.json').read_text())
rules={
 'ACT-BARTERTIRE':'Trade3supplies for1spare tire. One community exchange per stop; requires3supplies. This packs a spare, not an automatic repair.',
 'ACT-BARTERBATTERY':'Trade4supplies for1spare battery. One community exchange per stop; requires4supplies. This packs a spare, not an automatic repair.',
 'ACT-BARTERSUPPLIES':'Trade1spare tire for5supplies. One community exchange per stop; requires a tire and supplies at or below15.'
}
for u in support:
    if u['family_id'] in rules:u['mechanics']=rules[u['family_id']].replace('Trade3','Trade 3').replace('Trade4','Trade 4').replace('Trade1','Trade 1').replace('for1','for 1').replace('for5','for 5').replace('3supplies','3 supplies').replace('4supplies','4 supplies').replace('5supplies','5 supplies').replace('1spare','1 spare').replace('requires3','requires 3').replace('requires4','requires 4').replace('below15','below 15')
(ROOT/'support.json').write_text(json.dumps(support,ensure_ascii=False,indent=2)+'\n')
