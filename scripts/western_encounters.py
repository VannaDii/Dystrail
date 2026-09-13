"""Regional fictional encounters grounded in the linked historical policy records."""
from pathlib import Path
import json
from western_town_facts import FACTS
from encounter_evidence import apply_evidence

LANGS = ('en', 'es', 'it', 'ar')
ROOT = Path('dystrail-web/static/assets/data')


def choice(label, effects, log):
    return (label, effects, log)


PASS = choice(
    ('Keep moving', 'Seguir adelante', 'Proseguire', 'واصل الطريق'),
    {},
    ('The crew leaves the offer behind.', 'El grupo deja atrás la oferta.', 'Il gruppo si lascia l’offerta alle spalle.', 'يترك الطاقم العرض خلفه.'))

EVENTS = [
('west_grant_translation', 'PacificCoast', 'west_grants', 'enc-media-workshop',
 ('Patriotic Thesaurus', 'Tesauro patriótico', 'Thesaurus patriottico', 'قاموس المرادفات الوطني'),
 ('A coastal grant writer offers paid work translating the same public project into language that will survive Trump’s grant review. The sidewalk is now a freedom corridor. The shade tree is awaiting clearance.',
  'Una redactora de subvenciones ofrece trabajo para traducir el mismo proyecto al lenguaje de la revisión de Trump. La acera es ahora un corredor de libertad. El árbol espera autorización.',
  'Una consulente offre lavoro per tradurre lo stesso progetto nel linguaggio ammesso dalla revisione di Trump. Il marciapiede è ora un corridoio di libertà. L’albero attende il nullaosta.',
  'تعرض كاتبة منح عملاً مدفوعاً لترجمة المشروع نفسه إلى لغة تنجو من مراجعة ترامب. أصبح الرصيف ممر حرية. وشجرة الظل تنتظر التصريح.'),
 [choice(('Take the translation shift', 'Aceptar el turno de traducción', 'Accettare il turno di traduzione', 'اقبل وردية الترجمة'), {'cash_cents':1200,'sanity':-1},
         ('You earn twelve dollars. The project is unchanged; its adjectives have passed inspection.', 'Ganas doce dólares. El proyecto no cambia; sus adjetivos han pasado la inspección.', 'Guadagni dodici dollari. Il progetto è identico; gli aggettivi hanno superato l’ispezione.', 'تكسب اثني عشر دولاراً. المشروع لم يتغير؛ صفاته اجتازت التفتيش.')),
  choice(('Publish both versions', 'Publicar ambas versiones', 'Pubblicare entrambe le versioni', 'انشر النسختين'), {'credibility':2,'sanity':-1},
         ('You preserve the original and the patriotic translation. The receipts need no translation.', 'Conservas el original y la traducción patriótica. Las pruebas no necesitan traducción.', 'Conservi l’originale e la traduzione patriottica. Le prove non hanno bisogno di traduzione.', 'تحفظ الأصل والترجمة الوطنية. الأدلة لا تحتاج إلى ترجمة.')), PASS]),
('west_rail_replacement', 'PacificCoast', 'west_rail', 'enc-street',
 ('High-Speed Replacement Van', 'Furgoneta de alta velocidad', 'Furgone ad alta velocità', 'شاحنة السرعة الفائقة البديلة'),
 ('After another Trump rail-funding fight, a contractor proposes replacing high-speed rail with vans and motivational language. Their ribbon-cutting team has transportation needs of its own.',
  'Tras otra disputa por los fondos ferroviarios de Trump, un contratista propone sustituir el tren rápido por furgonetas y lenguaje motivador. El equipo de inauguración también necesita transporte.',
  'Dopo un’altra disputa sui fondi ferroviari di Trump, un appaltatore propone furgoni e frasi motivazionali al posto dell’alta velocità. Anche la squadra del taglio del nastro deve spostarsi.',
  'بعد نزاع آخر على تمويل القطارات في عهد ترامب، يقترح مقاول استبدال القطار السريع بشاحنات وعبارات تحفيزية. وفريق قص الشريط نفسه يحتاج إلى نقل.'),
 [choice(('Deliver their event supplies', 'Llevar sus materiales', 'Consegnare il materiale', 'انقل لوازم الحفل'), {'cash_cents':1500,'supplies':-1},
         ('You collect fifteen dollars. Your van is now the most operational part of the announcement.', 'Cobras quince dólares. Tu furgoneta es lo más operativo de todo el anuncio.', 'Incassi quindici dollari. Il tuo furgone è la parte più funzionante dell’annuncio.', 'تحصل على خمسة عشر دولاراً. شاحنتك أصبحت أكثر أجزاء الإعلان قابلية للعمل.')),
  choice(('Document the missing rail money', 'Documentar los fondos retirados', 'Documentare i fondi revocati', 'وثق تمويل القطار المسحوب'), {'credibility':2,'allies':1,'sanity':-1},
         ('A commuter joins your contacts. The brochure’s dotted line is still not a railway.', 'Un pasajero se suma a tus contactos. La línea de puntos del folleto sigue sin ser una vía.', 'Un pendolare entra tra i tuoi contatti. La linea tratteggiata del dépliant non è ancora una ferrovia.', 'ينضم راكب إلى معارفك. الخط المنقط في الكتيب لا يزال غير صالح كمسار قطار.')), PASS]),
('west_laboratory_overhead', 'MountainWest', 'west_labs', 'enc-clinic',
 ('A Building Is Overhead', 'Un edificio es un gasto indirecto', 'Un edificio è un costo indiretto', 'المبنى نفقة غير مباشرة'),
 ('A mountain-region laboratory is preparing for Trump’s proposed NIH overhead cap. The budget funds the experiment but objects to walls, power and refrigeration. A researcher offers you a practical choice.',
  'Un laboratorio de montaña se prepara para el tope de gastos indirectos del NIH propuesto por Trump. El presupuesto paga el experimento, pero cuestiona paredes, electricidad y refrigeración.',
  'Un laboratorio tra le montagne si prepara al tetto NIH proposto da Trump. Il bilancio finanzia l’esperimento ma contesta pareti, corrente e frigoriferi. Una ricercatrice propone una scelta concreta.',
  'يستعد مختبر في الغرب الجبلي لسقف NIH الذي اقترحه ترامب. الميزانية تمول التجربة لكنها تعترض على الجدران والكهرباء والتبريد. يعرض باحث خياراً عملياً.'),
 [choice(('Take a paid equipment-inventory shift', 'Inventariar equipos por un sueldo', 'Inventariare le attrezzature a pagamento', 'اقبل جرد المعدات مقابل أجر'), {'cash_cents':1400,'sanity':-1},
         ('Fourteen dollars earned. You confirm the freezer is an essential scientific instrument, not a lifestyle choice.', 'Ganas catorce dólares. Confirmas que el congelador es instrumental científico, no una elección de estilo de vida.', 'Guadagni quattordici dollari. Il congelatore è uno strumento scientifico essenziale, non una scelta di vita.', 'تكسب أربعة عشر دولاراً. تؤكد أن المجمد أداة علمية ضرورية وليس خياراً لنمط الحياة.')),
  choice(('Share a supply pack', 'Compartir provisiones', 'Condividere delle provviste', 'شارك حزمة مؤن'), {'supplies':-1,'hp':1,'allies':1},
         ('The research crew returns the favor with basic care and a new contact.', 'El equipo científico devuelve el favor con cuidados básicos y un nuevo contacto.', 'I ricercatori ricambiano con cure di base e un nuovo contatto.', 'يرد فريق البحث الجميل برعاية أساسية ومعرفة جديدة.')), PASS]),
('west_wind_loyalty', 'MountainWest', 'west_wind', 'enc-community',
 ('Wind Loyalty Hearing', 'Audiencia de lealtad del viento', 'Udienza sulla lealtà del vento', 'جلسة ولاء الرياح'),
 ('Following Trump’s order against wind and solar subsidies, a roadside energy fair is judging whether the breeze is sufficiently patriotic. A technician needs help that will actually keep equipment working.',
  'Tras el decreto de Trump contra las ayudas eólicas y solares, una feria energética juzga si la brisa es patriótica. Una técnica necesita ayuda que de verdad mantenga los equipos funcionando.',
  'Dopo il decreto di Trump contro i sussidi a eolico e solare, una fiera energetica valuta il patriottismo della brezza. Un tecnico chiede aiuto per far funzionare davvero le attrezzature.',
  'بعد أمر ترامب ضد دعم الرياح والشمس، يفحص معرض طاقة وطنية النسيم. يحتاج فني إلى مساعدة تبقي المعدات تعمل فعلاً.'),
 [choice(('Help inventory spare parts', 'Ayudar con el inventario de repuestos', 'Aiutare con l’inventario dei ricambi', 'ساعد في جرد قطع الغيار'), {'cash_cents':1000,'sanity':-1},
         ('Ten dollars earned. The wind keeps generating power without completing its loyalty questionnaire.', 'Ganas diez dólares. El viento sigue generando energía sin rellenar el cuestionario de lealtad.', 'Guadagni dieci dollari. Il vento produce energia senza compilare il questionario di lealtà.', 'تكسب عشرة دولارات. الرياح تواصل توليد الطاقة دون ملء استبيان الولاء.')),
  choice(('Record the technician’s testimony', 'Registrar el testimonio técnico', 'Registrare la testimonianza del tecnico', 'سجل شهادة الفني'), {'credibility':2,'allies':1,'sanity':-1},
         ('You gain a contact and an explanation involving actual electricity.', 'Consigues un contacto y una explicación basada en electricidad real.', 'Ottieni un contatto e una spiegazione basata sull’elettricità reale.', 'تكسب معرفة جديدة وشرحاً يتناول كهرباء حقيقية.')), PASS]),
('west_desert_pressure', 'Southwest', 'shower', 'enc-service',
 ('Desert Shower Freedom', 'Libertad de ducha en el desierto', 'Libertà di doccia nel deserto', 'حرية الاستحمام في الصحراء'),
 ('A desert service stop celebrates Trump’s shower-pressure order with a demonstration nobody budgeted water for. The plumber offers to end the ceremony before the tank runs dry.',
  'Una parada del desierto celebra el decreto de Trump sobre presión de ducha con una demostración para la que nadie presupuestó agua. La fontanera ofrece terminar antes de vaciar el depósito.',
  'Una stazione nel deserto celebra il decreto di Trump sulla pressione delle docce con una dimostrazione senza acqua a bilancio. L’idraulico propone di fermarla prima che il serbatoio si svuoti.',
  'تحتفل محطة صحراوية بأمر ترامب بشأن ضغط الدش بعرض لم يخصص أحد ماءً له. يقترح السباك إنهاء الحفل قبل نفاد الخزان.'),
 [choice(('Help shut off the demonstration', 'Ayudar a cerrar la demostración', 'Aiutare a fermare la dimostrazione', 'ساعد في إيقاف العرض'), {'supplies':2,'sanity':1},
         ('You leave with water and a working understanding of the off switch.', 'Te llevas agua y una comprensión práctica del interruptor de apagado.', 'Riparti con acqua e una conoscenza pratica dell’interruttore di spegnimento.', 'تغادر ومعك ماء وفهم عملي لزر الإيقاف.')),
  choice(('Take the paid plumbing shift', 'Aceptar el turno de fontanería', 'Accettare il turno da idraulico', 'اقبل وردية السباكة المدفوعة'), {'cash_cents':1200,'hp':-1},
         ('Twelve dollars earned. Working under the sink is less comfortable than announcing freedom above it.', 'Ganas doce dólares. Trabajar bajo el fregadero es menos cómodo que anunciar libertad encima.', 'Guadagni dodici dollari. Lavorare sotto il lavello è meno comodo che annunciare libertà da sopra.', 'تكسب اثني عشر دولاراً. العمل تحت الحوض أقل راحة من إعلان الحرية فوقه.')), PASS]),
('west_beef_passports', 'Southwest', 'west_beef', 'enc-convoy',
 ('America First, Argentina Next', 'Estados Unidos primero, Argentina después', 'America prima, Argentina dopo', 'أمريكا أولاً والأرجنتين تالياً'),
 ('Trump’s idea to import more Argentine beef has reached a cattle-country loading yard. The ranchers are comparing the America First speeches with the import paperwork. The manifest needs a second pair of eyes.',
  'La idea de Trump de importar más carne argentina llegó a un cargadero ganadero. Comparan los discursos de «Estados Unidos primero» con los papeles de importación. El manifiesto necesita otra revisión.',
  'L’idea di Trump di importare più manzo argentino è arrivata a uno scalo del bestiame. Gli allevatori confrontano i discorsi America First con i documenti d’importazione. Serve un altro controllo del manifesto.',
  'وصلت فكرة ترامب باستيراد مزيد من اللحم الأرجنتيني إلى ساحة تحميل الماشية. يقارن المربون خطابات «أمريكا أولاً» بأوراق الاستيراد. البيان يحتاج إلى مراجعة ثانية.'),
 [choice(('Take the paid manifest shift', 'Revisar el manifiesto por un sueldo', 'Controllare il manifesto a pagamento', 'اقبل وردية مراجعة البيان'), {'cash_cents':1600,'sanity':-1},
         ('Sixteen dollars earned. The cattle’s country of origin is clearer than the policy’s.', 'Ganas dieciséis dólares. El país de origen del ganado está más claro que el de la política.', 'Guadagni sedici dollari. L’origine del bestiame è più chiara di quella della politica.', 'تكسب ستة عشر دولاراً. بلد منشأ الماشية أوضح من منشأ السياسة.')),
  choice(('Compare the speeches with the receipts', 'Comparar discursos y documentos', 'Confrontare i discorsi con le prove', 'قارن الخطابات بالمستندات'), {'credibility':3,'sanity':-1},
         ('You add the contradiction to your evidence file. Nobody has yet asked the cows to apologize.', 'Añades la contradicción a las pruebas. Nadie ha pedido aún que las vacas se disculpen.', 'Aggiungi la contraddizione alle prove. Nessuno ha ancora chiesto scusa alle mucche.', 'تضيف التناقض إلى ملف الأدلة. لم يطلب أحد من الأبقار الاعتذار بعد.')), PASS]),
]

def install():
    path = ROOT / 'game.json'
    events = json.loads(path.read_text())
    ids = {e[0] for e in EVENTS}
    events = [e for e in events if e['id'] not in ids]
    # These stories are national policy encounters without a town-specific claim.
    western = ['PacificCoast', 'MountainWest', 'Southwest']
    national = {'raw_milk','tariff_whiplash','clinic_triage','classic_mutual_aid',
                'classic_water_drive','classic_service_station','classic_media_training',
                'deep_secure_line','deep_media_ambush','fundraiser_detour'}
    for event in events:
        if event['id'] in national:
            event['regions'] = list(dict.fromkeys(event['regions'] + western))
    localized = {lang: {} for lang in LANGS}
    for eid, region, hook, scene, names, descriptions, choices in EVENTS:
        event = dict(id=eid, name=names[0], desc=descriptions[0], weight=7,
                     regions=[region], modes=['classic','deep','deep_end'],
                     satire_hook=hook, scene=scene, choices=[])
        for i, (labels, effects, logs) in enumerate(choices):
            event['choices'].append(dict(label=labels[0], effects={**effects, 'log':logs[0]}))
            for j, lang in enumerate(LANGS):
                copy = localized[lang].setdefault(eid, dict(name=names[j], desc=descriptions[j]))
                copy[f'choice_{i}'] = labels[j]
                copy[f'log_{i}'] = logs[j]
        events.append(event)
    apply_evidence(events)
    path.write_text(json.dumps(events,ensure_ascii=False,indent=2)+'\n')
    for path in Path('dystrail-web/i18n').glob('*.json'):
        data = json.loads(path.read_text())
        data['encounter_copy'].update(localized.get(path.stem,localized['en']))
        path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')
    sources = ROOT / 'satire-sources.json'
    data = json.loads(sources.read_text())
    for hook, town, title, date in [
        ('west_grants','San Diego','Grant language under federal review','2025'),
        ('west_rail','Sacramento','California rail funding rescinded','2026-02-28'),
        ('west_labs','Salt Lake City','Proposed NIH overhead cap','2025-02'),
        ('west_wind','Cheyenne','Wind and solar subsidy order','2025-07-09'),
        ('west_beef','Amarillo','Texas ranchers object to beef import proposal','2025-10-20')]:
        row = next(f for f in FACTS if f[0] == town)
        data[hook] = dict(title=title,date=date,source=row[1],fact=row[3][0],
                         qualification='Historical record; this encounter, dialogue and prices are fictional.',
                         checked='2026-09-11',translations=dict(zip(LANGS[1:],row[3][1:],strict=True)))
    sources.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')


if __name__ == '__main__':
    install()
