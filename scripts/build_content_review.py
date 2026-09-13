"""Generate the review inventory directly from shipped game content."""
import csv
import argparse
import json
import re
from collections import Counter
from datetime import date
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--source-root', type=Path, default=WORKSPACE,
                    help='Read content from an immutable release source tree.')
args = parser.parse_args()
ROOT = args.source_root.resolve()
DATA = ROOT / 'dystrail-web/static/assets/data'
OUT = WORKSPACE / 'docs/ux/review-2026-09-11/implementation/content'
OUT.mkdir(parents=True, exist_ok=True)
read = lambda path: json.loads(path.read_text())
events = read(DATA / 'game.json')
sources = read(DATA / 'satire-sources.json')
towns = read(DATA / 'town-facts.json')
profiles = {p['town']: p for p in read(DATA / 'town-profiles.json')}
routes = read(ROOT / 'dystrail-game/data/routes.json')
copy = read(ROOT / 'dystrail-web/i18n/en.json')
art_source = (ROOT / 'dystrail-web/src/components/ui/journey_scene/encounters.rs').read_text()
assignments = {}
match_arm = r'((?:"[a-z0-9_]+"\s*\|\s*)*"[a-z0-9_]+")\s*=>\s*(?:"([a-z0-9-]+)"|\{\s*"([a-z0-9-]+)"\s*\})'
for ids, direct, braced in re.findall(match_arm, art_source.split('#[cfg(test)]', 1)[0]):
    scene = direct or braced
    for event_id in re.findall(r'"([a-z0-9_]+)"', ids):
        assert event_id not in assignments, f'Duplicate setting assignment: {event_id}'
        assignments[event_id] = scene
event_ids = {event['id'] for event in events}
assert set(assignments) == event_ids, (
    f'Encounter setting mismatch: missing {sorted(event_ids - assignments.keys())}; '
    f'unknown {sorted(assignments.keys() - event_ids)}'
)
for event in events:
    event['scene'] = assignments[event['id']]
    event.setdefault('modes', ['classic', 'deep', 'deep_end'])
    event['new_this_round'] = event['id'].startswith(('sat_', 'west_'))
    assert event['satire_hook'] in sources
scene_counts = Counter(e['scene'] for e in events)
for town in towns:
    town['profile'] = profiles[town['town']]
    town['routes'] = [{ 'id': r['id'], 'mile': s['mile'], 'region': s['region'], 'road_scene': s['scene'],
                       'stage': 'Origin' if s['mile'] == 0 else 'Destination' if s['name'] == 'D.C.' else 'Town stop',
                       'barter_kind': (s['mile'] // 10) % 3}
                     for r in routes for s in r['stops'] if s['name'] == town['town']]
care_hooks = ['maha','weather','bullets','maha','names','tariffs','signal','food']
systems = [
 {'name':'Collect evidence','stage':'26 encounter choices across all six regions','cost':'Other costs are listed with each choice.','effect':'Receipts +1 guaranteed; chance of another is 10% plus character and news-diet modifiers, bounded to 0–100%. Each held receipt adds 8 journey-score points.','availability':'Explicitly offered documentation, audit and testimony choices; actual gains are shown in the shared HUD, Outcome, The Trail and Journal.'},
 {'name':'Forage with a local guide','stage':'Camp outside town','cost':'2 hours','effect':'Supplies +2; Sanity +1 (cap 10).','availability':'Once every 3 game days, while supplies ≤18. No unresolved encounter, breakdown or crew-care decision.'},
 {'name':'Help glean a farm plot','stage':'Camp outside town','cost':'2 hours; Health −1','effect':'Supplies +4.','availability':'Shares the forage cooldown; supplies ≤16 and Health >1.'},
 {'name':'Work for food','stage':'Town','cost':'3 hours; Sanity −1','effect':'Supplies +4.','availability':'One work offer per stop, supplies ≤16 and Sanity >0.'},
 {'name':'Take a paid unloading shift','stage':'Town','cost':'3 hours; Sanity −1','effect':'Cash +$18.','availability':'Shares the town-work allowance; Sanity >0.'},
 {'name':'Community barter A','stage':'Town','cost':'30 minutes; Supplies −3','effect':'Spare Tire +1.','availability':'One barter per stop.'},
 {'name':'Community barter B','stage':'Town','cost':'30 minutes; Supplies −4','effect':'Battery +1.','availability':'One barter per stop.'},
 {'name':'Community barter C','stage':'Town','cost':'30 minutes; Spare Tire −1','effect':'Supplies +5.','availability':'One barter per stop, supplies ≤15.'},
 {'name':'Talk to locals','stage':'Town','cost':'30 minutes on the first rewarded visit; revisits are free.','effect':'The button previews Credibility +1, Receipts +1 or Allies +1. Towns vary the reward; credibility at 20 or allies at 50 yield a receipt instead.','availability':'Conversation is always accessible. One benefit per stop; the actual claimed reward persists and is struck out on later visits.'},
 {'name':'Fit an onboard spare','stage':'Breakdown','cost':'1 hour; one matching Tire, Battery, Alternator or Fuel Pump','effect':'Clear breakdown; Vehicle condition +8% (cap 100%); 2-day breakdown cooldown.','availability':'Matching spare must be aboard.'},
 {'name':'Order a replacement','stage':'Breakdown','cost':'90 minutes; Tire $15 / Battery $22 / Alternator $21 / Fuel Pump $17, character discount applies. Add $10 delivery outside town.','effect':'Clear breakdown; Vehicle condition +8% (cap 100%); 2-day breakdown cooldown.','availability':'Enough cash for the entire quoted price.'},
 {'name':'Barter for the named part','stage':'Breakdown','cost':'2 hours; Supplies −4','effect':'Clear breakdown; Vehicle condition +8% (cap 100%); 2-day breakdown cooldown.','availability':'At least 4 supplies.'},
 {'name':'Call local radio; work for a repair','stage':'Breakdown','cost':'4 hours; Sanity −2; Morale −1 (both floor 0)','effect':'Clear breakdown; Vehicle condition +3% (cap 100%); 2-day breakdown cooldown.','availability':'Always available during an unresolved breakdown, including zero cash and no spares.'},
]
care = [{'id': n, 'reason': copy['trail'][f'care_reason_{n}'], 'satire_hook': h,
         'choices': [
          {'label':copy['journey']['care_action'].split(' · ',1)[0],'cost':'1 hour; Supplies −2','effect':'Clear strain; Morale +1 (cap 10).'},
          {'label':copy['journey']['shelter_action'].split(' · ',1)[0],'cost':'1 hour; Morale −1','effect':'Crew member departs and disappears from all later van/cast scenes. Unavailable for the player character.'},
          {'label':copy['journey']['defer_action'].split(' · ',1)[0],'cost':'1 hour; Sanity −1','effect':'Strain persists. At critical strain, the member dies and Morale −3. Player death ends the run. The forecast identifies this before selection.'}]} for n,h in enumerate(care_hooks)]
inventory = {'generated':date.today().isoformat(),'encounters':events,'sources':sources,'towns':towns,
 'routes':[{'id':r['id'],'total_miles':r['total_miles'],'stops':r['stops']} for r in routes],
 'scene_usage':dict(scene_counts),'systems':systems,'care':care,
 'ally_departures':[copy['ally_loss'][f'reason_{i}'] for i in range(6)],
 'shop':read(DATA/'store.json'),'camp':read(DATA/'camp.json'),
 'weather':read(DATA/'weather.json'),'crossings':read(DATA/'crossings.json'),'boss':read(DATA/'boss.json')}
(OUT/'inventory.json').write_text(json.dumps(inventory,ensure_ascii=False,indent=2)+'\n')
with (OUT/'encounter-options.csv').open('w',newline='') as f:
    writer=csv.writer(f);writer.writerow(['id','title','regions','modes','scene','political_hook','choice','effects','outcome','source'])
    for e in events:
        for c in e['choices']:
            writer.writerow([e['id'],e['name'],' / '.join(e['regions']),' / '.join(e['modes']),e['scene'],e['satire_hook'],c['label'],json.dumps({k:v for k,v in c['effects'].items() if k!='log'}),c['effects'].get('log',''),sources[e['satire_hook']]['source']])
html = '''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Dystopian Trail · Content inventory</title>
<style>
:root{color-scheme:dark;--bg:#091620;--panel:#112936;--line:#3f606c;--ink:#f9efd2;--muted:#b8cdd3;--gold:#f6ce69}*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--ink);font:16px/1.6 system-ui,sans-serif}a{color:var(--gold)}button,select,input{font:inherit;min-height:44px;background:#173440;color:var(--ink);border:1px solid var(--line);border-radius:5px;padding:8px 12px}button,select{cursor:pointer}button[aria-pressed=true]{background:var(--gold);color:#10212b}button:focus-visible,a:focus-visible,select:focus-visible,input:focus-visible{outline:3px solid #55dacb;outline-offset:3px}header,main{max-width:1280px;margin:auto;padding:32px 24px}header{padding-bottom:20px}h1,h2,h3{font-family:Georgia,serif;line-height:1.2}h1{font-size:clamp(30px,5vw,56px);margin:12px 0}h2{font-size:28px}h3{font-size:22px;margin:0 0 8px}.eyebrow{color:var(--gold);font-size:12px;font-weight:800;letter-spacing:.12em;text-transform:uppercase}.intro{max-width:80ch;color:var(--muted)}.metrics{display:flex;gap:32px;flex-wrap:wrap;margin:24px 0}.metrics strong{font:700 36px Georgia,serif;display:block}.metrics span{font-size:13px;color:var(--muted)}nav{display:flex;gap:8px;flex-wrap:wrap}.filter{display:grid;grid-template-columns:2fr 1fr 1fr;gap:14px;margin:20px 0}.filter label{display:grid;gap:5px;font-size:13px}.cards{display:grid;gap:24px}.entry{border-top:1px solid var(--line);padding-top:24px;display:grid;grid-template-columns:240px minmax(0,1fr);gap:24px}.thumb{width:100%;aspect-ratio:2;image-rendering:pixelated;background:var(--panel);border:1px solid var(--line)}.art-note,.meta,.source,.outcome{font-size:13px;color:var(--muted)}.meta{margin:8px 0}.badge{display:inline-block;color:var(--gold);font-size:11px;border:1px solid var(--line);padding:0 7px;margin-left:8px}.choices{padding:0;list-style:none;margin:20px 0}.choices li{padding:12px 0;border-top:1px solid #284653;display:grid;grid-template-columns:minmax(0,1fr) minmax(0,1fr);gap:6px 20px}.choices .outcome{grid-column:1/-1;margin:0}.effects{color:#99dfc4;font-size:14px}.hook{background:#0d202b;border-left:3px solid var(--gold);padding:12px 16px;font-size:14px}.hook p{margin:0 0 8px}.hook small{color:var(--muted)}blockquote{margin:14px 0;padding:12px 20px;border-left:3px solid var(--gold);background:var(--panel)}table{border-collapse:collapse;width:100%;font-size:14px}th,td{text-align:left;border-bottom:1px solid var(--line);padding:12px;vertical-align:top}th{color:var(--gold)}.table-scroll{overflow-x:auto}.opportunity{padding:18px 22px;background:#26362e;border:1px solid #577862;margin:24px 0}.opportunity p{margin:8px 0}.catalog{display:grid;grid-template-columns:repeat(3,1fr);gap:20px}.catalog article{border:1px solid var(--line);padding:15px;background:var(--panel)}.catalog h3{font-size:18px;margin-top:12px}footer{max-width:1280px;margin:40px auto;padding:24px;border-top:1px solid var(--line);color:var(--muted);font-size:13px}[hidden]{display:none}@media(max-width:700px){.entry{grid-template-columns:1fr}.entry aside{max-width:340px}.filter{grid-template-columns:1fr}.choices li{grid-template-columns:1fr}.choices .outcome{grid-column:auto}.catalog{grid-template-columns:1fr}.metrics{gap:18px}header,main{padding:22px 18px}}
</style><header><a href="../">← Implementation review</a><p class="eyebrow">Dystopian Trail · Editorial review · September 11, 2026</p><h1>Every stop. Every choice.<br>Every political hook.</h1><p class="intro">The shipped content, its actual effects, its setting, and the documented incident behind the joke. Fictional encounters and resident commentary are separate from sourced facts. Search a theme, region, town, or scene to see where the satire repeats and where it can go further.</p><div class="metrics"><div><strong>__ENCOUNTERS__</strong><span>encounters · __NEW__ new</span></div><div><strong>__CHOICES__</strong><span>encounter choices</span></div><div><strong>__TOWNS__</strong><span>real towns</span></div><div><strong>__HOOKS__</strong><span>documented political hooks</span></div></div><nav aria-label="Inventory sections"><button data-view="encounters" aria-pressed="true">Encounters</button><button data-view="towns" aria-pressed="false">Towns</button><button data-view="systems" aria-pressed="false">Care & supplies</button><button data-view="art" aria-pressed="false">Scenes & gaps</button><a href="inventory.json" download>Complete JSON</a><a href="encounter-options.csv" download>Choices CSV</a></nav></header>
<main><div class="filter"><label>Search<input id="search" type="search" placeholder="Trump, tariff, raw milk, Madison…"></label><label>Region<select id="region"><option value="">All regions</option><option>PacificCoast</option><option>MountainWest</option><option>Southwest</option><option>Heartland</option><option>RustBelt</option><option>Beltway</option></select></label><label>Political hook<select id="hook"><option value="">All hooks</option></select></label></div><p id="count" class="meta" role="status"></p><div id="content"></div></main><footer>Generated from the shipped JSON and explicit scene assignments. Choice deltas are base values: stat caps and character modifiers still apply. Known cash and supply costs are checked before an option can be selected. Encounter selection favors unseen eligible events before repeating them. Sources checked September 11, 2026. The historical dates are part of the record, not claims that every policy remains in effect today.</footer>
<script id="inventory" type="application/json">__DATA__</script><script>
const D=JSON.parse(document.getElementById('inventory').textContent),target=document.getElementById('content');let view='encounters';
const counted=(n,label)=>`${n} ${label}${n===1?'':'s'}`;
const esc=s=>String(s??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const labels={sanity:'Sanity',credibility:'Credibility',supplies:'Supplies',hp:'Health',morale:'Morale',allies:'Allies',cash_cents:'Cash',receipts:'Receipts'};
const cells={'enc-media-workshop':0,'enc-community':1,'enc-bridge':2,'enc-service':3,'enc-civic':4,'enc-checkpoint':5,'enc-clinic':6,'enc-radio':7,'enc-street':8,'enc-convoy':9,'milk-classic':10,'enc-night-briefing':11};
const interiorCells={'enc-motel':0,'enc-cafe':1,'enc-library':2,'enc-museum':3,'enc-farm-office':4,'enc-service-counter':5};
const thumb=(name,town=false,road='',region='')=>{
 let n=cells[name],cols=3,rows=4,atlas='encounter-settings-v2';
 if(town){const west=['PacificCoast','MountainWest','Southwest'].includes(name);n=west?({PacificCoast:road==='open-california-hills'?1:0,MountainWest:2,Southwest:3}[name]):({Heartland:0,RustBelt:1,Beltway:2}[name]||0);cols=2;rows=3;atlas=west?'western-settings-v1':'journey-settings-v1';}
 else if(Object.hasOwn(interiorCells,name)){n=interiorCells[name];rows=2;atlas='encounter-settings-v3';}
 else if(name==='enc-rest-area'){const west=['Southwest','MountainWest'].includes(region);n=region==='Southwest'?4:5;cols=2;rows=3;atlas=west?'western-settings-v1':'journey-settings-v1';}
 if(!Number.isInteger(n)||n<0||n>=cols*rows)throw new Error(`Unknown setting thumbnail: ${name}`);
 const w=1536/cols,h=1024/rows;return `<svg class="thumb" style="aspect-ratio:${w}/${h}" viewBox="${n%cols*w} ${Math.floor(n/cols)*h} ${w} ${h}" role="img" aria-label="${esc(name)} setting"><image href="/play/static/img/journey/${atlas}.png" width="1536" height="1024"/></svg>`;
};
const effects=e=>Object.entries(e).filter(([k,v])=>k!=='log'&&v).map(([k,v])=>k==='add_receipt'?'Receipts +1':k==='use_receipt'?'Receipts −1':k==='rest'?'Rest day':k==='travel_bonus_ratio'?`Travel credit +${Math.round(v*100)}%`:`${esc(labels[k]||k)} ${v>0?'+':'−'}${k==='cash_cents'?'$'+(Math.abs(v)/100).toFixed(0):Math.abs(v)}`).join(' · ')||'No direct stat change';
const hook=id=>{const s=D.sources[id];if(!s)return '';return `<div class="hook"><p><strong>${esc(s.title)}</strong> · ${esc(s.date)}</p><p>${esc(s.fact)}</p><a href="${esc(s.source)}" target="_blank" rel="noopener">Read the source ↗</a>${s.qualification?`<p><small>${esc(s.qualification)}</small></p>`:''}</div>`};
for(const [id,s]of Object.entries(D.sources))document.getElementById('hook').insertAdjacentHTML('beforeend',`<option value="${id}">${esc(s.title)}</option>`);
const search=document.getElementById('search'),region=document.getElementById('region'),source=document.getElementById('hook');
function matches(item,regions,id){return(!search.value||JSON.stringify(item).toLowerCase().includes(search.value.toLowerCase()))&&(!region.value||regions.includes(region.value))&&(!source.value||id===source.value)}
function render(){let list=[];if(view==='encounters'){
 list=D.encounters.filter(e=>matches(e,e.regions,e.satire_hook));target.innerHTML='<div class="cards">'+list.map(e=>`<article class="entry"><aside>${thumb(e.scene,false,'',e.regions[0])}<p class="art-note">${esc(e.scene)}<br>Shared by ${D.scene_usage[e.scene]} encounter${D.scene_usage[e.scene]===1?'':'s'} · active crew cast</p></aside><div><h2>${esc(e.name)}${e.new_this_round?'<span class="badge">NEW</span>':''}</h2><p class="meta">${esc(e.regions.join(' / '))} · ${esc(e.modes.join(' / '))} · ${esc(e.id)} · weight ${e.weight}</p><p>${esc(e.desc)}</p><ol class="choices">${e.choices.map((c,i)=>`<li><strong>${i+1}. ${esc(c.label)}</strong><span class="effects">${effects(c.effects)}</span><p class="outcome">After choosing: ${esc(c.effects.log||'No additional narrative.')}</p></li>`).join('')}</ol>${hook(e.satire_hook)}</div></article>`).join('')+'</div>';
}else if(view==='towns'){
 list=D.towns.filter(t=>matches(t,t.routes.map(r=>r.region),t.satire_hook));target.innerHTML='<div class="opportunity"><strong>Local facts, regional artwork.</strong><p>Each fact belongs to its exact town. The seven town backdrops are regional illustrations, not depictions of every town’s actual streets. All __TOWNS__ conversations have sourced administration-related local or regional context. Proposals, estimates, interruptions and documented effects are distinguished in each record. Population and visitor information are presented separately in the town header.</p></div><div class="cards">'+list.map(t=>`<article class="entry"><aside>${thumb(t.routes[0].region,true,t.routes[0].road_scene)}<p class="art-note">${esc(t.routes[0].region)} town setting<br>Player portrait + one of six residents</p></aside><div><h2>${esc(t.town)}</h2><p class="meta">${t.routes.map(r=>`${esc(r.id)} · ${r.mile} mi · ${esc(r.stage)}`).join('<br>')}</p><p>${esc(t.text.en)}</p><p class="source"><a href="${esc(t.source)}" target="_blank" rel="noopener">Local source ↗</a> · checked ${t.checked}</p><blockquote>${esc(t.comment.en)}<br><small>Fictional resident commentary</small></blockquote><p class="meta">At town stops: supply shop, three selectable barter offers with one exchange per stop, one paid or food-for-work shift, repeatable local conversation, rest, depart. Origin and destination are not additional mid-route shopping stops.</p><p class="meta">Record status: ${esc(t.evidence_status)}</p><p><strong>${esc(t.profile.state)} · Population estimate (${esc(t.profile.population_year)}): ${Number(t.profile.population).toLocaleString()}</strong></p><p>${esc(t.profile.attraction.en)}</p><p class="source"><a href="${esc(t.profile.population_source)}" target="_blank" rel="noopener">Census source</a> · <a href="${esc(t.profile.attraction_source)}" target="_blank" rel="noopener">Attraction source</a></p></div></article>`).join('')+'</div>';
}else if(view==='systems'){
 list=D.systems;target.innerHTML='<h2>Gather, trade, earn, repair</h2><div class="table-scroll"><table><thead><tr><th>Action</th><th>Cost</th><th>Result / availability</th></tr></thead><tbody>'+D.systems.filter(s=>!search.value||JSON.stringify(s).toLowerCase().includes(search.value.toLowerCase())).map(s=>`<tr><td><strong>${esc(s.name)}</strong><br>${esc(s.stage)}</td><td>${esc(s.cost)}</td><td>${esc(s.effect)}<br><span class="meta">${esc(s.availability)}</span></td></tr>`).join('')+'</tbody></table></div><h2>Named crew incidents</h2><p class="intro">Eight persisted fictional reasons. The next unresolved strain check follows the same person; an absent crew member never reappears in the van. All care actions advance the clock by one hour.</p><div class="cards">'+D.care.filter(c=>matches(c,['PacificCoast','MountainWest','Southwest','Heartland','RustBelt','Beltway'],c.satire_hook)).map(c=>`<article><h3>${esc(c.reason)}</h3><ol class="choices">${c.choices.map(x=>`<li><strong>${esc(x.label)}</strong><span>${esc(x.cost)}</span><p class="outcome">${esc(x.effect)}</p></li>`).join('')}</ol>${hook(c.satire_hook)}</article>`).join('')+'</div><h2>Ally departures</h2><p>Six fictional messages explain ordinary ally attrition. Every loss pauses for acknowledgment and stays in the journal; encounter-specific losses retain their own explanation.</p><div class="cards">'+D.ally_departures.map(s=>`<article><p>${esc(s)}</p></article>`).join('')+'</div><h2>Every shop item</h2><p>Default cart: 4 rations, 2 water, 1 tire, 1 battery, 1 mask. $71 before character discounts; 16 supplies. The player may remove everything. Capacity: 20 supplies.</p><div class="table-scroll"><table><thead><tr><th>Item</th><th>Price</th><th>Grants</th></tr></thead><tbody>'+D.shop.categories.flatMap(c=>c.items).map(i=>`<tr><td>${esc(i.name)}</td><td>$${(i.price_cents/100).toFixed(0)}</td><td>${esc(JSON.stringify(i.grants))}</td></tr>`).join('')+'</tbody></table></div><p class="source">Complete camp, crossing, weather and boss configuration is included in the <a href="inventory.json">JSON inventory</a>.</p>';
}else{
 list=Object.entries(D.scene_usage);target.innerHTML='<div class="opportunity"><strong>Next editorial and art opportunities</strong><p>__ENCOUNTERS__ encounters use __COMPOSITIONS__ setting compositions. The most reused settings are __BUSIEST__. New encounter text is not a claim of __NEW__ bespoke new illustrations.</p><p>Geographically matched town landmarks, larger pools of local NPC dialogue, more documented local policy consequences, and alternate outcomes per political hook would broaden variety further. All shipped encounters already have a non-road setting and a source hook.</p><p>Road travel uses twelve geographic scenery variants, persistent scrolling, weather effects, and six independently rendered seats. Night, heat and cold treatments are applied from the actual game state. The periodic map follows each character’s own accurately traced route to D.C.</p></div><div class="catalog">'+list.map(([scene,n])=>`<article>${thumb(scene)}<h3>${esc(scene)}</h3><p>${n} encounters use this composition.</p><button data-scene="${scene}">Review those encounters</button></article>`).join('')+'</div><h2>Route origins</h2><div class="table-scroll"><table><thead><tr><th>Character</th><th>Route</th><th>Distance</th></tr></thead><tbody>'+D.routes.map(r=>`<tr><td>${esc(r.id)}</td><td>${r.stops.map(s=>esc(s.name)).join(' → ')}</td><td>${Math.round(r.total_miles)} mi</td></tr>`).join('')+'</tbody></table></div>';
}document.getElementById('count').textContent=view==='encounters'?`${counted(list.length,'encounter')} · ${counted(list.reduce((n,e)=>n+e.choices.length,0),'choice')}`:view==='towns'?counted(list.length,'town'):'';for(const b of document.querySelectorAll('[data-view]'))b.setAttribute('aria-pressed',String(b.dataset.view===view));document.querySelector('.filter').hidden=view==='art';region.parentElement.hidden=view==='systems';source.parentElement.hidden=view==='systems';}
document.querySelector('nav').addEventListener('click',e=>{const b=e.target.closest('[data-view]');if(b){view=b.dataset.view;search.value='';region.value='';source.value='';render();}});target.addEventListener('click',e=>{const b=e.target.closest('[data-scene]');if(b){view='encounters';search.value=b.dataset.scene;render();document.querySelector('main').scrollIntoView();}});for(const el of [search,region,source])el.addEventListener('input',render);render();
</script></html>'''
for key, value in {'ENCOUNTERS':len(events),'NEW':sum(e['new_this_round'] for e in events),'CHOICES':sum(len(e['choices']) for e in events),'TOWNS':len(towns),'HOOKS':len(sources),'COMPOSITIONS':len(scene_counts),'BUSIEST':' and '.join(f'{scene} ({count} encounters)' for scene,count in scene_counts.most_common(2))}.items():
    html=html.replace(f'__{key}__',str(value))
(OUT/'index.html').write_text(html.replace('__DATA__',json.dumps(inventory,ensure_ascii=False).replace('<','\\u003c')))
print(f'{len(events)} encounters, {sum(len(e["choices"]) for e in events)} choices, {len(towns)} towns, {len(scene_counts)} compositions')
