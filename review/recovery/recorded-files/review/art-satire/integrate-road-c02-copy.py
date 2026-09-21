"""Integrate only reviewed C02 prose and source context; engine choices stay intact."""
import json
from pathlib import Path
root=Path(__file__).resolve().parents[2];review=root/'review/art-satire';web=root/'dystrail-web'
pack={u['id']:u for u in json.loads((review/'source-refresh-20260914/complete-pack.json').read_text())['units']}
translations=json.loads((review/'road-c02-translations.json').read_text())
ids=['ENC-C02-'+v for v in 'ABC']
keys=['name','desc','choice_0','log_0','choice_1','log_1','choice_2','log_2','overlay_0','fact']
facts=[
 'The 2025 reconciliation law includes SNAP spending reductions and eligibility changes alongside tax provisions. It does not establish that every food pantry lost funding.',
 'The 2025 reconciliation law expands SNAP requirements and creates Medicaid community-engagement requirements. The federal Medicaid rollout begins in January 2027, with an earlier state option; it is not a nationwide 2026 implementation.',
 'Citizens United and SpeechNow underpin unlimited independent spending and contributions to independent-expenditure-only committees. Direct candidate contribution limits are a different rule.'
]
lines={
 'en':[['TAX','CUTS'],['PROOF OF','WORK'],['UNLIMITED','SPEECH']],
 'es':[['RECORTES DE','IMPUESTOS'],['PRUEBA DE','EMPLEO'],['DISCURSO','ILIMITADO']],
 'it':[['TAGLI ALLE','TASSE'],['PROVA','D’IMPIEGO'],['DISCORSO','ILLIMITATO']],
 'ar':[['خفض','الضرائب'],['إثبات','العمل'],['خطاب بلا','حدود']]
}
for path in (web/'i18n').glob('*.json'):
 d=json.loads(path.read_text());original=json.loads(path.read_text())
 for i,id in enumerate(ids):
  u=pack[id];values=[u['title'],u['setup']]
  for choice in u['choices']:values += [choice['label'],choice['outcome']]
  values += [u['scene']['overlays'][0]['text'],facts[i]]
  if path.stem in translations:values=translations[path.stem][id]
  assert len(values)==len(keys)
  d['visual_copy'][id]=dict(zip(keys,values))
  d['visual_copy'][id].update(dict(zip(['overlay_line1','overlay_line2'],lines.get(path.stem,lines['en'])[i])))
 check=json.loads(json.dumps(d))
 for id in ids:
  if id in original['visual_copy']:check['visual_copy'][id]=original['visual_copy'][id]
  else:del check['visual_copy'][id]
 assert check==original,path
 path.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n')
p=web/'static/assets/data/visual-sources.json';sources=json.loads(p.read_text())
for i,id in enumerate(ids):
 u=pack[id];s=u['sources'][0]
 sources[id]={'title':s.get('title',u['source_hook']),'source':s['url'],'date':s.get('date',''),'checked':'2026-09-14','fact':facts[i],'qualification':s.get('qualification',''),'translations':{lang:translations[lang][id][-1] for lang in ['es','it','ar']}}
p.write_text(json.dumps(sources,ensure_ascii=False,indent=2)+'\n')
print('Three C02 units, four narrative languages and explicit English fallback integrated; all other locale values unchanged.')
