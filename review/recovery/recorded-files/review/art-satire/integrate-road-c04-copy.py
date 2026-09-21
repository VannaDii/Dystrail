"""Update only reviewed C04 copy and source entries; keep unrelated locales intact."""
import json
from pathlib import Path
root=Path(__file__).resolve().parents[2]; review=root/'review/art-satire'; web=root/'dystrail-web'
pack={u['id']:u for u in json.loads((review/'source-refresh-20260914/complete-pack.json').read_text())['units']}
translations=json.loads((review/'road-c04-translations.json').read_text())
ids=['ENC-C04-'+v for v in 'ABC']
facts=[
 "NASA's Athena payload documentation says the lander's roughly horizontal orientation prevented the TRIDENT drill from reaching the lunar surface and completing its objectives. This was an uncrewed commercial mission; other instruments returned data.",
 "In April 2026 the FTC announced a complaint and proposed settlement concerning false US-origin claims for flag-display products. The announcement described allegations and an order requiring judicial approval, not a finding that all patriotic products were imported.",
 "A 2024 Florida law bars local governments from imposing certain workplace heat protections and water-break requirements. It does not make drinking water illegal or remove every federal obligation."
]
english={}
for i,unit in enumerate(ids):
 u=pack[unit]
 values={'name':u['title'],'desc':u['setup'],'fact':facts[i]}
 for n,c in enumerate(u['choices']):values.update({f'choice_{n}':c['label'],f'log_{n}':c['outcome']})
 if unit.endswith('-C'):
  values['desc']=values['desc'].replace('his chair','their chair')
  values.update(overlay_0=u['scene']['overlays'][0]['text'],overlay_line1='LOCAL HEAT RULES',overlay_line2='BLOCKED')
 english[unit]=values
for path in (web/'i18n').glob('*.json'):
 original=json.loads(path.read_text()); d=json.loads(path.read_text())
 for unit in ids:
  values=translations.get(path.stem,english)[unit]
  assert set(values)==set(english[unit]),(path.name,unit)
  assert all(values.values()),(path.name,unit)
  d['visual_copy'][unit]=values
 check=json.loads(json.dumps(d))
 for unit in ids:
  if unit in original['visual_copy']:check['visual_copy'][unit]=original['visual_copy'][unit]
  else:del check['visual_copy'][unit]
 assert check==original,path
 path.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n')
p=web/'static/assets/data/visual-sources.json'; sources=json.loads(p.read_text());original=json.loads(p.read_text())
for i,unit in enumerate(ids):
 u=pack[unit];s=u['sources'][0]
 sources[unit]={'title':s.get('title',u['source_hook']),'source':s['url'],'date':s.get('date',''),'checked':s.get('checked','2026-09-14'),'fact':facts[i],'qualification':s.get('qualification',s.get('claim','')),'translations':{lang:translations[lang][unit]['fact'] for lang in ['es','it','ar']}}
assert {k:v for k,v in sources.items() if k not in ids}=={k:v for k,v in original.items() if k not in ids}
p.write_text(json.dumps(sources,ensure_ascii=False,indent=2)+'\n')
print('C04 en/es/it/ar copy integrated with explicit English fallback; all unrelated locale/source values preserved.')
