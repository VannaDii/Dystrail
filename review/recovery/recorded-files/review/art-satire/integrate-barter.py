"""Bind reviewed barter variants to existing atomic community exchanges."""
import copy
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parents[2]
web = root/'dystrail-web'
review = root/'review/art-satire'
families = ['ACT-BARTERTIRE','ACT-BARTERBATTERY','ACT-BARTERSUPPLIES']
pack = json.loads((root/'review/satire-fresh-2026-09-13/complete-pack.json').read_text())
units = [u for u in pack['units'] if u.get('family_id') in families]
assert len(units) == 9
translations = json.loads((review/'barter-bc-translations.json').read_text())
# Feminine residents in A; preserve the premise and actual exchange in each language.
setups = {
 'es': [
 'En el punto de intercambio comunitario, una vecina ofrece una rueda a cambio de provisiones. Una radio cercana habla de gastos políticos ilimitados. Preguntas si la rueda viene con un senador.',
 'Una vecina ofrece una batería a cambio de provisiones. Trump promete la independencia energética estadounidense en la radio. Preguntas si eso incluye poner en marcha la furgoneta.',
 'Una vecina ofrece comida a cambio de vuestra rueda de repuesto. El Congreso debate quién merece ayuda. Ella principalmente quiere una rueda que mantenga el aire.'],
 'it': [
 'Al punto di scambio della comunità, una residente offre una ruota in cambio di provviste. Una radio vicina parla di spese politiche illimitate. Chiedete se la ruota include un senatore.',
 'Una residente offre una batteria in cambio di provviste. Trump promette l’indipendenza energetica americana alla radio. Chiedete se questo include mettere in moto il furgone.',
 'Una residente offre cibo in cambio della vostra ruota di scorta. Il Congresso discute di chi meriti assistenza. Lei desidera soprattutto una ruota che tenga l’aria.'],
 'ar': [
 'في ساحة التبادل المجتمعي، تعرض إحدى السكان إطارًا مقابل مؤن. تتحدث محطة إذاعية قريبة عن الإنفاق السياسي غير المحدود. تسألون إن كان الإطار يأتي مع عضو في مجلس الشيوخ.',
 'تعرض إحدى السكان بطارية مقابل مؤن. يعد ترامب باستقلال أمريكا في مجال الطاقة عبر الراديو. تسألون إن كان ذلك يشمل تشغيل الشاحنة.',
 'تعرض إحدى السكان طعامًا مقابل إطاركم الاحتياطي. يناقش الكونغرس من يستحق المساعدة. أما هي فتريد أساسًا إطارًا يحتفظ بالهواء.']
}
for path in (web/'i18n').glob('*.json'):
    data = json.loads(path.read_text())
    reasons = {
        'en':['Not enough supplies for this exchange.','You need a spare tire to trade.'],
        'es':['No hay suficientes provisiones para este intercambio.','Necesitáis una rueda de repuesto para intercambiar.'],
        'it':['Non ci sono abbastanza provviste per questo scambio.','Serve una ruota di scorta da scambiare.'],
        'ar':['المؤن لا تكفي لهذه المقايضة.','تحتاجون إلى إطار احتياطي للمقايضة.']
    }
    data['visual_trade'] = dict(zip(['need_supplies','need_tire'],reasons.get(path.stem,reasons['en'])))
    for unit in units:
        text = {k:unit[k] for k in ['title','setup','action','outcome']}
        if path.stem in translations:
            if unit['variant'] == 'A':
                text = copy.deepcopy(data['workshop'][unit['family_id']])
                text['setup'] = setups[path.stem][families.index(unit['family_id'])]
                text['action'] = text['action'].rstrip('.')
            else:
                text = dict(zip(['title','setup','action','outcome'],translations[path.stem][unit['id']]))
        data.setdefault('visual_copy',{})[unit['id']] = text
    path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')
path = web/'static/assets/data/visual-sources.json'
sources = json.loads(path.read_text())
original = json.loads((web/'static/assets/data/satire-sources.json').read_text())
by_url = {r['source']:r for r in original.values()}
for unit in units:
    sources[unit['id']] = copy.deepcopy(by_url[unit['political_basis']['source_url']])
# Use the same verified CRS food-assistance report as the pantry activity;
# the workshop's AP address was unavailable during the source review.
sources['ACT-BARTERSUPPLIES-A'] = copy.deepcopy(sources['ACT-FOODWORK-A'])
path.write_text(json.dumps(sources,ensure_ascii=False,indent=2)+'\n')
catalog_path = review/'asset-catalog.json'
catalog = json.loads(catalog_path.read_text())
ids = ['2ee893b9-c350-48b1-96f6-f853dfb41b9f','4e1f8743-51df-43dc-abad-95b885927611','b59ce8e8-c10c-4aac-aae2-7a0e501f6fa1']
bounds = [[0,340,683,1024],[0,341,680,1024],[0,341,684,1024]]
for variant,image_id,ys in zip('ABC',ids,bounds):
    asset = 'barter-'+variant.lower()
    path = 'dystrail-web/static/img/scenes-v2/'+asset+'.png'
    entry = dict(id=asset,path=path,source_image='exec-'+image_id+'.png',
        prompt='review/art-satire/prompts/'+asset+'.txt',sha256=hashlib.sha256((root/path).read_bytes()).hexdigest(),
        method='built-in image generation',background='opaque',status='integrated_pending_render_review',
        cells={f+'-'+variant:{'offered':i*2,'completed':i*2+1} for i,f in enumerate(families)},
        row_boundaries=ys,column_boundaries=[0,768,1536],source_inset=4,
        review_note='Goods swap from far to near side of the same table. No vehicle repair, player crew or lettering baked into the scene. Supporting residents are feminine or neutral.')
    catalog['assets'] = [a for a in catalog['assets'] if a['id'] != asset] + [entry]
for unit in catalog['units']:
    if unit['family'] in families:
        unit.update(asset='barter-'+unit['variant'].lower(),production_status='integrated_pending_render_review')
catalog_path.write_text(json.dumps(catalog,ensure_ascii=False,indent=2)+'\n')
print('Integrated nine barter variants, eighteen scene frames, copy and selected factual sources.')
