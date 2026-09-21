"""Build presentation copy/source/art bindings without altering activity mechanics."""
import copy
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parents[2]
web = root / 'dystrail-web'
review = root / 'review/art-satire'
families = ['ACT-FORAGE', 'ACT-GLEAN', 'ACT-FOODWORK', 'ACT-CASHWORK']
pack = json.loads((root / 'review/satire-fresh-2026-09-13/complete-pack.json').read_text())
units = [u for u in pack['units'] if u['family_id'] in families]
assert len(units) == 12
translations = json.loads((review / 'activity-bc-translations.json').read_text())
english = {u['id']: {k:u[k] for k in ['title','setup','outcome']} for u in units}
english['ACT-CASHWORK-A']['setup'] = english['ACT-CASHWORK-A']['setup'].replace('He points','They point')
english['ACT-GLEAN-A']['setup'] = english['ACT-GLEAN-A']['setup'].replace('His crops','Their crops')
english['ACT-CASHWORK-B']['setup'] = 'A venue offers $18 to unload staging for a billionaire’s political fundraiser. She says she built everything herself. The stage needs hired help to stand up.'
gender = {
 'es': {
 'ACT-CASHWORK-A': {'setup':'Un restaurante ofrece $18 por una limpieza de tres horas. La dueña elogia la promesa de Trump de «no gravar las propinas». Preguntas por el salario. Ella señala el bote de propinas.'},
 'ACT-GLEAN-A': {'setup':'Una persona que cultiva la tierra deja al grupo recoger las verduras restantes mientras se queja de los aranceles sobre las piezas del tractor. Los cultivos son locales. Sus gastos de funcionamiento han dado la vuelta al mundo.','outcome':'Trabajáis en la hilera y recogéis comida, doloridos por el esfuerzo. Os muestran una factura de reparación. «China ha vuelto a saltarse un pago».'}},
 'it': {
 'ACT-CASHWORK-A': {'setup':'Un ristorante offre $18 per un turno di pulizia di tre ore. La proprietaria elogia la promessa di Trump di «nessuna tassa sulle mance». Chiedete della paga. Lei indica il barattolo delle mance.'},
 'ACT-GLEAN-A': {'setup':'Chi coltiva questo campo lascia al gruppo gli ortaggi rimasti e si lamenta dei dazi sui ricambi del trattore. I raccolti sono locali. Le spese di gestione hanno fatto il giro del mondo.','outcome':'Lavorate lungo il filare e raccogliete cibo, con i muscoli doloranti. Vi viene mostrata una fattura di riparazione. «La Cina ha saltato un altro pagamento».'}},
 'ar': {
 'ACT-CASHWORK-A': {'setup':'يعرض مطعم $18 مقابل تنظيف يستغرق ثلاث ساعات. تمدح صاحبة المطعم وعد ترامب بـ«لا ضرائب على الإكراميات». تسألون عن الأجر. تشير إلى وعاء الإكراميات.'},
 'ACT-GLEAN-A': {'setup':'يسمح لكم أهل المزرعة بجمع ما تبقى من المحصول، وهم يتذمرون من الرسوم الجمركية على قطع الجرار. نمت المحاصيل محليًا. أما نفقات تشغيلها فقد طافت العالم.','outcome':'تعملون في الصف وتجمعون طعامًا، وعضلاتكم تؤلمكم من الجهد. يلوح أهل المزرعة بفاتورة إصلاح. «تخلّفت الصين عن دفعة أخرى».'}}
}
for path in (web/'i18n').glob('*.json'):
    data = json.loads(path.read_text())
    lang = path.stem
    for unit, text in english.items():
        if lang in translations:
            family = unit.rsplit('-',1)[0]
            text = ({k:data['workshop'][family][k] for k in ['title','setup','outcome']}
                    if unit.endswith('-A') else dict(zip(['title','setup','outcome'],translations[lang][unit])))
            text.update(gender.get(lang,{}).get(unit,{}))
        data.setdefault('visual_copy',{})[unit] = text
    path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')

original = json.loads((web/'static/assets/data/satire-sources.json').read_text())
path = web/'static/assets/data/visual-sources.json'
sources = json.loads(path.read_text())
by_url = {v['source']:v for v in original.values()}
for unit in units:
    sources[unit['id']] = copy.deepcopy(by_url[unit['political_basis']['source_url']])
food = copy.deepcopy(original['ACT-FOODWORK'])
food.update(source=sources['ORDER-SHUTDOWN-C']['source'],checked='2026-09-14')
for unit in ['ACT-FOODWORK-A','ACT-GLEAN-B']:
    sources[unit] = food
path.write_text(json.dumps(sources,ensure_ascii=False,indent=2)+'\n')

catalog_path = review/'asset-catalog.json'
catalog = json.loads(catalog_path.read_text())
image_ids = ['3655f57c-2a9e-48db-9bb1-5edf13681559','dc0da6db-a9b5-4235-9c9e-7fbcf3e0ce4c','ab9b4565-d9e9-4a51-ac25-685a454d0db9']
bounds = [[0,242,489,721,1024],[0,256,510,733,1024],[0,256,512,737,1024]]
for variant,image_id,ys in zip('ABC',image_ids,bounds):
    asset = 'activities-'+variant.lower()
    path = 'dystrail-web/static/img/scenes-v2/'+asset+'.png'
    entry = dict(id=asset,path=path,source_image='exec-'+image_id+'.png',
        prompt='review/art-satire/prompts/'+asset+'.txt',sha256=hashlib.sha256((root/path).read_bytes()).hexdigest(),
        method='built-in image generation',background='opaque',status='integrated_pending_render_review',
        cells={f+'-'+variant:{'offered':i*2,'completed':i*2+1} for i,f in enumerate(families)},
        row_boundaries=ys,column_boundaries=[0,768,1536],source_inset=4,
        review_note='Distinct offered and completed frames; feminine or neutral local NPCs, no player crew. Explicit frame bounds preserve uneven authored rows. No baked lettering.')
    catalog['assets'] = [a for a in catalog['assets'] if a['id'] != asset] + [entry]
for unit in catalog['units']:
    if unit['family'] in families:
        unit.update(asset='activities-'+unit['variant'].lower(),production_status='integrated_pending_render_review')
catalog_path.write_text(json.dumps(catalog,ensure_ascii=False,indent=2)+'\n')
print('Integrated 12 activity variants, 24 scene frames, localized copy and selected sources.')
