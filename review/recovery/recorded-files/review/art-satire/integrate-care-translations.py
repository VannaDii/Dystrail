"""Apply reviewed care translations without replacing other locale subtrees."""
from pathlib import Path
import json, copy, re
root=Path(__file__).resolve().parents[2]
locales=root/'dystrail-web/i18n'
rows={lang:json.loads((root/f'review/art-satire/care-translations-{lang}.json').read_text()) for lang in ['es','it','ar']}
rows['es']['CARE-06-B'][1]=rows['es']['CARE-06-B'][1].replace('no basta para ser autoría','no basta para atribuirse la autoría')
rows['it']['CARE-03-A'][1]="Dopo un turno a termine, {name} passa la notte a dimostrare che quelle ore valgono per i requisiti lavorativi dei sussidi del Congresso. All'alba, l'unico lavoro non pagato è dimostrare di lavorare."
rows['ar']['CARE-04-C'][1]=rows['ar']['CARE-04-C'][1].replace('يمرض {name} بسبب','يصيب المرض {name} بسبب')
rows['ar']['CARE-07-A'][1]=rows['ar']['CARE-07-A'][1].replace('تحصيان خمس مهام.','تضم القائمة خمس مهام.')
for lang,data in rows.items():
 (root/f'review/art-satire/care-translations-{lang}.json').write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')
en=json.loads((locales/'en.json').read_text())
changes={
 'CARE-01-C':{'setup':('He recommends','The vendor recommends')},
 'CARE-02-B':{'setup':('he says','the boss says')},
 'CARE-02-C':{'setup':('his private cooler','their private cooler'),'continuing':('his argument','their argument')},
 'CARE-03-C':{'setup':("the owner’s nephew referred them. He’s twelve.","a relative of the owner referred them. That relative is twelve."),'continuing':('The nephew’s referral box','The family-referral box'),'helped':('The nephew is unavailable','The relative is unavailable')},
 'CARE-04-C':{'setup':('He has mastered','The cook has mastered')},
 'CARE-07-B':{'setup':('He copied','The boss copied')},
 'CARE-08-A':{'setup':('his memoir','their memoir')},
}
for unit,fields in changes.items():
 for key,(old,new) in fields.items():
  d=en['visual_copy'][unit]['outcomes'] if key=='helped' else en['visual_copy'][unit]
  assert old in d[key],(unit,key)
  d[key]=d[key].replace(old,new)
 en['visual_copy'][unit]['outcomes']['deferred']=en['visual_copy'][unit]['continuing']
common={
'es':{
 'critical':'La salud de {name} está en estado crítico. Continuar ahora sin cuidados causará su muerte. Si es tu personaje, la partida terminará.',
 'sheltered':'{name} sigue con vida y recibe cuidados en un refugio, fuera de la expedición.',
 'companion_lost':'{name} ha muerto después de que el grupo continuara sin prestar cuidados. Su asiento en la furgoneta está vacío.',
 'player_lost':'Has muerto tras continuar sin cuidados. Tu viaje termina aquí.'},
'it':{
 'critical':'Le condizioni di {name} sono critiche. Proseguire ora senza cure ne causerà la morte. Se è il tuo personaggio, il viaggio finirà.',
 'sheltered':'{name} resta in vita e riceve assistenza in un rifugio, fuori dalla spedizione.',
 'companion_lost':'{name} ha perso la vita dopo che il gruppo ha proseguito senza prestare cure. Il suo posto nel furgone è vuoto.',
 'player_lost':'Hai perso la vita dopo aver proseguito senza cure. Il tuo viaggio finisce qui.'},
'ar':{
 'critical':'حالة {name} حرجة. مواصلة السير الآن دون رعاية ستؤدي إلى الوفاة. إذا كانت هذه شخصيتك، فستنتهي الرحلة.',
 'sheltered':'تم إيواء {name} على قيد الحياة، وانتهت المشاركة في الرحلة.',
 'companion_lost':'انتهت حياة {name} بعد مواصلة الفريق السير دون تقديم رعاية. المقعد في الشاحنة فارغ الآن.',
 'player_lost':'انتهت حياتك بعد مواصلة السير دون رعاية. رحلتك تنتهي هنا.'}}
for path in locales.glob('*.json'):
 lang=path.stem;data=en if lang=='en' else json.loads(path.read_text());hearing=copy.deepcopy(data['hearing'])
 for n in range(1,9):
  for v in 'ABC':
   unit=f'CARE-{n:02}-{v}';oldsign=data['visual_copy'][unit]['sign'];entry=copy.deepcopy(en['visual_copy'][unit])
   if lang in rows:
    title,setup,continuing,helped=rows[lang][unit]
    entry.update(title=title,setup=setup,continuing=continuing,critical=common[lang]['critical'],sign=oldsign)
    entry['outcomes']={k:common[lang][k] for k in ['sheltered','companion_lost','player_lost']}
    entry['outcomes'].update(helped=helped,deferred=continuing)
   data['visual_copy'][unit]=entry
 assert data['hearing']==hearing
 path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')
# Check every rendered field, including lethal outcomes, without inferring gender from custom names.
for lang in ['en','es','it','ar']:
 data=json.loads((locales/f'{lang}.json').read_text())
 for n in range(1,9):
  for v in 'ABC':
   unit=f'CARE-{n:02}-{v}';entry=data['visual_copy'][unit];source=en['visual_copy'][unit]
   for field in ['title','setup','continuing','critical']:
    assert set(re.findall(r'\{[^}]+\}',entry[field]))==set(re.findall(r'\{[^}]+\}',source[field])),(lang,unit,field)
    if lang!='en':assert entry[field]!=source[field],(lang,unit,field)
   for outcome in source['outcomes']:
    assert set(re.findall(r'\{[^}]+\}',entry['outcomes'][outcome]))==set(re.findall(r'\{[^}]+\}',source['outcomes'][outcome])),(lang,unit,outcome)
    if lang!='en':assert entry['outcomes'][outcome]!=source['outcomes'][outcome],(lang,unit,outcome)
print('Integrated all 24 care units in en/es/it/ar; 16 other locales retain explicit English narrative fallback. Name placeholders and hearing subtrees preserved.')
