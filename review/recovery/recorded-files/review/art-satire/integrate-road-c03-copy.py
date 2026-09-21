"""Scoped C03 presentation copy; preserve every unrelated locale and engine value."""
import json
from pathlib import Path
root=Path(__file__).resolve().parents[2]; review=root/'review/art-satire'; web=root/'dystrail-web'
pack={u['id']:u for u in json.loads((review/'source-refresh-20260914/complete-pack.json').read_text())['units']}
translations=json.loads((review/'road-c03-translations.json').read_text())
ids=['ENC-C03-'+v for v in 'ABC']
keys=['name','desc','choice_0','log_0','choice_1','log_1','choice_2','log_2','overlay_0','fact']
facts=[
 'The March 2025 order directed steps toward closing the Education Department to the extent permitted by law. It did not itself abolish the department.',
 'The July 2025 White House announcement said Trump and donors would fund the ballroom structure and cited tents used for large functions. Its original estimate was not a final cost; security work was assigned separately to the Secret Service.',
 'In July 2025 the Eighth Circuit vacated the FTC click-to-cancel rule over regulatory procedure. This did not declare every difficult cancellation practice lawful; other consumer protections can still apply.'
]
lines={
 'en':[['POWER RETURNED','TO PARENTS'],['NAME THE','NEW WING'],['PLEASE CONTINUE','HOLDING']],
 'es':[['PODER DEVUELTO','A LAS FAMILIAS'],['PON NOMBRE A','LA NUEVA ALA'],['POR FAVOR, SIGA','EN ESPERA']],
 'it':[['POTERE RESTITUITO','AI GENITORI'],['DAI UN NOME','ALLA NUOVA ALA'],['RESTARE','IN ATTESA']],
 'ar':[['أُعيدت السلطة','إلى الأهالي'],['سمّوا الجناح','الجديد'],['يرجى البقاء','على الانتظار']]
}
for path in (web/'i18n').glob('*.json'):
 d=json.loads(path.read_text()); original=json.loads(path.read_text())
 for i,id in enumerate(ids):
  u=pack[id]; values=[u['title'],u['setup']]
  for c in u['choices']: values += [c['label'],c['outcome']]
  values += [u['scene']['overlays'][0]['text'],facts[i]]
  if id=='ENC-C03-B':values[1]=values[1].replace('under a leaking marquee','under a marquee still dripping from its last leak')
  if id=='ENC-C03-C':
   values[1]=values[1].replace('his gym','their gym').replace('He invites','They invite')
   values[3]=values[3].replace('him on','them on').replace('He asks','They ask')
   values[4]=values[4].replace('his cancellation','their cancellation')
   values[5]=values[5].replace('his permission','their permission').replace('record him selecting','record them selecting').replace('congratulates him','congratulates them')
  if path.stem in translations:values=translations[path.stem][id]
  assert len(values)==len(keys),(path,id)
  d['visual_copy'][id]=dict(zip(keys,values))
  d['visual_copy'][id].update(dict(zip(['overlay_line1','overlay_line2'],lines.get(path.stem,lines['en'])[i])))
  if i==0:d['visual_copy'][id]['overlay_1']={'es':'CONTROL LOCAL','it':'CONTROLLO LOCALE','ar':'التحكم المحلي'}.get(path.stem,'LOCAL CONTROL')
 check=json.loads(json.dumps(d))
 for id in ids:
  if id in original['visual_copy']:check['visual_copy'][id]=original['visual_copy'][id]
  else:del check['visual_copy'][id]
 assert check==original,path
 path.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n')
p=web/'static/assets/data/visual-sources.json'; sources=json.loads(p.read_text())
for i,id in enumerate(ids):
 u=pack[id];s=u['sources'][0]
 sources[id]={'title':s.get('title',u['source_hook']),'source':s['url'],'date':s.get('date','2025-03-20' if i==0 else ''),'checked':'2026-09-14','fact':facts[i],'qualification':s.get('qualification',s.get('claim','')),'translations':{lang:translations[lang][id][-1] for lang in ['es','it','ar']}}
p.write_text(json.dumps(sources,ensure_ascii=False,indent=2)+'\n')
print('C03 A/B/C copy integrated in en/es/it/ar; explicit English fallback elsewhere. Other locale values preserved.')
