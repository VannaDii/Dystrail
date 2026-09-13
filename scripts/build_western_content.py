"""Install authored, source-checked western town content into the offline data files."""
import json
from pathlib import Path
from western_town_facts import FACTS

ROOT = Path('dystrail-web/static/assets/data')
LANGS = ('en', 'es', 'it', 'ar')


def localized(*copy):
    return dict(zip(LANGS, copy, strict=True))


# Population is the April 1, 2020 Census count, not an estimate or metro population.
PROFILES = [
('Seattle', 'Washington', 737015, 'seattlecitywashington', 'https://www.pikeplacemarket.org/',
 ('Pike Place Market — the waterfront public market.', 'Pike Place Market: el mercado público junto al litoral.', 'Pike Place Market: il mercato pubblico sul lungomare.', 'سوق بايك بليس العام قرب الواجهة البحرية.')),
('Portland', 'Oregon', 652503, 'portlandcityoregon', 'https://www.explorewashingtonpark.org/',
 ('Washington Park — gardens, museums and wooded trails.', 'Washington Park: jardines, museos y senderos arbolados.', 'Washington Park: giardini, musei e sentieri nel bosco.', 'حديقة واشنطن: حدائق ومتاحف ومسارات بين الأشجار.')),
('San Francisco', 'California', 873965, 'sanfranciscocitycalifornia', 'https://www.nps.gov/prsf/index.htm',
 ('The Presidio — a former military post turned national park site.', 'El Presidio: un antiguo puesto militar convertido en parque nacional.', 'Il Presidio: un ex presidio militare diventato area di parco nazionale.', 'بريسيديو: موقع عسكري سابق أصبح جزءاً من الحدائق الوطنية.')),
('Los Angeles', 'California', 3898747, 'losangelescitycalifornia', 'https://griffithobservatory.lacity.gov/',
 ('Griffith Observatory — astronomy and views across the city.', 'Observatorio Griffith: astronomía y vistas de la ciudad.', 'Osservatorio Griffith: astronomia e panorami sulla città.', 'مرصد غريفيث: علم الفلك وإطلالات على المدينة.')),
('Sacramento', 'California', 524943, 'sacramentocitycalifornia', 'https://www.californiarailroad.museum/',
 ('California State Railroad Museum — locomotives and the story of western rail.', 'Museo Estatal del Ferrocarril: locomotoras e historia ferroviaria del oeste.', 'Museo ferroviario statale della California: locomotive e storia delle ferrovie dell’Ovest.', 'متحف سكك حديد كاليفورنيا: قاطرات وتاريخ السكك في الغرب.')),
('San Diego', 'California', 1386932, 'sandiegocitycalifornia', 'https://balboapark.org/',
 ('Balboa Park — museums, gardens and Spanish Colonial Revival architecture.', 'Parque Balboa: museos, jardines y arquitectura neocolonial española.', 'Balboa Park: musei, giardini e architettura neocoloniale spagnola.', 'حديقة بالبوا: متاحف وحدائق وعمارة مستوحاة من الطراز الاستعماري الإسباني.')),
('Spokane', 'Washington', 228989, 'spokanecitywashington', 'https://my.spokanecity.org/riverfrontspokane/',
 ('Riverfront Park — Spokane Falls and the former world’s fair grounds.', 'Riverfront Park: las cataratas de Spokane y la antigua exposición universal.', 'Riverfront Park: le cascate di Spokane e l’ex area dell’esposizione universale.', 'حديقة ريفرفرونت: شلالات سبوكان وموقع المعرض العالمي السابق.')),
('Missoula', 'Montana', 73489, 'missoulacitymontana', 'https://missoulacarousel.org/',
 ('A Carousel for Missoula — a community-built, hand-carved carousel.', 'A Carousel for Missoula: un carrusel de madera tallado y construido por la comunidad.', 'A Carousel for Missoula: una giostra intagliata a mano, costruita dalla comunità.', 'دوامة ميسولا: خيول منحوتة يدوياً ومشروع أنجزه المجتمع المحلي.')),
('Billings', 'Montana', 117116, 'billingscitymontana', 'https://mossmansion.com/',
 ('Moss Mansion — a historic house museum.', 'Moss Mansion: una casa histórica convertida en museo.', 'Moss Mansion: una dimora storica trasformata in museo.', 'قصر موس: منزل تاريخي تحول إلى متحف.')),
('Rapid City', 'South Dakota', 74703, 'rapidcitycitysouthdakota', 'https://www.nps.gov/moru/index.htm',
 ('Nearby Mount Rushmore — the carved granite memorial in the Black Hills.', 'Cerca: monte Rushmore, el monumento tallado en granito de las Black Hills.', 'Nelle vicinanze: il Monte Rushmore, monumento scolpito nel granito delle Black Hills.', 'بالقرب: جبل راشمور، النصب المنحوت في غرانيت بلاك هيلز.')),
('Sioux Falls', 'South Dakota', 192517, 'siouxfallscitysouthdakota', 'https://www.experiencesiouxfalls.com/falls-park',
 ('Falls Park — the waterfalls of the Big Sioux River.', 'Falls Park: las cascadas del río Big Sioux.', 'Falls Park: le cascate del fiume Big Sioux.', 'حديقة فولز: شلالات نهر بيغ سيو.')),
('Boise', 'Idaho', 235684, 'boisecitycityidaho', 'https://capitolcommission.idaho.gov/',
 ('Idaho State Capitol — the state’s historic seat of government.', 'Capitolio de Idaho: la sede histórica del gobierno estatal.', 'Campidoglio dell’Idaho: la sede storica del governo statale.', 'كابيتول أيداهو: المقر التاريخي لحكومة الولاية.')),
('Salt Lake City', 'Utah', 199723, 'saltlakecitycityutah', 'https://nhmu.utah.edu/',
 ('Natural History Museum of Utah — fossils and the landscapes of the Intermountain West.', 'Museo de Historia Natural de Utah: fósiles y paisajes del oeste intermontano.', 'Museo di storia naturale dello Utah: fossili e paesaggi dell’Ovest intermontano.', 'متحف يوتا للتاريخ الطبيعي: أحافير ومناظر الغرب بين الجبال.')),
('Reno', 'Nevada', 264165, 'renocitynevada', 'https://automuseum.org/',
 ('National Automobile Museum — historic cars and recreated street scenes.', 'Museo Nacional del Automóvil: coches históricos y calles de época recreadas.', 'Museo nazionale dell’automobile: auto storiche e strade d’epoca ricostruite.', 'المتحف الوطني للسيارات: سيارات تاريخية وشوارع قديمة معاد إنشاؤها.')),
('Cheyenne', 'Wyoming', 65132, 'cheyennecitywyoming', 'https://www.cheyennedepotmuseum.org/',
 ('Cheyenne Depot Museum — railroad history in the restored Union Pacific depot.', 'Museo del Depósito de Cheyenne: historia ferroviaria en la estación restaurada de Union Pacific.', 'Cheyenne Depot Museum: storia ferroviaria nell’ex stazione Union Pacific restaurata.', 'متحف محطة شايان: تاريخ السكك الحديدية في محطة يونيون باسيفيك المرممة.')),
('Las Vegas', 'Nevada', 641903, 'lasvegascitynevada', 'https://neonmuseum.org/',
 ('The Neon Museum — rescued signs from Las Vegas history.', 'Museo del Neón: rótulos rescatados de la historia de Las Vegas.', 'Museo del neon: insegne salvate dalla storia di Las Vegas.', 'متحف النيون: لافتات محفوظة من تاريخ لاس فيغاس.')),
('Flagstaff', 'Arizona', 76831, 'flagstaffcityarizona', 'https://lowell.edu/',
 ('Lowell Observatory — the observatory where Pluto was discovered.', 'Observatorio Lowell: donde se descubrió Plutón.', 'Osservatorio Lowell: il luogo della scoperta di Plutone.', 'مرصد لويل: حيث اكتُشف بلوتو.')),
('Albuquerque', 'New Mexico', 564559, 'albuquerquecitynewmexico', 'https://www.cabq.gov/artsculture/albuquerque-museum',
 ('Albuquerque Museum — art and history near Old Town.', 'Museo de Albuquerque: arte e historia junto al casco antiguo.', 'Museo di Albuquerque: arte e storia accanto alla città vecchia.', 'متحف ألبوكيركي: الفن والتاريخ قرب البلدة القديمة.')),
('Amarillo', 'Texas', 200393, 'amarillocitytexas', 'https://tpwd.texas.gov/state-parks/palo-duro-canyon',
 ('Nearby Palo Duro Canyon State Park — red-rock canyon trails.', 'Cerca: parque estatal del cañón Palo Duro, con senderos entre rocas rojizas.', 'Nelle vicinanze: il parco statale del canyon Palo Duro, con sentieri tra rocce rosse.', 'بالقرب: حديقة وادي بالو دورو ومسارات الصخور الحمراء.')),
('Phoenix', 'Arizona', 1608139, 'phoenixcityarizona', 'https://heard.org/',
 ('Heard Museum — American Indian art and culture.', 'Museo Heard: arte y cultura de los pueblos indígenas de Estados Unidos.', 'Museo Heard: arte e cultura dei popoli nativi americani.', 'متحف هيرد: فن وثقافة الشعوب الأصلية الأمريكية.')),
('Tucson', 'Arizona', 542629, 'tucsoncityarizona', 'https://www.nps.gov/sagu/index.htm',
 ('Saguaro National Park — giant cacti on both sides of Tucson.', 'Parque Nacional Saguaro: cactus gigantes a ambos lados de Tucson.', 'Parco nazionale dei Saguaro: cactus giganti sui due lati di Tucson.', 'حديقة ساغوارو الوطنية: صبّار عملاق على جانبي توسان.')),
('El Paso', 'Texas', 678815, 'elpasocitytexas', 'https://www.nps.gov/cham/index.htm',
 ('Chamizal National Memorial — the peaceful settlement of a U.S.–Mexico boundary dispute.', 'Monumento Nacional Chamizal: un acuerdo pacífico sobre la frontera entre México y Estados Unidos.', 'Memoriale nazionale Chamizal: la soluzione pacifica di una disputa di confine tra Stati Uniti e Messico.', 'نصب شاميزال الوطني: تسوية سلمية لنزاع حدودي بين الولايات المتحدة والمكسيك.')),
('San Antonio', 'Texas', 1434625, 'sanantoniocitytexas', 'https://www.nps.gov/saan/index.htm',
 ('San Antonio Missions — historic mission sites along the river.', 'Misiones de San Antonio: conjuntos históricos junto al río.', 'Missioni di San Antonio: complessi storici lungo il fiume.', 'بعثات سان أنطونيو: مواقع تاريخية على امتداد النهر.')),
]


def install_profiles():
    path = ROOT / 'town-profiles.json'
    profiles = json.loads(path.read_text())
    existing = {row['town']: row for row in profiles}
    names = {row[0] for row in PROFILES}
    profiles = [row for row in profiles if row['town'] not in names]
    for town, state, population, slug, source, attraction in PROFILES:
        profile = dict(town=town, state=state, population=population,
            population_year=2020,
            population_source=f'https://www.census.gov/quickfacts/fact/table/{slug}/POP010220',
            attraction=localized(*attraction), attraction_source=source)
        previous = existing.get(town, {})
        if previous.get('population_year', 0) > 2020:
            profile.update({key: value for key, value in previous.items()
                            if key == 'population' or key.startswith('population_')})
        profiles.append(profile)
    path.write_text(json.dumps(profiles, ensure_ascii=False, indent=2) + '\n')


if __name__ == '__main__':
    install_profiles()
    path = ROOT / 'town-facts.json'
    names = {row[0] for row in FACTS}
    facts = [row for row in json.loads(path.read_text()) if row['town'] not in names]
    for town, source, status, fact, remark in FACTS:
        facts.append(dict(town=town, source=source, checked='2026-09-11',
            administration_hook=True, evidence_status=status, supporting_sources=[],
            text=localized(*fact), comment=localized(*remark)))
    path.write_text(json.dumps(facts, ensure_ascii=False, indent=2) + '\n')
