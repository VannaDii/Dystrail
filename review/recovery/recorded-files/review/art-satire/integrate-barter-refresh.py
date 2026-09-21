"""Apply only three revised barter records; preserve all other active copy/art."""
import hashlib,json
from pathlib import Path
root=Path(__file__).resolve().parents[2];review=root/'review/art-satire';web=root/'dystrail-web'
ids=['ACT-BARTERTIRE-B','ACT-BARTERTIRE-C','ACT-BARTERSUPPLIES-B']
units={u['id']:u for u in json.loads((review/'source-refresh-20260914/complete-pack.json').read_text())['units']}
translations=json.loads((review/'barter-refresh-translations.json').read_text())
for path in (web/'i18n').glob('*.json'):
    data=json.loads(path.read_text());before=json.loads(path.read_text())
    for id in ids:
        text=[units[id][k] for k in ['title','setup','action','outcome']]
        if path.stem in translations:text=translations[path.stem][id]
        data['visual_copy'][id]=dict(zip(['title','setup','action','outcome'],text))
    unchanged=json.loads(json.dumps(data))
    for id in ids:unchanged['visual_copy'][id]=before['visual_copy'][id]
    assert unchanged==before,path
    path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')
facts=[
 ['The March 2025 order created a Strategic Bitcoin Reserve, with statutory exceptions and budget-neutral rules for further acquisitions.',
  'La orden de marzo de 2025 creó una Reserva Estratégica de Bitcoines, con excepciones legales y reglas de neutralidad presupuestaria para nuevas adquisiciones.',
  'L’ordine del marzo 2025 ha creato una riserva strategica di bitcoin, con eccezioni di legge e vincoli di neutralità di bilancio per ulteriori acquisizioni.',
  'أنشأ أمر مارس 2025 احتياطيًا استراتيجيًا من البيتكوين، مع استثناءات قانونية واشتراط الحياد المالي للاستحواذات الإضافية.'],
 ['The qualified-tip deduction has eligibility and income limits. Reported tips generally remain subject to Social Security and Medicare taxes.',
  'La deducción por propinas cualificadas tiene requisitos y límites de renta. Las propinas declaradas siguen generalmente sujetas a las cotizaciones de Social Security y Medicare.',
  'La deduzione sulle mance qualificate ha requisiti e limiti di reddito. Le mance dichiarate restano generalmente soggette ai contributi Social Security e Medicare.',
  'لخصم الإكراميات المؤهلة شروط وحدود للدخل. وتظل الإكراميات المبلّغ عنها عمومًا خاضعة لضرائب الضمان الاجتماعي وميديكير.'],
 ['Oklahoma removed the state sales tax on qualifying groceries in August 2024. Local taxes and excluded categories remain.',
  'Oklahoma eliminó el impuesto estatal sobre los alimentos que cumplen los requisitos en agosto de 2024. Se mantienen los impuestos locales y las categorías excluidas.',
  'L’Oklahoma ha eliminato l’imposta statale sugli alimentari ammessi nell’agosto 2024. Restano le imposte locali e le categorie escluse.',
  'ألغت أوكلاهوما ضريبة المبيعات على مستوى الولاية للبقالة المؤهلة في أغسطس 2024. وبقيت الضرائب المحلية والفئات المستثناة.']]
p=web/'static/assets/data/visual-sources.json';data=json.loads(p.read_text())
for id,text in zip(ids,facts):
    s=units[id]['sources'][0]
    data[id]={'title':s['title'],'date':s.get('date','undated'),'fact':text[0],'source':s['url'],'checked':'2026-09-14','qualification':s.get('qualification',''),'translations':dict(zip(['es','it','ar'],text[1:]))}
data[ids[1]]['source']='https://www.irs.gov/publications/p15'
data[ids[1]]['title']='IRS: Publication 15 (2026), Employer’s Tax Guide'
p.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')
p=review/'asset-catalog.json';catalog=json.loads(p.read_text());asset='barter-refresh-20260914';assetpath=f'dystrail-web/static/img/scenes-v2/{asset}.png'
entry=dict(id=asset,path=assetpath,source_image='exec-18183f97-b069-45a4-ba09-db8d5cdf6b6b.png',initial_image='exec-01ae6e81-6d36-4c3f-92aa-aef5a37f22fc.png',prompt=f'review/art-satire/prompts/{asset}.txt',sha256=hashlib.sha256((root/assetpath).read_bytes()).hexdigest(),method='built-in image generation and edit',background='opaque',status='integrated_pending_render_review',cells={id:dict(offered=i*2,completed=i*2+1) for i,id in enumerate(ids)},row_boundaries=[0,341,680,1024],column_boundaries=[0,768,1536],source_inset=4,source_revision='source-refresh-20260914',review_note='Three revised source units only. Original generated bags were open; built-in edit closed them. Blank pay slip/receipt remain with residents, one intact tire and closed provisions bag swap positions. No player or repair baked into scene. Feminine/neutral residents; server retains a headscarf from established style reference.')
catalog['assets']=[a for a in catalog['assets'] if a['id']!=asset]+[entry]
for u in catalog['units']:
    if u['id'] in ids:u.update(asset=asset,production_status='integrated_pending_render_review')
p.write_text(json.dumps(catalog,ensure_ascii=False,indent=2)+'\n')
print('Updated three barter records; unrelated locale subtrees preserved exactly.')
