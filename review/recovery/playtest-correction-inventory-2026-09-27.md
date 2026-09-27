# Playtest visual correction inventory — 2026-09-27

This is the correction ledger for the current recovery worktree. Counts are **scene variants or locations**, not image-generation requests. Categories can overlap. Reuse and composition come before new art; no additional generation is authorized by this ledger.

## Directly observed in the 27 September playtest

| Area | Correction | Production decision |
|---|---|---|
| All world scenes | Enforce the same short scene frame on a given viewport; keep actions in reach, retain bottom-left captions, avoid stretching figures | CSS camera/crop plus desktop and phone inspection of every scene family |
| Moving van | Seat the actual surviving crew in front of seatbacks with consistent scale and windshield/door occlusion | Correct existing 1536×1024 composite; reuse approved cast/van art |
| Moving billboards | Make signs pass the van, space them evenly and farther apart, freeze each sign’s copy, increase the run-seeded pool to at least 3× the measured mean signs per leg | Layout/selection/copy; reuse text-free props and translate copy |
| Clean-air banner and other blank signs | Determine whether intentionally blank source art lacks its HTML overlay; add localized satire in interface layer only | No embedded text in raster art |
| Community-offer outcomes | Bind retained outcome compositions where correct; hold mismatches | Map each offer/result to an approved asset |
| Hotel biscuit scene | Hotel lift bank plus stairs, with a coherent biscuit/jet premise | ENC-C18-B: retained asset search, then review hold if none fits |
| 51 towns | Show recognizable local geography or landmarks rather than generic town art | Reference publicly available current place imagery, then use approved pixel-art style with attributable source review |
| Result scene | Keep scorecard and navigation reachable; the pending crossing scene must show its Continue action even after the run enters the Result phase | Render decision children without the ordinary journey toolbar in Result; rename the misleading story title |

## Existing asset gaps confirmed by the scene ledger

- **Road decisions: 35 variants** still use a historical fallback: 24 need the stated location and 11 need an essential visual gag. These are variants, not automatically 35 new images. Four further road variants are explicit held fallbacks: ENC-C02-A, ENC-C04-A, ENC-C07-C, ENC-C08-A.
- **Ally/community offers: 12 held art mismatches**; each requires a correct retained scene or a review hold.
- **Care: 9 integrated setting backgrounds still lack a credibly scaled in-scene affected traveler; 15 other care scenes/outcomes remain. CARE-07-A is rejected and excluded.**
- **Crossings: 4 compositions** remain after retained integration: CROSS-01-A, CROSS-01-C, CROSS-02C-C, CROSS-03-A.
- **Generated panels on hold: 3 variants** (ENC-S17-B, ENC-S18-A, TOWN-01-A). These failed visual review and cannot be silently bound.
- **Town geography: 51 stops** need a per-stop recognizability audit; the current text-profile inventory does not prove their artwork is correct.

The localization audit found 20 locale files. English has 3,904 leaf keys; the other 19 currently omit between 1,596 and 2,355 of those keys, with additional long strings still identical to English. Existing English fallback is a functional safeguard, not a completed translation. This is a measured content-production gap and must remain open in acceptance reports.

Recent-record audit: the Spokane 1909 free-speech variant (`TOWN-37-C`) has been replaced with the city's documented 2025 aerospace-funding rescission. A text/source-date screen flags 22 other town-conversation variants for editorial review because their cited source is older or undated and their English copy lacks an explicit recent federal hook: `TOWN-06-C`, `08-C`, `11-B`, `14-A`, `14-B`, `15-C`, `16-B`, `17-C`, `20-B`, `22-B`, `24-C`, `25-C`, `26-C`, `28-B`, `30-C`, `33-B`, `36-B`, `39-C`, `40-C`, `41-C`, `42-B`, `44-B`. This is a triage list, not proof that all 22 are factually wrong. Each needs a verified current source and localized copy before acceptance.

Billboard sizing basis: 106 legs across six routes average 173.2 miles. At one billboard per 60-mile travel step, the average is 3.34 billboards per leg; three times that mean rounds up to 11 distinct messages. The retained pool has 12 messages already translated across the 20 locale files. The longest leg can show ten signs, so the full pool cycles without repetition during a normal leg.

Implemented in the recovery branch: one run-seeded sign per driving step, one viewport crossing per step, unchanged image and copy during that crossing, with the same image/headline/copy on phone. All 12 messages have three nonempty fields in every locale file. Browser fit still needs rechecking after the local browser executable is restored; field presence alone does not validate translation quality or layout.

### Road location variants

- `ENC-C14-B` — Outdoor repair fair and first-aid chair
- `ENC-D02-B` — Indoor school gym equipment sale
- `ENC-D02-C` — Nursing-home rummage sale
- `ENC-D08-A` — Pregnancy-center open house with advertising display
- `ENC-D08-B` — Union-hall table sale
- `ENC-D10-B` — School-supply sale
- `ENC-S01-C` — Veterans hall supper
- `ENC-S04-C` — Marina visitor area
- `ENC-S06-C` — Roadside letter stall
- `ENC-S08-C` — Souvenir printing stall beside food counter
- `ENC-S10-B` — Mountain-photograph souvenir stall
- `ENC-S12-A` — Grocery checkout and crates
- `ENC-S12-B` — Discount-store loading bay
- `ENC-S12-C` — Catering depot
- `ENC-S13-B` — Food-and-water shop checkout
- `ENC-S13-C` — Food-and-water shop and lost property
- `ENC-S14-B` — Clinic fundraiser with chairs
- `ENC-S18-B` — Souvenir printer and postcard racks
- `ENC-S19-C` — Event printer and disclosure inserts
- `ENC-S26-A` — Transport supporters leaflet stall
- `ENC-S28-A` — House open-house cleanup
- `ENC-S30-A` — Coastal transport meeting
- `ENC-S32-A` — Outdoor turbine repair stall
- `ENC-S34-C` — Baby-goods warehouse

### Road essential-gag variants

`ENC-C16-C`, `ENC-C18-B`, `ENC-D06-B`, `ENC-D09-A`, `ENC-S03-B`, `ENC-S05-A`, `ENC-S07-C`, `ENC-S09-A`, `ENC-S09-C`, `ENC-S10-A`, `ENC-S17-B`

### Ally/community mismatches

- `ALLY-01-B` — Straw-hat stall replaces plain hat cartons, customs bill and crew phone request.
- `ALLY-01-C` — Route map and lunch replace enormous donor microphone and disconnected small microphone.
- `ALLY-02-B` — Opposing arrow sheets replace disconnected headset and packed belongings.
- `ALLY-03-A` — Bank/mailbox lobby replaces required garage invoice, apron and overnight-shift alarm.
- `ALLY-03-B` — Cash-register scene lacks the medical bill under a second-job timecard.
- `ALLY-03-C` — Apron is present but groceries look abundant, contradicting sparse food budget. Preserve identity; revise bag/second-job context.
- `ALLY-04-A` — Gaming classroom replaces rival-color merchandise tables and paired card readers.
- `ALLY-04-C` — Miniature objects replace laptop, monthly calendar and theatrical countdown.
- `ALLY-05-C` — Football trophy and award replace discarded receipt and blocked phone message.
- `ALLY-06-A` — Industrial shoulder injury replaces injured ankle, closed ramp and privileged entry.
- `ALLY-06-B` — Machine-shop consultation replaces cleaning equipment and empty service desk.
- `ALLY-06-C` — Sore back and ice pack replace sore shoulder, speaker handcart and distant donor stagehands.

### Crossing compositions

- `CROSS-01-A` — Officer inspecting open hood and ordinary alternator of actual stopped van; no dismantling or damage
- `CROSS-01-C` — Officer at actual van manufacturer plate; actual waiting travelers and disproportionately tall blank forms stack
- `CROSS-02C-C` — Steel quote, solitary cone and unfinished repair area with actual van safely on approach
- `CROSS-03-A` — Official looking into actual van; cash tray and travelers case folder visible


### Town location source inventory

The current town scene uses a regional setting, not a town-specific landmark. These 51 existing profile subjects and source pages are research leads only; none is an approved image license or a completed visual composition.

| Town | Existing local subject | Source page |
|---|---|---|
| Minneapolis | The Stone Arch Bridge opened in 1883 as a railroad bridge and later became a walking and cycling crossing. | [source](https://www.minneapolisparks.org/parks-destinations/historical_sites/stone_arch_bridge/) |
| La Crosse | Riverside Park looks over the Mississippi River; its International Friendship Gardens include plants from around the world. | [source](https://explorelacrosse.com/things-to-do/sights-attractions/riverside-park/) |
| Madison | Downtown Madison occupies an isthmus between Lakes Mendota and Monona. | [source](https://em.danecounty.gov/documents/pdf/2022-Hazard-Mitigation-Plan/DCNHMP22---3---Muni-Section---Cities-.pdf) |
| Chicago | Millennium Park’s Cloud Gate reflects the Chicago skyline in a polished steel sculpture. | [source](https://www.millenniumparkfoundation.org/art-architecture) |
| South Bend | South Bend’s East Race Waterway is an urban whitewater course. The city announced it would be closed for upgrades in the 2026 season. | [source](https://311.southbendin.gov/knowledgecenter/article/?id=KA-04513) |
| Toledo | Glass City Center stands in downtown Toledo, a short block from the Maumee River and about six miles from Lake Erie. | [source](https://www.glasscitycenter.com/p/events/location) |
| Cleveland | The Cuyahoga empties into Lake Erie at Cleveland. Its 1969 fire helped galvanize the national water-pollution response. | [source](https://www.nps.gov/articles/000/cuyahoga-national-heritage-river.htm) |
| Pittsburgh | Pittsburgh’s Three Rivers Water Trail provides access to the Allegheny, Monongahela and Ohio rivers. | [source](https://www.nps.gov/places/three-rivers-water-trail.htm) |
| Cumberland | The C&O Canal reached Cumberland in 1850. Today its towpath meets the Great Allegheny Passage here. | [source](https://pubs.nps.gov/eTIC/OLYM-POPO/POHE_866_124321_0001_of_0046.pdf) |
| Hagerstown | Jonathan Hager founded the settlement in 1762. His house survives as a city museum. | [source](https://www.hagerstownmd.org/309/Jonathan-Hager-House-Museum) |
| Frederick | Downtown Frederick is known for its clustered church spires. | [source](https://www.visitfrederick.org/plugins/maps/map/downtown-frederick/5be4a7fea62e703dcef330bd/) |
| D.C. | The C&O Canal connected communities along the Potomac with markets in D.C.; it carried coal, lumber and farm products. | [source](https://www.nps.gov/choh/index.htm?vm=r) |
| Kansas City | Kansas City’s many public fountains gave it the nickname City of Fountains. | [source](https://www.visitkc.com/articles/first-timers-guide-kansas-city/) |
| Columbia | The MKT Trail follows an old railroad bed from downtown Columbia to the Katy Trail near McBaine. | [source](https://www.como.gov/trails/mkt-nature-and-fitness-trail/) |
| St. Louis | The Gateway Arch is 630 feet tall and equally wide at ground level. | [source](https://www.nps.gov/jeff/faqs.htm) |
| Springfield, IL | Abraham Lincoln bought the Springfield house at Eighth and Jackson Streets in 1844 and left for D.C. in 1861. | [source](https://home.nps.gov/liho/learn/historyculture/alincolnbio.htm) |
| Denver | Denver’s mile-high elevation is commemorated on the west steps of the Colorado State Capitol. | [source](https://content.leg.colorado.gov/sites/default/files/images/visitor_brochure_for_web_accessible.pdf) |
| North Platte | Union Pacific’s Bailey Yard in North Platte is a major rail sorting hub, with 315 miles of track. | [source](https://www.visitnorthplatte.com/things-to-do/attractions/trains-railroads/bailey-yard/) |
| Omaha | The Bob Kerrey Pedestrian Bridge crosses the Missouri River between Nebraska and Iowa. | [source](https://www.visitomaha.com/) |
| Des Moines | The Pappajohn Sculpture Park opened in downtown Des Moines in 2009, on 4.4 acres. | [source](https://desmoinesartcenter.org/visit/pappajohn-sculpture-park/) |
| Iowa City | Iowa City became a UNESCO City of Literature in November 2008, the first such city in the United States. | [source](https://www.iowacityofliterature.org/wp-content/uploads/2023/09/2022-2023-Annual-Report.pdf) |
| Austin | The Congress Avenue Bridge shelters one of North America’s largest urban bat colonies. | [source](https://www.austintexas.gov/page/bats) |
| Waco | Waco Mammoth National Monument preserves a nursery herd of Columbian mammoths, including females and young. | [source](https://www.nps.gov/places/waco-mammoth-herd.htm) |
| Dallas | Fair Park preserves a major collection of 1930s Art Deco architecture, with museums and public art. | [source](https://www.fairparkdallas.com/sites-and-attractions) |
| Oklahoma City | The Bricktown Canal was one of nine projects in the original MAPS initiative approved by voters in 1993. | [source](https://www.okc.gov/News-articles/Bricktown-Canal-celebrates-25-year-milestone) |
| Tulsa | Tulsa’s 11th Street bridge crossed the Arkansas River in 1917; it was renamed for Route 66 advocate Cyrus Avery in 2004. | [source](https://www.nps.gov/places/11th-street-arkansas-river-bridge.htm) |
| Joplin | Joplin grew as a lead and zinc mining town. Its history museum preserves that mining heritage. | [source](https://www.joplin-museum.org/faq) |
| Springfield, MO | The name Route 66 was proposed from Springfield on April 30, 1926. | [source](https://www.springfieldmo.org/about-springfield/history/) |
| Seattle | Pike Place Market — the waterfront public market. | [source](https://www.pikeplacemarket.org/) |
| Portland | Washington Park — gardens, museums and wooded trails. | [source](https://www.explorewashingtonpark.org/) |
| San Francisco | The Presidio — a former military post turned national park site. | [source](https://www.nps.gov/prsf/index.htm) |
| Los Angeles | Griffith Observatory — astronomy and views across the city. | [source](https://griffithobservatory.lacity.gov/) |
| Sacramento | California State Railroad Museum — locomotives and the story of western rail. | [source](https://www.californiarailroad.museum/) |
| San Diego | Balboa Park — museums, gardens and Spanish Colonial Revival architecture. | [source](https://balboapark.org/) |
| Spokane | Riverfront Park — Spokane Falls and the former world’s fair grounds. | [source](https://my.spokanecity.org/riverfrontspokane/) |
| Missoula | A Carousel for Missoula — a community-built, hand-carved carousel. | [source](https://missoulacarousel.org/) |
| Billings | Moss Mansion — a historic house museum. | [source](https://mossmansion.com/) |
| Rapid City | Nearby Mount Rushmore — the carved granite memorial in the Black Hills. | [source](https://www.nps.gov/moru/index.htm) |
| Sioux Falls | Falls Park — the waterfalls of the Big Sioux River. | [source](https://www.experiencesiouxfalls.com/falls-park) |
| Boise | Idaho State Capitol — the state’s historic seat of government. | [source](https://capitolcommission.idaho.gov/) |
| Salt Lake City | Natural History Museum of Utah — fossils and the landscapes of the Intermountain West. | [source](https://nhmu.utah.edu/) |
| Reno | National Automobile Museum — historic cars and recreated street scenes. | [source](https://automuseum.org/) |
| Cheyenne | Cheyenne Depot Museum — railroad history in the restored Union Pacific depot. | [source](https://www.cheyennedepotmuseum.org/) |
| Las Vegas | The Neon Museum — rescued signs from Las Vegas history. | [source](https://neonmuseum.org/) |
| Flagstaff | Lowell Observatory — the observatory where Pluto was discovered. | [source](https://lowell.edu/) |
| Albuquerque | Albuquerque Museum — art and history near Old Town. | [source](https://www.cabq.gov/artsculture/albuquerque-museum) |
| Amarillo | Nearby Palo Duro Canyon State Park — red-rock canyon trails. | [source](https://tpwd.texas.gov/state-parks/palo-duro-canyon) |
| Phoenix | Heard Museum — American Indian art and culture. | [source](https://heard.org/) |
| Tucson | Saguaro National Park — giant cacti on both sides of Tucson. | [source](https://www.nps.gov/sagu/index.htm) |
| El Paso | Chamizal National Memorial — the peaceful settlement of a U.S.–Mexico boundary dispute. | [source](https://www.nps.gov/cham/index.htm) |
| San Antonio | San Antonio Missions — historic mission sites along the river. | [source](https://www.nps.gov/saan/index.htm) |

## Acceptance rules

- No embedded text or religious symbols in generated or retained art. HTML/CSS supplies all readable signage and is localized.
- Keep 768×512 selected atlas cells at native 3:2 proportions and the approved camera/perspective rules in `scene-cell-review.json`; place actual crew from game state, not painted substitute occupants.
- Check each candidate at desktop and phone widths for crop, subjects, foreground occlusion, captions, and reachable actions. A successful build or non-empty image is insufficient.
- The approved 19 image-generation attempts have been used. User review is required for further image-generation scope; no retries are automatic.
- Complete all 20 selected locales across interface and active content, and verify native meaning and fit. Existing English fallback and matching key tests do not meet this finish line.
- Preserve 644 accounted content records and keep 36 mechanics-dependent records inactive.
