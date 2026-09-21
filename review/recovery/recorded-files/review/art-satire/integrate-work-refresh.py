"""Integrate four revised work scenes without replacing other copy or mechanics."""
import hashlib,json,shutil
from pathlib import Path
root=Path(__file__).resolve().parents[2];r=root/'review/art-satire';w=root/'dystrail-web'
ids=['ACT-FOODWORK-A','ACT-FOODWORK-B','ACT-FOODWORK-C','ACT-CASHWORK-B']
pack={u['id']:u for u in json.loads((r/'source-refresh-20260914/complete-pack.json').read_text())['units']};translations=json.loads((r/'work-refresh-translations.json').read_text())
for path in (w/'i18n').glob('*.json'):
    data=json.loads(path.read_text());original=json.loads(path.read_text())
    for id in ids:
        values=[pack[id][key] for key in ['title','setup','action','outcome']]
        if path.stem in translations:values=translations[path.stem][id]
        data['visual_copy'][id]=dict(zip(['title','setup','action','outcome'],values))
    check=json.loads(json.dumps(data))
    for id in ids:check['visual_copy'][id]=original['visual_copy'][id]
    assert check==original,path
    path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')
facts=[
 ['The January 2026 law permits whole and reduced-fat milk in the school lunch program. It adds options, without requiring every child to choose them.',
 'La ley de enero de 2026 permite leche entera y semidesnatada en el programa de almuerzos escolares. Añade opciones sin obligar a cada menor a elegirlas.',
 'La legge del gennaio 2026 consente latte intero e a ridotto contenuto di grassi nel programma di mensa scolastica. Aggiunge opzioni senza imporle a ogni bambino.',
 'يسمح قانون يناير 2026 بالحليب كامل الدسم وقليل الدسم في برنامج الغداء المدرسي. ويضيف خيارات من دون إلزام كل طفل باختيارها.'],
 ['Texas Farm Bureau opposed Trump’s October 2025 suggestion to increase Argentine beef imports. The dinner and dialogue are fictional.',
 'Texas Farm Bureau se opuso a la sugerencia de Trump de octubre de 2025 de aumentar las importaciones de carne argentina. La cena y el diálogo son ficticios.',
 'Texas Farm Bureau si oppose alla proposta di Trump dell’ottobre 2025 di aumentare le importazioni di carne argentina. La cena e il dialogo sono inventati.',
 'عارض اتحاد مزارعي تكساس اقتراح ترامب في أكتوبر 2025 زيادة استيراد اللحم الأرجنتيني. المأدبة والحوار خياليان.'],
 ['In April 2026 the FTC announced enforcement actions over allegedly false Made in USA claims. Proposed court orders required judicial approval; these fictional pans are not an identified product.',
 'En abril de 2026, la FTC anunció medidas por presuntas afirmaciones falsas de fabricación estadounidense. Las órdenes propuestas requerían aprobación judicial; estas sartenes ficticias no son un producto identificado.',
 'Nell’aprile 2026 la FTC annunciò azioni contro presunte false dichiarazioni Made in USA. Gli ordini proposti richiedevano approvazione giudiziaria; queste pentole inventate non sono un prodotto identificato.',
 'أعلنت لجنة التجارة الفيدرالية في أبريل 2026 إجراءات بشأن ادعاءات يُزعم زيفها عن الصنع في أمريكا. احتاجت الأوامر المقترحة إلى موافقة قضائية؛ وهذه الأواني الخيالية ليست منتجًا محددًا.'],
 ['The DOE inspector general reported a 2024 settlement for claimed labor hours not worked at Pantex, and related operational harm. The hall job is fictional.',
 'La inspección general del Departamento de Energía informó de un acuerdo de 2024 por horas facturadas y no trabajadas en Pantex, y daños operativos relacionados. El trabajo de la sala es ficticio.',
 'L’ispettore generale del Dipartimento dell’Energia ha riferito di un accordo del 2024 per ore fatturate ma non lavorate a Pantex e relativi danni operativi. Il lavoro nella sala è inventato.',
 'أفاد المفتش العام لوزارة الطاقة بتسوية في 2024 بشأن ساعات فُوترت ولم تُعمل في بانتكس، وأضرار تشغيلية مرتبطة بها. العمل في القاعة خيالي.']]
p=w/'static/assets/data/visual-sources.json';data=json.loads(p.read_text())
for id,text in zip(ids,facts):
    s=pack[id]['sources'][0]
    data[id]={'title':s['title'],'source':s['url'],'date':s.get('date','2025-10-20'),'checked':'2026-09-14','fact':text[0],'qualification':s.get('qualification',''),'translations':dict(zip(['es','it','ar'],text[1:]))}
p.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')
asset='work-refresh-20260914';ap=f'dystrail-web/static/img/scenes-v2/{asset}.png';shutil.copy2(r/'work-refresh-candidate.png',root/ap)
p=r/'asset-catalog.json';catalog=json.loads(p.read_text())
entry=dict(id=asset,path=ap,source_image='exec-df4e831a-3f24-43d4-bf48-b3c988e907ad.png',prompt=f'review/art-satire/prompts/{asset}.txt',sha256=hashlib.sha256((root/ap).read_bytes()).hexdigest(),method='built-in image generation',background='opaque',status='integrated_pending_render_review',source_revision='source-refresh-20260914',cells={id:dict(offered=i*2,completed=i*2+1) for i,id in enumerate(ids)},row_boundaries=[0,256,510,766,1024],column_boundaries=[0,768,1536],source_inset=4,review_note='All indoor: milk donations sorted; dirty cookware cleaned; blank patriotic paper sleeve discarded; hall chairs cleared. No fixed crew or cash painted into scene. Food reward appears only after work or moves from far to near table edge. Feminine/neutral residents.')
catalog['assets']=[a for a in catalog['assets'] if a['id']!=asset]+[entry]
for u in catalog['units']:
    if u['id'] in ids:u.update(asset=asset,production_status='integrated_pending_render_review')
p.write_text(json.dumps(catalog,ensure_ascii=False,indent=2)+'\n')
print('Integrated four revised work units; all unrelated locale values preserved.')
