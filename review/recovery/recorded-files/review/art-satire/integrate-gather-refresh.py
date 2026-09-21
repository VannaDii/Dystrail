"""Apply the revised gathering scenes and editable crate stamp."""
import hashlib,json,shutil
from pathlib import Path
root=Path(__file__).resolve().parents[2];r=root/'review/art-satire';w=root/'dystrail-web'
ids=['ACT-FORAGE-C','ACT-GLEAN-A','ACT-GLEAN-B','ACT-GLEAN-C']
pack={u['id']:u for u in json.loads((r/'source-refresh-20260914/complete-pack.json').read_text())['units']};translations=json.loads((r/'gather-refresh-translations.json').read_text())
labels={'en':['SELL BY','LAST YEAR'],'es':['VENDER ANTES DE','EL AÑO PASADO'],'it':['VENDITA ENTRO','L’ANNO SCORSO'],'ar':['يُباع قبل','العام الماضي']}
for path in (w/'i18n').glob('*.json'):
    data=json.loads(path.read_text());original=json.loads(path.read_text())
    for id in ids:
        values=[pack[id][key] for key in ['title','setup','action','outcome']]
        # User cast brief covers fictional supporting characters as well as crew.
        if id=='ACT-GLEAN-A':values[1]=values[1].replace('His tractor','Her tractor')
        if id=='ACT-GLEAN-C':values[1]=values[1].replace('He studies','She studies')
        if path.stem in translations:values=translations[path.stem][id]
        data['visual_copy'][id]=dict(zip(['title','setup','action','outcome'],values))
    data.setdefault('visual_labels',{})['glean_crate']=labels.get(path.stem,labels['en'])
    check=json.loads(json.dumps(data))
    for id in ids:check['visual_copy'][id]=original['visual_copy'][id]
    if 'visual_labels' in original:check['visual_labels']=original['visual_labels']
    else:del check['visual_labels']
    assert check==original,path
    path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')
facts=[
 ['NIH public access rules remove the embargo for covered author-accepted manuscripts from July 2025. They do not make every journal or historical paper freely available.',
 'Las normas de acceso público de los NIH eliminan el embargo para los manuscritos aceptados cubiertos desde julio de 2025. No dan acceso gratuito a todas las revistas o publicaciones antiguas.',
 'Le regole NIH eliminano l’embargo per i manoscritti accettati coperti dalla politica dal luglio 2025. Non rendono gratuiti tutti i periodici o gli articoli storici.',
 'تلغي قواعد المعاهد الوطنية للصحة الحظر المؤقت على المخطوطات المقبولة المشمولة منذ يوليو 2025. ولا تتيح كل مجلة أو بحث قديم مجانًا.'],
 ['The FTC announced a proposed Deere repair-access settlement in July 2026. Judicial approval was required; the scene does not claim the promised tools are already available.',
 'La FTC anunció una propuesta de acuerdo con Deere sobre acceso a reparaciones en julio de 2026. Requería aprobación judicial; la escena no afirma que las herramientas ya estén disponibles.',
 'La FTC annunciò nel luglio 2026 una proposta di accordo con Deere sull’accesso alle riparazioni. Serviva l’approvazione giudiziaria; la scena non afferma che gli strumenti siano già disponibili.',
 'أعلنت لجنة التجارة الفيدرالية في يوليو 2026 تسوية مقترحة بشأن إتاحة أدوات إصلاح دير. كانت تتطلب موافقة قضائية؛ ولا يدّعي المشهد أن الأدوات الموعودة متاحة بالفعل.'],
 ['California’s July 2026 rules standardize quality and safety date labels on covered newly manufactured foods. They do not require every food to carry a date, and exemptions apply.',
 'Las normas californianas de julio de 2026 uniforman las fechas de calidad y seguridad en los nuevos alimentos cubiertos. No exigen fechar todos los alimentos y existen exenciones.',
 'Le regole californiane del luglio 2026 uniformano le date di qualità e sicurezza per i nuovi alimenti coperti. Non impongono una data su ogni alimento e prevedono esenzioni.',
 'توحّد قواعد كاليفورنيا من يوليو 2026 بطاقات تاريخ الجودة والسلامة للأغذية الجديدة المشمولة. لا تلزم كل غذاء بحمل تاريخ، وتوجد استثناءات.'],
 ['Maryland authorized stadium bonds for Hagerstown and identified public budget contributions. The corn comparison is fictional and makes no claim of funds diverted from a named service.',
 'Maryland autorizó bonos para el estadio de Hagerstown e identificó aportaciones presupuestarias públicas. La comparación con el maíz es ficticia y no afirma un desvío de fondos de un servicio concreto.',
 'Il Maryland autorizzò obbligazioni per lo stadio di Hagerstown e indicò contributi pubblici. Il paragone con il mais è inventato e non afferma che fondi siano stati sottratti a un servizio specifico.',
 'أجازت ماريلاند سندات لملعب هاغرستاون وحددت مساهمات من الموازنة العامة. مقارنة الذرة خيالية ولا تدّعي تحويل أموال من خدمة محددة.']]
p=w/'static/assets/data/visual-sources.json';data=json.loads(p.read_text())
for id,text in zip(ids,facts):
    s=pack[id]['sources'][0]
    data[id]={'title':s['title'],'source':s['url'],'date':s.get('date','undated'),'checked':'2026-09-14','fact':text[0],'qualification':s.get('qualification',''),'translations':dict(zip(['es','it','ar'],text[1:]))}
data['ACT-GLEAN-C']['date']='2022-10-19'
p.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')
asset='gather-refresh-20260914';ap=f'dystrail-web/static/img/scenes-v2/{asset}.png';shutil.copy2(r/'gather-refresh-candidate.png',root/ap)
p=r/'asset-catalog.json';catalog=json.loads(p.read_text())
entry=dict(id=asset,path=ap,source_image='exec-97994eb1-95b4-4131-9815-a42bc038f0b8.png',initial_image='exec-09e62dff-b20e-44de-93eb-6ea895046c7e.png',prompt=f'review/art-satire/prompts/{asset}.txt',sha256=hashlib.sha256((root/ap).read_bytes()).hexdigest(),method='built-in image generation and edit',background='opaque',status='integrated_pending_render_review',source_revision='source-refresh-20260914',cells={id:dict(offered=i*2,completed=i*2+1) for i,id in enumerate(ids)},row_boundaries=[0,242,489,742,1024],column_boundaries=[0,768,1536],source_inset=4,review_note='Baskets fill only after gathering. Botanist keeps research printout; tractor remains stationary with closed toolbox; reused wooden crate remains empty; corn/baseball comparison persists. Crate label enlarged by built-in edit for localized native lettering. No fixed crew or resource counts painted into scene.')
catalog['assets']=[a for a in catalog['assets'] if a['id']!=asset]+[entry]
for u in catalog['units']:
    if u['id'] in ids:u.update(asset=asset,production_status='integrated_pending_render_review')
p.write_text(json.dumps(catalog,ensure_ascii=False,indent=2)+'\n')
print('Integrated four revised gathering units and separate localized label; unrelated values preserved.')
