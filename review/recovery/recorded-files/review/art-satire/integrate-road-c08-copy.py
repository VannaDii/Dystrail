"""Integrate C08 narrative only, retaining unrelated copy and factual qualifications."""
import json
from pathlib import Path
root=Path(__file__).resolve().parents[2];r=root/'review/art-satire';web=root/'dystrail-web'
pack={u['id']:u for u in json.loads((r/'source-refresh-20260914/complete-pack.json').read_text())['units']};ids=['ENC-C08-'+v for v in 'ABC']
facts=["The January 20, 2025 executive order directed federal use of Gulf of America. It changed federal naming, not the coastline or universal international usage. The map club and dialogue are fictional.","Trump hosted leading investors in his meme coin at his Virginia golf club on May 22, 2025; about 220 were invited. Investment was tied to access; this is not a finding that a policy decision was purchased. The cafe and residents are fictional.","In February 2025 the administration tried to withdraw approval for New York congestion pricing, and Trump posted ‘LONG LIVE THE KING!’. On March 3, 2026 a federal district court rejected that termination on the asserted grounds. This describes the dated ruling, not every later appeal or a permanent end to the toll. The courier and picnic are fictional."]
lines=[['SAME COAST.','NEW FEDERAL NAME.'],['ORDINARY','VOTER'],['LONG LIVE','THE KING']]
en={}
for i,id in enumerate(ids):
 u=pack[id];v={'name':u['title'],'desc':u['setup'],'fact':facts[i],'overlay_0':u['scene']['overlays'][0]['text'],'overlay_line1':lines[i][0],'overlay_line2':lines[i][1]}
 for n,c in enumerate(u['choices']):v[f'choice_{n}']=c['label'];v[f'log_{n}']=c['outcome']
 if i==2:
  for key in ['desc','log_0','log_1','log_2']:v[key]=v[key].replace('his bill','her bill').replace('His bill','Her bill').replace('his toll','her toll').replace('his story','her story').replace('he says','she says').replace('his toy','her toy')
 en[id]=v
translations=json.loads((r/'road-c08-translations.json').read_text())
for path in (web/'i18n').glob('*.json'):
 original=json.loads(path.read_text());d=json.loads(path.read_text());values=translations.get(path.stem,en)
 for id in ids:assert set(values[id])==set(en[id]);d['visual_copy'][id]=values[id]
 check=json.loads(json.dumps(d))
 for id in ids:
  if id in original['visual_copy']:check['visual_copy'][id]=original['visual_copy'][id]
  else:check['visual_copy'].pop(id)
 assert check==original
 path.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n')
p=web/'static/assets/data/visual-sources.json';d=json.loads(p.read_text());original=json.loads(p.read_text())
for i,id in enumerate(ids):
 u=pack[id];s=u['sources'][0];d[id]={'title':s.get('title',u['source_hook']),'source':s['url'],'date':s.get('date',''),'checked':'2026-09-14','fact':facts[i],'qualification':u['political_basis']['real_hook'],'translations':{lang:translations[lang][id]['fact'] for lang in ['es','it','ar']}}
assert {k:v for k,v in d.items() if k not in ids}=={k:v for k,v in original.items() if k not in ids}
p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n');print('C08 en/es/it/ar copy integrated; unrelated locale/source values preserved.')
