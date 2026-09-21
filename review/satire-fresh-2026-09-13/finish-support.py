import json
from pathlib import Path

ROOT=Path(__file__).resolve().parent
units=json.loads((ROOT/'support.json').read_text())
by={u['id']:u for u in units}

# Keep selection/integration warnings out of player-facing epilogues.
better={
 'END-DESTROYED-B':'The van is destroyed. Your journey ends with a vehicle that has finally become as immobile as the people you wanted to argue with.',
 'END-DESTROYED-C':'The van is destroyed and the trip ends. A roadside advert offers a fresh start with low monthly payments. The photograph shows a different class of problem.',
 'END-COLD-A':'Cold exposure ends the expedition. The journey stops in conditions no warm words could change.',
 'END-COLD-C':'Exposure to cold ends the trip. The van’s roof was never the same thing as protection from everything beneath the sky.',
 'END-HEAT-A':'Heat exposure ends the expedition. Calling it a beautiful day could not make it a safe one.',
 'END-COLLAPSE-HUNGER-C':'Hunger brings the trip to an end. An inspiring story about sacrifice cannot be substituted for lunch.',
 'END-COLLAPSE-VEHICLE-A':'Vehicle trouble brings the journey to a halt. The plan depended on getting somewhere. The van has withdrawn that particular feature.',
 'END-COLLAPSE-VEHICLE-C':'Vehicle trouble ends the expedition. The view beyond the windscreen has become rather more permanent than anyone intended.',
 'END-COLLAPSE-WEATHER-A':'Weather causes the journey to collapse. The sky has declined to honor the itinerary.',
 'END-COLLAPSE-WEATHER-C':'Weather ends the journey. The road remains on the map, which has enjoyed far better protection from the elements than you have.',
 'END-COLLAPSE-BREAKDOWN-B':'The expedition ends with the van still broken. A spare part was the distance between a journey and a parking place.',
 'END-COLLAPSE-DISEASE-A':'Illness ends the expedition. The road ahead remains, but continuing is no longer possible.',
 'END-COLLAPSE-DISEASE-B':'Disease ends this journey. The people carrying the case were always more than its delivery system.',
 'END-COLLAPSE-DISEASE-C':'Illness brings the trip to an end. The case cannot travel farther than the people carrying it.',
 'END-COLLAPSE-CROSSING-A':'The crossing failure ends the expedition. The road ahead remains a direction rather than an available route.',
 'END-COLLAPSE-CROSSING-B':'The crossing failure ends the journey. Your case has encountered an obstacle that does not accept arguments.',
 'END-COLLAPSE-CROSSING-C':'The expedition ends at the crossing. The destination remains elsewhere. A sign indicating a route has proved different from being allowed to take it.',
 'END-COLLAPSE-PANIC-A':'Panic ends the expedition. The demands of the road have become more than you can carry. The itinerary is put down.',
 'END-COLLAPSE-PANIC-B':'Panic brings the journey to an end. There is no final argument to make here. For now, the road stops.',
 'END-COLLAPSE-PANIC-C':'Panic ends the trip. The road continues beyond the windscreen; the expedition does not.',
 'END-INCOMPLETE-B':'This journey is unfinished. The case is still packed, the road still leads onward, and D.C. has not noticed its lucky postponement.',
 'END-FALLBACK-A':'The available record does not say how the journey ended. Even a game about government can decline to turn missing paperwork into a victory announcement.',
 'END-FALLBACK-B':'The ending is missing from this record. The facts you do have remain. No press secretary has been hired to fill the gap.',
 'END-FALLBACK-C':'This record does not establish an ending. The rest remains unknown, a rare moment of restraint for an official-looking screen.'
}
for key,text in better.items():by[key]['epilogue']=text

def scenes(family,rows):
    for v,row in zip('ABC',rows):
        if isinstance(row,str):desc,labels=row,[]
        else:desc,labels=row
        key=f'{family}-{v}'
        by[key]['scene']={
            'scene_id':key.lower()+'-scene',
            'description':desc,
            'cast':'Use active roster appearances; no invented traveler. External contacts and officials are separate NPCs.',
            'composition':'Readable at phone size: one focal action, clear silhouettes, uncluttered background; use the game’s established pixel-art style.',
            'overlays':[{'text':label,'placement':'On the indicated blank prop or a UI layer adjacent to it','display_condition':'Only when this version is selected; localize independently'} for label in labels],
            'asset_notes':'No embedded lettering, readable text, numbers, logos, captions, interface text or watermarks. Keep written surfaces blank; all words are editable overlays.'
        }
        by[key]['illustration']=desc

care_scenes={
 '01':[
 ('Roadside milk stall with a beaming vendor and a traveler hurrying toward a closed portable-toilet door. A smiling cow illustration on a blank bottle contrasts with the traveler’s distress.',['Make America Healthy Again','Alive with goodness']),
 ('Sunlit market table with unrefrigerated milk bottles. A sick traveler leans safely over a bucket while the vendor proudly points to a blank sign.',['Free from government interference']),
 ('Pasture-side stall: a sick traveler rests beside the van while an unconcerned cow chews the corner of a blank sign. The vendor gestures toward the cow as an authority.',[])],
 '02':[
 ('Loading bay in harsh sun: an affected traveler sits in shade, water nearby. A supervisor holds an oversized marker beside a blank warning panel with a dark scribble area.',['HEAT WARNING','#sharpiegate']),
 ('Outdoor event setup: an exhausted traveler rests beside tent poles. Through an open flap, air-conditioning cools a neat row of bottled water in an empty keynote chair.',['Worker Resilience']),
 ('Warehouse yard: the dehydrated traveler rests by the van. Through a locked office window, a water cooler sits beyond reach while the supervisor holds the key.',[])],
 '03':[
 ('Parked van at dawn, a tired traveler surrounded by duplicated blank work forms. Two almost identical piles have swallowed the passenger seat.',[]),
 ('Dim van cabin: exhausted traveler at a tablet with blank training panels; dawn glows through the window. A medal-shaped blank icon hovers as an editable UI element.',['Work-Life Balance']),
 ('Van at a quiet stop: a tired traveler retypes beside several identical unlettered résumé sheets. A cheerful blank chatbot bubble occupies a separate overlay region.',[])],
 '04':[
 ('Market wellness table: the sick traveler faces a sample bucket, away from the viewer. The vendor offers a second bottle with an unlettered label.',['Flush the bad stuff out']),
 ('Market booth: a confident influencer adjusts a ring light while an affected traveler rests by the van with a bucket. Unmarked powder tubs fill the booth.',[]),
 ('Roadside product stall: a sick traveler holds an opened bottle while a seller points at an absurdly long blank receipt.',['Money-back guarantee'])],
 '05':[
 ('At a safe parking spot, a weary navigator compares a paper map with three road signs pointing in different directions. All signs are blank artwork.',['Liberty Way','Freedom Drive','Patriot Parkway']),
 ('Night in the stopped van: exhausted navigator studies a paper map beside a phone displaying empty interface shapes and a disconnected-signal symbol.',['Offline directions: subscription required']),
 ('A tired traveler studies a map whose route loops around three generic fuel pumps. An arrow-shaped blank billboard looms beyond the parked van.',['GO FURTHER'])],
 '06':[
 ('Garage counter at dawn: weary traveler waves a small white napkin toward a card reader. A mechanic presents a long blank invoice.',['Add a tip?']),
 ('Garage waiting area: exhausted traveler and mechanic face a long blank invoice that loops off the counter. Two empty label spaces identify mutually contradictory charges.',['Convenience fee','Mobile-service fee']),
 ('Inside the parked van before dawn: drained traveler holds a phone beside a warranty folder; closed garage shutters are visible outside.',['Your call is important to us'])],
 '07':[
 ('Van at night: a tired traveler is surrounded by bright, blank message bubbles from a former temp supervisor. A closed work apron lies in their bag.',['Work-life balance poll']),
 ('Close view of a traveler trying to sleep while a phone lights up repeatedly on the dashboard. Blank schedule blocks provide space for localized interface overlays.',['Available?']),
 ('Exhausted traveler in the stopped van looking at two blank message bubbles, the second crowding the first.',['No response needed','Did you see my message?'])],
 '08':[
 ('Book-event loading entrance: affected worker rests on the floor after a strain. A wealthy author steps around them toward a stage while stacks of unlettered books dwarf both people.',['I Did It All Myself','Overcoming Obstacles']),
 ('Pantry loading area: sore traveler beside heavy boxes; a suited sponsor poses with a conspicuously empty box as two assistants adjust his angle. No text or logos on donations.',[]),
 ('Food pantry doorway: a strained traveler leans against the van; sacks wait where a delivery truck would normally load. A coordinator holds a blank service-cancellation sheet.',['Leaner operations'])]
}
for f,rows in care_scenes.items():
    scenes('CARE-'+f,rows)
    for v in 'ABC':
        by[f'CARE-{f}-{v}']['scene']['state_variants']='Retain the same affected person and setting motif for continuing/critical states. Show worsening fatigue without graphic injury. Recovery changes posture; shelter uses a safe indoor cutaway. Fatal outcomes use an empty place, never an invented corpse or a restored traveler.'

ally_scenes={
 '01':[
 ('Illustrative cutaway of an outside contact photographing red hats beside a ring light. The traveling crew’s blank message screen occupies a small foreground corner.',['Affiliate link']),
 ('Illustrative cutaway of an outside contact in a rally queue beneath two blank entry boards; a wealthy attendee enters a separate carpeted lane.',['VIP','General admission']),
 ('Outside contact opening a generic welcome email beside a neat donation envelope. Blank message areas are reserved for overlays; no contact sits in the van.',['Donor Number 847,219'])],
 '02':[
 ('Outside contact in a kitchen with a laptop, blank dismissal letter and desk headset. The headset faces an empty office chair as the replacement.',[]),
 ('Outside contact surrounded by moving boxes and job listings represented by blank cards. A rent envelope sits more prominently than a decorative corporate trophy.',['Human capital released']),
 ('Outside contact at home holding a store discount voucher beside a dismissal envelope. A blank shop bag sits unused.',['Farewell discount'])],
 '03':[
 ('Outside contact changes from daytime work clothes into an evening apron, a long blank repair bill spread across the table.',[]),
 ('Outside contact pins extra work shifts beside a photo of their car and an elaborate unlettered payment calendar.',['Drive Free']),
 ('Outside contact’s retired broken vehicle part sits on a comfortable shelf; the contact shoulders a work bag for another shift.',[])],
 '04':[
 ('Outside contact runs a stall with two racks of opposing generic political colors. One cash drawer sits between them. No real logos or embedded slogans.',[]),
 ('Outside contact displays a luxurious blank hoodie in a glossy case beside an ordinary worker’s worn jacket.',['Anti-billionaire solidarity']),
 ('Outside contact packages a subscription product at a desk; a giant blank cancellation form unrolls toward the floor.',['Your voice matters','Call to cancel'])],
 '05':[
 ('Outside contact watches a dramatic unlettered video thumbnail while muting incoming messages. A blank phone interface allows separate block-state overlays.',['Think for yourself']),
 ('Outside contact suspiciously examines a picture of the crew’s ordinary van on a phone. The real van appears only in a separate visual cutaway, with no extra passengers.',[]),
 ('Outside contact solemnly presents an anonymous sheet with all writing areas blank, shielding the top where a source would go. A question bubble is reserved for overlay.',[])],
 '06':[
 ('Outside contact rests a supported ankle at home; a separate small cutaway shows a photo-op occupying an accessibility ramp. No graphic injury.',[]),
 ('Outside contact sits at home with feet elevated, protest placard leaning nearby. The placard is blank. A speaker still gestures on a distant screen.',[]),
 ('Outside contact rests a sore shoulder beside heavy portable loudspeakers. Their blank phone screen shows a new request through an overlay region.',[])]
}
for f,rows in ally_scenes.items():scenes('ALLY-'+f,rows)

cross_scenes={
 '01':[
 ('Checkpoint: officer checks a blank form while the ordinary van waits. A nearly empty wallet lies beside the travel folder.',['ECONOMY THRIVING']),
 ('Road checkpoint with a clearly visible lane looping back to another booth. The van waits at the first cone; blank boards indicate the complaint lane.',['Government complaints']),
 ('Guard holds a scanner upside down over an unlettered permit. A blank digital-efficiency poster hangs above a growing stack of printed error pages.',['Paperless future'])],
 '02C':[
 ('A guard holds the van beside one traffic cone while a luxury coach passes through an open adjacent lane. Blank signs and generic vehicle trim only.',['Same rules for everyone']),
 ('Guard cautiously holds the crew’s thick folder of bills by one corner while the van waits. The folder and forms have no embedded text.',[]),
 ('Long checkpoint queue with a large loudspeaker overhead. The guard’s weary expression matches the travelers; the announcement appears only as overlay.',['Please remain calm'])],
 '02D':[
 ('Obstructed bridge: a celebratory ribbon spans a gap in the approach. An official proudly holds a framed photograph of the clear opposite side; a worker gestures at the obstruction.',[]),
 ('Closed bridge gate with an oversized blank terminal beside it. The van is stopped outside; the guard points to the screen rather than the roadway.',['Crossing session expired']),
 ('An intact bridge is visible behind a guard staring at a map screen that omits it. Render only map shapes and bridge geometry, never embedded map labels.',[])],
 '03':[
 ('Bridge approach: workers hold an oversized blank commemorative plaque near a damaged patch while the van waits at a safe distance.',['Credit to our sponsors']),
 ('Officials pose with scissors beside unfinished bridge surfacing. A photographer points the camera away from the remaining roadwork. Blank banners only.',[]),
 ('An inaccessible feedback box sits visibly across a blocked bridge. A worker on the van’s side points toward it with an apologetic shrug.',['Public feedback'])]
}
for f,rows in cross_scenes.items():
    scenes('CROSS-'+f,rows)
    for v in 'ABC':by[f'CROSS-{f}-{v}']['scene']['state_variants']='Keep the vehicle on the near side until passage succeeds. Successful passage may show the far side. A diversion shows the actual branching road. Terminal art does not depict a fall, collision or death unless recorded.'

order_scenes={
 'SHUTDOWN':[
 ('Dashboard radio foreground; beyond it, a closed public-service door sits beside an operating bill-payment kiosk.',['Government closed']),
 ('Fictional announcement cutaway: essential workers wait beside a closed office, while an official holds an oversized blank closure notice.',[]),
 ('Fictional briefing cutaway: an official stands at six microphones under bright lights while a small closed-service door sits at the edge of frame.',[])],
 'MILITARIZE':[
 ('Road map on the stopped van’s dashboard with a detour barrier symbol over the route. No travelers detained or relocated.',[]),
 ('Fictional official in comfortable airport seating discusses restrictions while a small cutaway shows the ordinary van waiting at cones.',[]),
 ('Wide road scene with the van beside an absurd concentration of directional barriers, leaving only a narrow allowed route.',[])],
 'GAG':[
 ('Fictional book-committee hearing: a witness gingerly holds a closed, blank-covered book as though it might bite.',[]),
 ('Fictional official examining an open blank book through a protective face shield. The danger is their expression, not any harmful content.',[]),
 ('Committee members recoil from an open blank index page while a librarian waits with folded arms.',[])],
 'TARIFFS':[
 ('Shopkeeper swivels a card reader toward the crew after a radio announcement. A tiny globe beside the reader underlines who is actually being asked to pay.',[]),
 ('An almost empty grocery bag sits beneath a triumphant blank bulletin screen in a shop. The shopper’s wallet is visibly thin.',['Trade victory']),
 ('The crew’s worn wallet is ceremonially placed at the head of a miniature negotiation table on the van’s dashboard.',[])],
 'TAXCUTS':[
 ('Fictional education office closes its door while children’s blank workbooks remain on a table outside. No real school or identifiable child depicted.',[]),
 ('A large blank self-reliance notice replaces a public education help window; a parent and child stand before it.',['Be self-reliant']),
 ('Teacher in a second-job apron corrects a blank government announcement at a kitchen table beside a stack of unlettered workbooks.',[])],
 'DEREGULATE':[
 ('Van bonnet foreground, with a radio and an elaborate blank organizational chart overlay region. A harmless mechanical rattle is suggested through separate motion marks.',[]),
 ('Fictional department corridor piled with obsolete blank badges and blank signs; a small van cutaway waits for actual replacement parts.',[]),
 ('Stationary van with a loose-looking component silhouetted under its bonnet while a fictional official unveils a slimmer empty chart.',[])]
}
for f,rows in order_scenes.items():scenes('ORDER-'+f,rows)

repair_scenes={
 'TIRE':[
 ('Van pulled safely aside with a visibly failed tire; scraps of tread sit below a blank durability billboard.',['Built to last a lifetime']),
 ('Safe roadside view of a flat wheel and abandoned tread, with the crew consulting their spares rather than driving.',[]),
 ('Flat tire foreground; a crew member holds a spare while another compares the actual displayed repair choices. No invented subscription interface.',[])],
 'BATTERY':[
 ('Parked van with dark dashboard; driver’s hand rests at the ignition and a spare battery is visible only if carried.',[]),
 ('Dark dashboard beside a lit phone showing an unlettered luxury-car advert; the ordinary stranded van frames the contrast.',[]),
 ('Crew beside the open bonnet of the stopped van, one person gesturing a hopeful restart while the dead battery remains the focus.',[])],
 'ALTERNATOR':[
 ('Open bonnet close-up: failed alternator and battery are visually distinct, with the strained battery emphasized through safe metaphorical motion marks.',[]),
 ('Crew member pats the dashboard while another points toward the broken charging system. The hopeful gesture changes nothing.',[]),
 ('Phone cables tangled beside a dead vehicle dashboard; charging needs range from tiny devices to the entire immobile van.',[])],
 'FUELPUMP':[
 ('Simple cutaway silhouette of the stopped van showing fuel in the tank and an interrupted path at the pump. No labels embedded.',[]),
 ('Parked van dashboard fuel gauge contrasts with a motionless engine in a simple inset. All gauge numbers must be overlays.',[]),
 ('Small failed pump in a mechanic’s palm beside the large stranded van. Emphasize the disproportion between component and consequence.',[])]
}
for f,rows in repair_scenes.items():
    scenes('REPAIR-'+f,rows)
    for v in 'ABC':by[f'REPAIR-{f}-{v}']['scene']['state_variants']='Before success, show the van stopped and the failed part. After success, show the actually acquired replacement fitted. In-town purchase uses a shop; roadside purchase uses delivery; radio-work may show an ordinary agreed job. Never show cash paid, supplies consumed or a spare present unless the action/state supports it.'

activity_scenes={
 'FORAGE':[
 ('Local guide identifies safe gathered food beside the stopped van; a boutique basket displayed across the road carries a blank fancy tag.',[]),
 ('Guide holds a safe food plant while a traveler lowers an unrelated interesting leaf without eating it. Do not show identifiable toxic species or instructional harvesting detail.',[]),
 ('A guide gestures toward the surrounding landscape while a visitor searches a plain gathered-food basket for a label.',[])],
 'GLEAN':[
 ('Farm with permission to glean; crew lifts crates of cosmetically irregular produce, one oddly forked carrot the visual focal point.',[]),
 ('Crew gathers potatoes of wildly different shapes beside identical empty supermarket-style display slots. No retailer branding.',[]),
 ('Farmer hands the crew crates while a consultant holds a blank diagram and remains conspicuously far from the lifting.',[])],
 'FOODWORK':[
 ('Town pantry: crew sorts donations while a sponsor rotates blank labels toward a camera. A prepared food parcel waits separately.',[]),
 ('Pantry volunteers stock shelves behind a board meeting arranged around a conspicuously full food table.',[]),
 ('Pantry cook points to a steaming pot while a consultant presents blank reward tokens that plainly are not food.',[])],
 'CASHWORK':[
 ('Town appliance-shop loading bay with an explicit blank temporary-work board, crew considering the offer beside a van.',['Three-hour shift · $18']),
 ('Shop loading bay stacked with heavy blank-covered motivational books; the crew considers a clearly separate work offer.',['Three-hour shift · $18','I Did It All Myself']),
 ('Office-supply loading bay: elaborate executive chairs with plush cushions tower over the workers’ simple folding seats.',['Three-hour unloading shift · $18'])],
 'BARTERTIRE':[
 ('Town exchange table: a resident offers one spare tire opposite the crew’s supply packs. Keep both sides visible until the trade succeeds.',[]),
 ('Resident demonstrates a complete spare tire on a simple table, contrasting with a nearby blank premium-service display.',[]),
 ('Community exchange table viewed at low angle while an entrepreneur searches beneath it for a hidden ownership mechanism.',[])],
 'BARTERBATTERY':[
 ('Exchange table with a practical spare battery opposite a glossy blank investor presentation board.',[]),
 ('Resident offers a sturdy battery; a blank luxury-property billboard behind the stall visually dwarfs the useful object.',[]),
 ('Battery on the exchange table with an innocent charging cable curling beside it; no payment card or account screen is pictured.',[])],
 'BARTERSUPPLIES':[
 ('Crew compares a spare tire with a genuine basket of food offered at a community exchange. Nobody attempts to eat rubber.',[]),
 ('Exchange stall: a tire and actual provisions sit on opposite sides of the table, with a nearly empty cooler in the van.',[]),
 ('Spare-parts box full of rubber contrasts with the nearly empty pantry box; the community exchange offers a sensible reversal.',[])],
 'REST':[
 ('Parked van at a permitted rest spot: traveler turns off a blank podcast screen while present companions settle quietly.',[]),
 ('Crew at rest beside the van; an eager traveler reaches toward a closed notebook that someone gently moves away.',[]),
 ('Wide quiet rest spot: active travelers lounge in ordinary safe positions and the van remains stationary. No productivity counters or invented amenities.',[])]
}
for f,rows in activity_scenes.items():scenes('ACT-'+f,rows)

condition_scenes={
 'CLEAR':[
 'Clear road and open sky through the van’s windscreen; simple daylight without a smiling mascot or institutional logo.',
 'Clouds part over the route, with an empty foreground patch where a publicity podium might have been.',
 'Broad bright horizon framed by the ordinary van cabin; sunlight reaches every visible occupant equally.'
 ],
 'STORM':[
 'Van traveling in rain only when the game records travel; a closed roadside car wash sits beyond the verge with a blank sign.',
 'Heavy rain turns picnic-table hollows into tiny pools beside a stopped van, with no actual flooding claim beyond the scene.',
 'Rain beads on the roof and windscreen; visible occupants react to the drumming without a readable sound effect embedded.'
 ],
 'HEAT':[
 'Harsh sun on the van and softened-looking seat cushions; heat shimmer outside, no injured traveler unless exposure has actually begun.',
 ('Sun-baked road beside a blank luxury-resort billboard depicting shade and water, beyond the ordinary van.',['A refreshing escape']),
 'Traveler cautiously lifts an elbow from a sun-heated dashboard while the van is safely stopped.'
 ],
 'COLD':[
 'Visible breath in the van and present travelers leaning toward warmth; avoid assigning distress to absent companions.',
 'A blanket is passed among present travelers inside the van, becoming the bright focal object in a cold palette.',
 'Windshield frost and visible breath contrast with the van’s modest heater vents; no temperature text embedded.'
 ],
 'SMOKE':[
 ('Gray haze dulls a road behind a blank optimistic billboard. The van occupies a safe position appropriate to current travel state.',['Fresh opportunities']),
 ('A blank scenic-route sign points toward a view obscured by haze. No actual named landmark is fabricated.',['Scenic route']),
 'Smoke-softened landscape beside a generic barbecue advert represented by an unlettered food image. No new wildfire event is depicted.'
 ],
 'ILLNESS':[
 'Quiet cabin vignette with tissues and a closed travel bag; no extra passenger or specific named illness is invented.',
 'Stationary van with a resting active traveler if one is identified by state; companions’ posture is practical rather than cheerful reassurance.',
 'An unused seat corner holds tissues and water beside the crew’s luggage. The illness is represented through props, not an extra character.'
 ],
 'HUNGER':[
 'An empty food container sits between active travelers; a thought-bubble space may show a sandwich as an illustration, clearly imagined.',
 'Close view inside a near-empty pantry box in the van, with an active traveler checking the corners.',
 ('Empty food box below a blank motivational roadside poster; no death or collapse drawn automatically.',['Success begins with a positive mindset'])
 ],
 'HEATEXPOSURE':[
 'Van pulled aside under punishing heat, affected active traveler visibly fatigued if state identifies one; no comedic collapse or graphic injury.',
 'Heat-stressed van foreground against a luxury-resort picture full of shade, contrasting practical need and advertised comfort.',
 'Bright sun and a spare, strained cabin composition; keep the actual health warning as a UI overlay rather than embedded numbers.'
 ],
 'COLDEXPOSURE':[
 'Van cabin in a cold palette; visible active travelers have restrained signs of discomfort, and only carried protection is shown.',
 'Cold light enters through a door seam while an active traveler pulls existing clothing closer; no coats invented if none are carried.',
 'An unnumbered thermometer shape occupies the foreground of the cold van cabin. Actual temperature/health information belongs in overlays.'
 ]
}
for f,rows in condition_scenes.items():scenes('COND-'+f,rows)

opening_scenes={
 'JOURNALIST':[
 'Seattle departure: player notebook balanced on a cooler while five companions arrange luggage in the van; loose unlettered receipts dominate the notebook.',
 'Seattle departure parking area: all six travelers beside a stubborn cooler lid, with the journalist’s notebook tucked under an arm.',
 'Seattle origin: six people check supplies beside the van while the journalist holds an unlettered expense envelope that is visibly empty.'
 ],
 'ORGANIZER':[
 'Portland departure: six travelers negotiate luggage around an open van door, one overlarge bag wedged at the hinge.',
 'Portland origin: six travelers each protect a personal bag while the organizer points at the same awkward luggage corner.',
 'Portland origin: six raised hands give way to puzzled expressions over a sealed food tin and a missing can-opener space.'
 ],
 'WHISTLEBLOWER':[
 'San Francisco departure: player secures duplicate unlettered evidence folders while companions inspect a modest spare-parts box.',
 'San Francisco origin: a companion searches luggage for food while the whistleblower protects a plain evidence case; all six travelers present.',
 'San Francisco departure: player closes a blank procedural binder while five companions load a van with ordinary bags, not official equipment.'
 ],
 'LOBBYIST':[
 'Los Angeles origin: six ordinary travelers load a visibly worn van; the lobbyist’s battered portfolio contrasts with glossy office buildings in the distance.',
 'Los Angeles departure: player compares a contacts notebook with jumper cables offered by one of five companions. No private driver or aides.',
 'Los Angeles origin: lobbyist and five companions gather beside the van with a plain community petition folder and modest travel provisions.'
 ],
 'STAFFER':[
 'Sacramento departure: staffer untangles a charger while five companions pack the van. The papers are plain and no official powers are implied.',
 'Sacramento origin: six travelers divide papers and luggage beside the open van; an empty folding chair evokes the meeting they are replacing.',
 'Sacramento departure: player holds a blank checklist while companions raise food items like urgent amendments.'
 ],
 'SATIRIST':[
 'San Diego departure: satirist compares an unlettered joke notebook with a blank government-announcement screen while five companions load the van.',
 'San Diego origin: player gestures comically beside the ordinary van as a companion quietly points at the fuel budget wallet.',
 'San Diego departure: six travelers organize supplies, with the satirist holding a notebook and the others giving a practiced skeptical look.'
 ]
}
for f,rows in opening_scenes.items():scenes('OPEN-'+f,rows)

hearing_scenes={
 'C':[
 'D.C. hearing: a long-winded chair at a large desk and the actual arriving travelers at a small public microphone. All nameplates blank.',
 'Hearing chair stares at a clock while the public microphone faces the arriving travelers; clock numerals and timing appear only as overlays.',
 'Hearing officials look mildly surprised at ordinary travelers entering the public space; empty chairs remain empty according to actual roster.'
 ],
 'D':[
 ('Hearing official’s aide turns a public microphone away while a blank welcome screen dominates the wall.',['Public participation welcome']),
 'A clerk holds an already prepared blank summary sheet while the actual arriving travelers take their hearing place.',
 'Huge portrait of an attentive listener hangs behind distracted hearing officials; the portrait’s eyes face the public microphone.'
 ]
}
for f,rows in hearing_scenes.items():
    scenes('HEARING-'+f,rows)
    for v in 'ABC':by[f'HEARING-{f}-{v}']['scene']['state_variants']='Three procedural camera/paper-position changes may reuse this scene; do not show three ballots. One vote tableau appears only after the rounds are endured. Show no winning or losing tally until actual resolution.'

# Endings require distinct visual motifs, but must not manufacture a roster,
# corpse, hearing, location, crash or purchase absent from the journey record.
ending_scenes={
 'VICTORY':[
 'D.C. hearing chair claims the foreground while actual surviving arrivals share restrained relief behind the microphone; no new law-signing ceremony.',
 'Close view of the crew’s unlettered evidence folder being placed in the official hearing record beside the actual successful vote overlay.',
 'Officials exchange uneasy looks as actual surviving arrivals gather their belongings after the successful vote.'
 ],
 'VOTEFAIL':[
 'Hearing chair makes a polished thank-you gesture while the public microphone dims; show only travelers who arrived.',
 'Crew evidence folder remains on the hearing table after the failed vote; the losing result is a separate overlay, never printed into art.',
 'An official hand switches off the microphone after the failed vote while actual arriving travelers remain in the background.'
 ],
 'SANITY':[
 'Closed travel notebook on the stopped van’s dashboard; show the actual final location and roster, with no caricature of mental illness.',
 'Stopped van beside a blank motivational billboard at the actual compatible final location; present travelers appear tired, not mocked.',
 'An active traveler’s hands set down the case folder in the stationary van. Do not depict a named person if the record does not identify one.'
 ],
 'DESTROYED':[
 'Actual destroyed-vehicle state shown in a quiet wide shot; no flames, collision participants or casualties unless recorded.',
 'Close view of the irrecoverably damaged van’s unlettered dashboard, with the road beyond out of focus.',
 'Destroyed van foreground with a generic replacement-car advert on a blank sign only where a roadside setting fits the record.'
 ],
 'COLD':[
 'Stopped van under a cold sky, focusing on frost and the road rather than unrecorded people or bodies.',
 'A plain travel slogan card lies beside a cold window; keep its wording as a separate optional overlay.',
 'Van roof and frost-lined windows against the actual compatible cold setting, no miraculous shelter shown.'
 ],
 'HEAT':[
 'Stopped van in harsh light and heat haze; no invented deceased traveler or empty water inventory.',
 'Heat-shimmered road beside a generic sunshine-holiday picture only if compatible with the recorded location.',
 'Unnumbered temperature display reflected in the windscreen of the stopped van; final costs belong to overlays.'
 ],
 'COLLAPSE-HUNGER':[
 'Empty food box beneath the evidence folder inside the stationary van, with no invented fatality.',
 'Close view of an empty pantry container and the actual crew’s packed belongings; no food is added for decoration.',
 'The road seen through the van’s windscreen behind an empty lunch container, illustrating the unmet practical need.'
 ],
 'COLLAPSE-VEHICLE':[
 'Van stopped at its recorded final location, bonnet closed or open according to recorded breakdown; do not depict total destruction by default.',
 'Quiet wide shot of the actual stationary van with its route stretching beyond; no assumed tow vehicle.',
 'Interior view of the same motionless roadside view through the windscreen, emphasizing a journey that has ceased.'
 ],
 'COLLAPSE-WEATHER':[
 'Actual final weather around the stopped van, using the recorded condition rather than defaulting to storm.',
 'Unlettered itinerary on the dashboard with the actual adverse weather beyond the glass.',
 'Route map remains dry inside the cabin while actual recorded weather affects the van outside.'
 ],
 'COLLAPSE-BREAKDOWN':[
 'Stopped van with the unresolved failed part highlighted only if recorded; no repair tools magically completing the job.',
 'The actual spare-parts inventory beside the stationary van; show an empty relevant slot only if no spare was carried.',
 'Unlettered warning icon in the motionless van’s dashboard; the warning and actual part name remain separate overlays.'
 ],
 'COLLAPSE-DISEASE':[
 'Quiet stationary van with an evidence folder and a folded blanket; show no body or character death unless recorded.',
 'The journey’s unlettered papers lie beside actual medical/supply inventory, with no invented miracle product or cure.',
 'Soft, restrained view of the stopped van from outside; use actual survivor roster without depicting anyone recovered.'
 ],
 'COLLAPSE-CROSSING':[
 'Actual terminal crossing obstruction and the van on the near side; no fall or crash introduced.',
 'The crew’s case folder sits against the windscreen with the recorded blocked crossing beyond.',
 'Road-direction sign with blank lettering area points beyond the actual terminal crossing; van remains on its recorded side.'
 ],
 'COLLAPSE-PANIC':[
 'A travel itinerary is set down inside the stopped van; subdued composition, no grotesque expressions or medical symbols.',
 'The recorded final location seen through a quiet van cabin; protect the dignity of any actual present travelers.',
 'The road continues outside the stationary windscreen while the crew’s route notebook rests closed inside.'
 ],
 'INCOMPLETE':[
 'Packed evidence folder inside the actual saved van at the last recorded stop; no D.C. skyline unless reached.',
 'The incomplete route displayed through unlabeled map shapes and a last-position marker; all route names are overlays.',
 'Wide view of the last recorded van location in neutral light; no outcome implied by celebrations or memorial objects.'
 ],
 'FALLBACK':[
 'Unlettered journey notebook with a visibly incomplete final page; abstract neutral background instead of an invented location.',
 'A plain case folder and blank result panel, with no winners, casualties or city skyline implied.',
 'Neutral map texture beside the preserved unlettered journey record; unknown areas left visually unresolved, not filled by invented scenery.'
 ]
}
for f,rows in ending_scenes.items():scenes('END-'+f,rows)

assert all(u.get('scene') for u in units)
(ROOT/'support.json').write_text(json.dumps(units,ensure_ascii=False,indent=2)+'\n')
