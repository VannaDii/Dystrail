"""Bind the latest rest copy to the existing rest action, never its mechanics."""
import hashlib, json
from pathlib import Path
root=Path(__file__).resolve().parents[2]
web=root/'dystrail-web';review=root/'review/art-satire'
pack=json.loads((root/'review/satire-fresh-2026-09-13/complete-pack.json').read_text())
units=[u for u in pack['units'] if u.get('family_id')=='ACT-REST']
assert len(units)==3
translations=json.loads((review/'rest-translations.json').read_text())
for path in (web/'i18n').glob('*.json'):
    data=json.loads(path.read_text())
    for unit in units:
        values=[unit[k] for k in ['title','setup','action','outcome']]
        if unit['id']=='ACT-REST-B':
            values[3]='Rest does what it can. You close the curtains and let Washington argue with the daylight outside.'
        if path.stem in translations: values=translations[path.stem][unit['id']]
        data.setdefault('visual_copy',{})[unit['id']]=dict(zip(['title','setup','action','outcome'],values))
    path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')
path=web/'static/assets/data/visual-sources.json'
sources=json.loads(path.read_text())
facts={
 'A': ['Illinois permits covered workers to earn paid leave for any reason. Coverage exclusions, accrual and notice rules apply; this is not an immediate grant to every worker.',
 'Illinois permite que las personas cubiertas acumulen permiso remunerado por cualquier motivo. Existen exclusiones y reglas de acumulación y aviso; no es una concesión inmediata a todo el mundo.',
 'L’Illinois consente a chi rientra nei requisiti di maturare permessi retribuiti per qualsiasi motivo. Si applicano esclusioni e regole di maturazione e preavviso; non è un diritto immediato per tutti.',
 'تسمح إلينوي لمن يشملهم القانون باكتساب إجازة مدفوعة لأي سبب. تنطبق استثناءات وقواعد للاكتساب والإشعار؛ وليست منحة فورية لكل العاملين.'],
 'B': ['Trump opposed daylight saving in December 2024, then called for more evening daylight in April 2025. The statements were proposals, not enacted changes to federal clock rules.',
 'Trump se opuso al horario de verano en diciembre de 2024 y pidió más luz vespertina en abril de 2025. Eran propuestas, no cambios aprobados en las normas federales del reloj.',
 'Trump si oppose all’ora legale nel dicembre 2024, poi chiese più luce serale nell’aprile 2025. Erano proposte, non modifiche approvate alle regole federali sugli orari.',
 'عارض ترامب التوقيت الصيفي في ديسمبر 2024، ثم دعا إلى مزيد من ضوء النهار مساءً في أبريل 2025. كانت التصريحات مقترحات، لا تغييرات مُقَرّة في القواعد الفيدرالية للتوقيت.'],
 'C': ['In March 2025, senior Trump officials discussed Yemen strike details in a Signal chat that inadvertently included journalist Jeffrey Goldberg.',
 'En marzo de 2025, altos cargos de Trump comentaron detalles de ataques en Yemen en un chat de Signal que incluía por error al periodista Jeffrey Goldberg.',
 'Nel marzo 2025, alti funzionari di Trump discussero dettagli di attacchi nello Yemen in una chat Signal che includeva per errore il giornalista Jeffrey Goldberg.',
 'في مارس 2025، ناقش مسؤولون كبار في إدارة ترامب تفاصيل ضربات في اليمن داخل محادثة على سيغنال ضمّت الصحفي جيفري غولدبرغ بالخطأ.']}
for unit in units:
    v=unit['variant'];s=unit['sources'][0]
    sources[unit['id']]=dict(title=s['title'],date=s.get('date','2025-03-26'),fact=facts[v][0],source=s['url'],checked='2026-09-14',qualification='Original fictional travelers and dialogue; the source documents the political hook.',translations=dict(zip(['es','it','ar'],facts[v][1:])))
# Workshop AP addresses failed direct open; use the same reporting at working publication URLs.
sources['ACT-REST-B']['source']='https://www.ksat.com/news/politics/2025/04/11/trump-wants-congress-to-end-the-changing-of-clocks-and-keep-the-country-on-daylight-saving-time/'
sources['ACT-REST-C']['source']='https://www.ap.org/news-highlights/spotlights/2025/the-atlantic-releases-the-signal-chat-showing-hegseths-detailed-attack-plans-against-the-houthis/'
path.write_text(json.dumps(sources,ensure_ascii=False,indent=2)+'\n')
catalog_path=review/'asset-catalog.json';catalog=json.loads(catalog_path.read_text())
asset='dystrail-web/static/img/scenes-v2/rest-props.png'
entry=dict(id='rest-props',path=asset,source_image='exec-af66aca9-4082-4c51-8932-e835f081e1ed.png',prompt='review/art-satire/prompts/rest-props-latest.txt',sha256=hashlib.sha256((root/asset).read_bytes()).hexdigest(),method='built-in image generation',background='RGBA',status='integrated_pending_render_review',cells={f'ACT-REST-{v}':dict(offered=i*2,completed=i*2+1) for i,v in enumerate('ABC')},source_revision='source-refresh-20260914',review_note='Transparent props over the actual road. Current crew outside before A/C, seated before B, obscured by curtains during completed rest. No food drawn when supplies may be zero. B outcome qualifies recovery because daily settlement can offset gains.')
catalog['assets']=[a for a in catalog['assets'] if a['id']!='rest-props']+[entry]
clock='dystrail-web/static/img/scenes-v2/rest-clock-faces.png'
clock_entry=dict(id='rest-clock-faces',path=clock,source_image='exec-ce5c957e-6708-4d7e-8932-2289c3be7018.png',prompt='review/art-satire/prompts/rest-clock-faces.txt',sha256=hashlib.sha256((root/clock).read_bytes()).hexdigest(),method='built-in image edit',background='RGB; rejected as a transparent sheet',status='integrated_pending_render_review',review_note='Only the two generated blank cream dials are used, through ellipse clip paths. All surrounding pixels come from the separate RGBA prop sheet. Clock hands are native SVG tied to actual game hour and minute. No raster modification.')
catalog['assets']=[a for a in catalog['assets'] if a['id']!='rest-clock-faces']+[clock_entry]
for u in catalog['units']:
    if u['family']=='ACT-REST': u.update(asset='rest-props',production_status='integrated_pending_render_review')
catalog_path.write_text(json.dumps(catalog,ensure_ascii=False,indent=2)+'\n')
print('Integrated three latest rest variants and source context in EN/ES/IT/AR, English fallback elsewhere.')
