import copy,json,hashlib,re,subprocess,collections
from pathlib import Path
release=Path(__file__).resolve().parent;root=release.parents[2]
def baseline(path):return json.loads(subprocess.check_output(['git','show',f'f1ed4fe:{path}'],cwd=root,text=True))
def read(path):return json.loads((root/path).read_text())
def mechanics(events):
 events=copy.deepcopy(events)
 for e in events:
  for key in ['name','desc','satire_hook']:e.pop(key,None)
  for choice in e['choices']:choice.pop('label',None);choice['effects'].pop('log',None)
 return events
path='dystrail-web/static/assets/data/game.json';events=read(path);assert mechanics(events)==mechanics(baseline(path))
units=json.loads((release/'selected-units.json').read_text());road={u['runtime_key']:u for u in units if u['category']=='Road encounters'}
for event in events:
 u=road[event['id']];assert event['name']==u['title'] and event['desc']==u['setup']
 assert [(c['label'],c['effects']['log']) for c in event['choices']]==[(c['label'],c['outcome']) for c in u['choices']]
path='dystrail-web/static/assets/data/town-facts.json';towns=read(path);newnames={u['town'] for u in units if u['category']=='Town conversations'}
unchanged=[t for t in towns if t['town'] not in newnames];assert unchanged==[t for t in baseline(path) if t['town'] not in newnames]
changed=subprocess.check_output(['git','diff','--name-only'],cwd=root,text=True).splitlines()
assert not any(p.startswith(('dystrail-game/','dystrail-tester/','site/')) for p in changed)
assert not any('/static/img/' in p or p.endswith(('.css','Cargo.toml','Cargo.lock')) for p in changed)
assert len(events)==65 and sum(len(e['choices']) for e in events)==187
eng=read('dystrail-web/i18n/en.json');flat={}
def flatten(v,path=''):
 for k,value in v.items():
  key=f'{path}.{k}' if path else k
  if isinstance(value,dict):flatten(value,key)
  elif isinstance(value,str):flat[key]=value
flatten(eng)
locales={}
for lang in ['es','it','ar']:
 locale=read(f'dystrail-web/i18n/{lang}.json');english=flat.copy();flat={};flatten(locale);translated=flat.copy();flat=english
 required={k:v for k,v in english.items() if k.startswith(('workshop.','encounter_copy.'))}
 missing=sorted(set(required)-set(translated));errors=[]
 for key,value in required.items():
  if key in translated and collections.Counter(re.findall(r'\{\w+\}',value))!=collections.Counter(re.findall(r'\{\w+\}',translated[key])):errors.append(key)
 assert not missing and not errors,(lang,missing,errors)
 locales[lang]={'narrative_strings':len(required),'missing':len(missing),'placeholder_errors':errors}
report={'selected_packages':len(units),'encounters':len(events),'ordered_choices':sum(len(e['choices']) for e in events),'town_conversations':len(newnames),'endpoint_records_preserved':len(unchanged),'ordered_gameplay_effects_unchanged':True,'engine_files_unchanged':True,'scene_art_unchanged':True,'localization':locales,'deferred':'B/C variants, 12 proposed new encounter families, new mechanics and new art','source_revision':json.loads((release/'document-verification.json').read_text()).get('revision')}
(release/'content-validation.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
