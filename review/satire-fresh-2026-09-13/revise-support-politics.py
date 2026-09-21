import json
from pathlib import Path

ROOT=Path(__file__).resolve().parent
units=json.loads((ROOT/'support.json').read_text())
by={u['id']:u for u in units}
sources={s['id']:s for s in json.loads((ROOT/'political-sourcebook.json').read_text())['sources']}
sources.update({
 'dei':{'subject':'Trump anti-DEI actions','fact_and_limits':'January 2025 executive actions targeted federal DEI programs and contractor requirements; fictional employers and incidents are invented.','url':'https://www.whitehouse.gov/presidential-actions/2025/01/ending-illegal-discrimination-and-restoring-merit-based-opportunity/'},
 'energy':{'subject':'Trump energy and climate rollback','fact_and_limits':'January 2025 energy order promotes fossil fuel development and reverses specified climate measures. A particular fictional weather event is not attributed to one order.','url':'https://www.whitehouse.gov/presidential-actions/2025/01/unleashing-american-energy/'},
 'sharpie':{'subject':'Sharpiegate','fact_and_limits':'The 2019 Hurricane Dorian map controversy involved an altered forecast map displayed by Trump. The fictional unloading boss imitates that tactic.','url':'https://www.oig.doc.gov/OIGPublications/OIG-20-032-I.pdf'}
})

def stamp(u,topic,scene,overlays=()):
 s=sources[topic]
 u['political_basis']={'subject':s['subject'],'real_hook':s['fact_and_limits'],'source_url':s['url'],'fictionalization':'Original fictional people, dialogue, scene and prices; not a report of this event occurring.'}
 u['evidence_status']='Fictional political satire with a documented real-world hook; mechanics retained'
 u['sources']=[{'title':s['subject'],'url':s['url'],'qualification':s['fact_and_limits']}]
 u['illustration']=scene
 u['scene']['description']=scene
 u['scene']['overlays']=[{'text':v,'placement':'Separate editable overlay on the indicated blank prop or beside it','display_condition':'Only with this variant; never baked into the image'} for v in overlays]
 u['status']='political rewrite for user review'

def care(id,title,topic,setup,continuing,helped,scene,overlays=()):
 u=by[id];u.update(title=title,setup=setup,continuing=continuing)
 u['choices'][0].update(label='Provide care',outcome=helped)
 u['choices'][1].update(label='Leave in shelter',outcome='{name} stays in shelter and leaves the traveling crew alive. The incident is cleared.')
 u['choices'][2].update(label='Press on',outcome='You continue without care. {name} remains unwell, and the incident can worsen.')
 u['outcomes'].update(helped=helped,deferred=continuing,sheltered='{name} remains alive in shelter and has left the expedition.')
 stamp(u,topic,scene,overlays)

care('CARE-01-A','Alive With Goodness','raw_milk',by['CARE-01-A']['setup'],
 '{name} is still ill from the MAHA raw milk. The vendor has promoted the stomach cramps to a cleansing process.',
 '{name} recovers with care. The vendor asks for a testimonial. You offer the bathroom key.',
 'The affected traveler heads toward a portable toilet while a raw-milk vendor proudly presents an unlettered bottle. A small knife-shaped shadow falls from the bottle; no violence.',('Make America Healthy Again','Alive with goodness'))
care('CARE-01-B','The Freedom Fridge','raw_milk',
 '{name} drinks raw milk at a MAHA stall opposing government food rules. They start vomiting. The vendor checks the fridge: unplugged, in protest.',
 '{name} is still sick. The vendor says refrigeration is a slippery slope to federal control of cheese.',
 '{name} recovers. The vendor plugs in the fridge and asks you to keep his surrender quiet.',
 'Unplugged mini-fridge behind a MAHA vendor; affected traveler leaning near a wash station. The unplugged cable is the focal gag.',('MAHA','Keep government out of my fridge'))
care('CARE-01-C','A Second Opinion','raw_milk',
 'A MAHA vendor tells {name} raw milk builds immunity. Now they’re ill. He recommends a second bottle. Apparently the immune system needs a rematch.',
 '{name} remains ill. The unopened second bottle sits in the van like a very small hostile nation.',
 'Care helps {name} recover. You decline the rematch. The bottle keeps its undefeated record.',
 'Vendor offers another blank milk bottle while the ill traveler sits beside the van. Keep the second bottle sealed and conspicuously separate.',('Build natural immunity'))
care('CARE-02-A','Executive Forecast','sharpie',by['CARE-02-A']['setup'],
 '{name} is still dehydrated. The Sharpie has made a full recovery and returned to management.',
 'Care restores {name}. The sun continues operating outside the boss’s chain of command.',
 'Dehydrated traveler rests beside unloaded crates. A boss stands with a marker beside a blank warning board; heat shimmers beyond his small patch of shade.',('Heat warning','Cancelled','#sharpiegate'))
care('CARE-02-B','Local Control','heat_preemption',
 'During an unloading shift, {name} becomes dehydrated. The boss praises Florida’s ban on local heat-break rules. “Government shouldn’t tell us what to do,” he says, blocking the water.',
 '{name} remains dehydrated. The boss’s freedom to manage has developed an impressive thirst.',
 '{name} recovers with care. For several revolutionary minutes, thirst gets a vote.',
 'A boss blocks a shaded water cooler while the affected traveler sits among delivery crates. His own full glass has condensation.',('Local control'))
care('CARE-02-C','Drill, Baby, Drink','energy',
 '{name} becomes dehydrated unloading a “Drill, baby, drill” rally stage. The organizer says heat is natural. So is the ice in his private cooler.',
 '{name} is still dehydrated. The organizer has ordered more ice for his argument.',
 'Care helps {name} recover. The organizer guards the cooler like the last remaining oil reserve.',
 'Rally-stage crates on hot asphalt, traveler in shade, organizer leaning protectively over a private ice cooler. No baked-in rally lettering.',('Drill, baby, drill'))
care('CARE-03-A','Proof of Labor','benefit_work',
 'After a temp shift, {name} stays up proving those hours count under Congress’s benefits work rules. By dawn, the only unpaid work is proving they work.',
 '{name} is exhausted from the benefits paperwork. Their shift is documented; the night spent proving it remains volunteer work.',
 'Care helps {name} recover. You put the forms away before they develop a work requirement of their own.',
 'Exhausted traveler at a late-night diner table with a small mountain of blank forms and a crumpled work glove.',('Benefits work requirements'))
care('CARE-03-B','Efficient Exhaustion','doge_parks',
 '{name} loses sleep helping a laid-off DOGE-cut worker assemble job applications. The government called them redundant. Every employer wants the same information entered again.',
 '{name} is still exhausted. The last application has asked the worker to upload the résumé it just deleted.',
 'Care restores {name}. For once, something has been saved without creating a new account.',
 'Traveler nods over a laptop beside a former public worker holding a plain résumé; repetitive empty form boxes remain blank for overlays.',('Position eliminated'))
care('CARE-03-C','The Merit Test','dei',
 '{name} loses a night applying for a government contractor’s temp work. The firm denounces DEI. Its application asks whether the owner’s nephew referred them. He’s twelve.',
 '{name} remains exhausted. The nephew’s referral box is still the shortest route through the merit system.',
 '{name} recovers. The nephew is unavailable for comment; it’s a school night.',
 'Exhausted adult traveler faces a blank application while a contractor’s family portrait prominently displays a child in an oversized executive chair.',('Merit-based hiring'))
care('CARE-04-A','Drain the Sample Bucket','raw_milk',
 '{name} tries a MAHA tonic promising to “drain the swamp” from their body. They vomit into the sample bucket. The swamp has acquired a gift shop.',
 '{name} is still sick from the tonic. The vendor says discomfort proves it works. Refunds apparently prove nothing.',
 '{name} recovers with care. The vendor offers a loyalty card. You suggest a mop.',
 'Wellness stall with unlettered tonic bottles and a bucket beneath the counter; affected traveler turned away, vendor holding a blank loyalty card.',('MAHA','Drain the swamp'))
care('CARE-04-B','The Unelected Stomach','raw_milk',
 '{name} becomes sick after a MAHA supplement sample. The vendor blames unelected health experts. Nobody recalls voting for the powder, either.',
 '{name} remains ill. The vendor has demanded an investigation into the stomach’s political affiliations.',
 'Care helps {name} recover. Their stomach accepts the result without demanding a recount.',
 'Ill traveler beside a wellness display; vendor scrutinizes an unmarked powder sachet through a magnifying glass as though it were a suspicious ballot.',('MAHA supplement'))
care('CARE-04-C','Personal Responsibility, Now in Mango','raw_milk',
 '{name} tries a MAHA cleanse sold as freedom from the FDA. Now they’re sick. The vendor celebrates personal responsibility by pointing at the customer.',
 '{name} is still ill. The vendor’s cure for government overreach has reached the entire stomach.',
 '{name} recovers. You ask the vendor to take some personal responsibility. He suddenly has a lawyer.',
 'Vendor gestures away from an unlettered supplement bottle toward the affected traveler; a second stall worker quietly flips a blank guarantee card.',('Freedom from the FDA'))
care('CARE-05-A','Patriotically Lost','gulf',
 '{name} loses sleep reconciling maps after Trump’s Gulf and mountain renaming. The van still needs the same turn. Geography has failed its loyalty test.',
 '{name} is exhausted. Two maps disagree about the name of something that has not moved.',
 'Care restores {name}. You follow the road, which has managed to stay in office.',
 'Traveler slumps between two otherwise identical blank maps; the route lines match precisely while removable label boxes sit in different piles.',('Gulf of Mexico','Gulf of America'))
care('CARE-05-B','The Mountain Formerly Known','gulf',
 '{name} stays up matching a renamed federal landmark to an older route guide. A mountain has two names. Somehow the human has to do the paperwork.',
 '{name} is still exhausted. The mountain has declined to provide a preferred spelling.',
 '{name} recovers. You agree to recognize the mountain by the enormous mountain underneath the name.',
 'Exhausted traveler compares an old folded guide with a blank digital map, both showing the same unmistakable mountain silhouette.',('Denali','Mount McKinley'))
care('CARE-05-C','Rebranding Detour','gulf',
 '{name} loses sleep updating the route after another federal renaming announcement. The app celebrates American greatness by sending you past the same gas station twice.',
 '{name} remains exhausted. The renamed destination is secure; the route to it is doing a victory lap.',
 'Care helps {name} recover. You disable the celebratory update and keep the actual road.',
 'A traveler studies a route loop on a blank phone beside the van; the same distinctive gas-station statue appears through both windows.',('Names restored. Route recalculating.'))
care('CARE-06-A','A Very Small Trade War','tariffs',
 '{name} spends all night disputing Trump’s tariff on a van part. By dawn, they’re waving a white napkin at the card reader. It asks for a tip.',
 '{name} is exhausted from the tariff dispute. China has not sent a relief driver.',
 '{name} recovers with care. You declare a ceasefire with the receipt printer.',
 'Exhausted traveler at a garage counter holding a white napkin near a blank card reader; an unlettered parts box sits between them and the cashier.',('Tariff','Add a tip?'))
care('CARE-06-B','America First in Line','tariffs',
 '{name} loses sleep tracing a “foreign-paid” tariff through the repair quote. Every arrow leads back to your wallet. America first, apparently.',
 '{name} remains exhausted. You have traced international trade to the lint in the van’s coin tray.',
 'Care helps {name} recover. The coin tray is formally cleared of responsibility for China.',
 'Traveler draws arrows between an unlettered invoice and a nearly empty wallet; every arrow returns to the wallet.',('Foreign countries pay'))
care('CARE-06-C','The Tariff Hotline','tariffs',
 '{name} stays up calling about a tariff on the van’s replacement part. The hotline promises to connect them with whoever pays it. Their own phone starts ringing.',
 '{name} is still exhausted. The hold music has survived three attempts to make China answer.',
 '{name} recovers. You let the next call go to voicemail. International relations can leave a message.',
 'Sleep-deprived traveler holds a phone while another handset in the van visibly vibrates beside a blank parts invoice.',('Tariff assistance'))
care('CARE-07-A','Five Things Before Breakfast','doge_parks',
 '{name} loses sleep helping a DOGE-cut friend list their work for an efficiency review. They count five tasks. The person requesting the list has created a sixth.',
 '{name} remains exhausted. The review has classified time spent answering the review as administrative waste.',
 'Care restores {name}. You record one completed task: keeping an actual person functioning.',
 'Traveler nods over a plain laptop while five physical work objects—keys, gloves, brush, map and radio—line the table.',('List your accomplishments'))
care('CARE-07-B','Secure Group Chat','signal',
 'After a loading job, {name} spends the night sorting the boss’s endless group messages. He copied the Pentagon’s bombing-plan chat. Nobody is sure who invited the pizza delivery driver.',
 '{name} is still exhausted. The delivery driver knows tomorrow’s rota and would prefer to know who ordered garlic bread.',
 '{name} recovers. You mute the chat before the boss authorizes a second shift by emoji.',
 'Traveler in the van faces an overflowing blank chat screen; nearby delivery driver holds a pizza box and a phone with the same notification shapes.',('Secure management chat'))
care('CARE-07-C','The Need-to-Know Shift','signal',
 '{name} loses sleep after a temp boss copies the Signal bombing-plan scandal: rota, payroll, everyone’s phone number, one chat. He says it’s secure because the group has a padlock emoji.',
 '{name} remains exhausted. The padlock emoji has not prevented the boss’s uncle from replying to all.',
 'Care helps {name} recover. You remove the notifications. The emoji remains bravely on duty.',
 'Sleep-deprived traveler turns down a blank phone screen showing only an oversized padlock icon; generic notification tiles pile behind it.',('Secure team chat'))
care('CARE-08-A','Self-Made, Staff Carried','campaign_money',
 'At a paid unloading job for a billionaire’s political fundraiser, {name} injures their back moving his memoir, I Did It All Myself. The donor steps over them to reach the podium.',
 '{name} remains injured. The donor’s speech about self-reliance has required another pair of hands to turn the pages.',
 'Care helps {name} recover. You leave the remaining memoirs to pull themselves up by their dust jackets.',
 'Injured traveler beside crates of blank memoir covers; an affluent fundraiser guest steps around the crates toward a podium. Other crew remain by the van.',('I Did It All Myself'))
care('CARE-08-B','The Donation Photo','snap',
 '{name} strains their back moving food-pantry donations as the radio covers Congress’s SNAP cuts. A politician arrives to lift an empty box. Finally, a balanced workload.',
 '{name} is still injured. The politician’s empty box has appeared in three photographs without requiring medical attention.',
 '{name} recovers with care. The empty box is unavailable to help; it has a press conference.',
 'Affected traveler sits beside heavy donated groceries; fictional politician comfortably lifts an obviously empty open-bottomed box for a photographer.',('Fighting hunger'))
care('CARE-08-C','The Smaller Delivery Service','snap',
 'At a food pantry discussing Congress’s food-aid cuts, {name} helps haul sacks and strains their back. The volunteer calls it trickle-down. Nothing appears to be trickling; it’s all carrying.',
 '{name} remains injured. The pantry still has sacks to move and no billionaire has dripped out of the ceiling.',
 'Care helps {name} recover. The next sack can wait for the invisible hand.',
 'Strained traveler beside sacks and an unused hand truck; a volunteer looks up at a completely ordinary dry ceiling.',('Food-aid cuts'))

ALLY=[
('01-A','The Red Hat Referral','campaign_money','Your outside contact {name} quits helping and joins a MAGA merchandise drive. Their goodbye includes a discount code. Political conversion now has affiliate tracking.','Outside contact photographs red hats beside a ring light while a blank goodbye message appears in an inset.',['MAGA','Affiliate link']),
('01-B','America First, Contact Last','tariffs','Your contact {name} drops the trip to sell America First hats. They ask you to reimburse the tariff on the shipment. China has apparently nominated friends and family.','Outside contact stands beside plain cartons of hats and a customs bill; the traveling crew sees the request on a blank phone.',['America First']),
('01-C','The Donor Conversion','campaign_money','Your contact {name} stops supporting the trip and takes a job courting super-PAC donors. They still believe in your voice. They’ve simply discovered the volume knob.','Outside contact adjusts an enormous donor-event microphone while a small, unconnected microphone sits aside.',['Independent expenditures']),
('02-A','Efficiency Has Arrived','doge_parks','DOGE cuts cost your outside contact {name} their job. They withdraw to find work. Their last assignment was to document the value of documenting their value.','Outside contact carries a box of desk belongings past an empty public-service counter; blank forms overflow the box.',['Position eliminated']),
('02-B','Nobody to Answer','doge_parks','Your contact {name} loses a public-service job in the DOGE cuts and must stop helping. The agency’s goodbye says to call with questions. They were the person who answered.','Outside contact holds a disconnected office headset beside a cardboard box; the same extension appears only as an editable overlay.',['Call us with questions']),
('02-C','The Merit Vacancy','dei','Your contact {name} loses their job as a contractor scraps its DEI work and must withdraw. The boss announces a new era of merit. His son announces that lunch is ready.','Outside contact packs an office desk while the owner’s adult son occupies a larger desk behind them.',['Merit-based opportunity']),
('03-A','America Pays First','tariffs','Your contact {name} takes extra shifts to cover tariff-inflated repair costs and can’t keep helping. They’ve found out which foreign country pays. It’s the one printed on their address.','Outside contact checks a garage invoice beside a work apron and an overnight shift alarm.',['Tariff surcharge']),
('03-B','Trickle-Down Overtime','health_budget','Your contact {name} needs extra shifts for medical bills and withdraws. Congress is selling health-budget cuts as savings. They would like to know where the savings get deposited.','Outside contact puts a medical bill under a second job’s timecard; no dollar figures embedded.',['Savings']),
('03-C','The Food Budget','snap','Your contact {name} takes another shift to cover groceries and stops helping. They watched Congress sell food-aid cuts as independence. They are now independently working Saturday.','Outside contact pulls on a second work apron beside a sparsely filled grocery bag.',['Saturday shift']),
('04-A','Bipartisan Checkout','campaign_money','Your contact {name} leaves to sell merchandise to rival political rallies. They’ve solved the two-party problem with two card readers.','Outside contact sets up rival-color merchandise tables, with an identical blank card reader on each.',['Bipartisan checkout']),
('04-B','The Patriotic Returns Policy','gulf','Your contact {name} quits helping to sell Gulf of America souvenirs. They keep the old Gulf of Mexico stock underneath. The ocean has yet to request a refund.','Outside contact flips reversible blank souvenir tags on identical coastal trinkets.',['Gulf of America','Gulf of Mexico']),
('04-C','Emergency Appeal','campaign_money','Your contact {name} leaves to write political fundraising emails. Every message says democracy ends at midnight. Their contract renews monthly.','Outside contact works at a laptop beside a monthly calendar and a theatrical countdown clock, all text as overlays.',['Democracy ends at midnight']),
('05-A','The Deep State Group Chat','signal','Your contact {name} sends a video claiming journalists invented the Signal scandal, then blocks the crew. The explanation is now more secure than the bombing chat.','Outside contact closes a blank group-chat screen with a physical padlock lying uselessly beside the phone.',['Contact blocked']),
('05-B','The Classified Stomach','raw_milk','Your contact {name} decides your criticism of MAHA milk proves you work for the FDA and cuts contact. Your new federal career comes with no salary and several stomach cramps.','Outside contact shields blank milk bottles from a phone displaying an unanswered crew message.',['FDA operative?']),
('05-C','The Tariff Awakening','tariffs','Your contact {name} says tariffs cannot raise American prices because Trump said China pays. They block you after the crew sends a receipt. Evidence is apparently an import.','Outside contact pushes an unlettered receipt away while blocking a message on a blank phone.',['Import rejected']),
('06-A','The Protest Barrier','campaign_money','Your contact {name} twists an ankle outside a political fundraiser and withdraws. The accessible entrance was reserved for donors. Equal access was available at the next sponsorship level.','Outside contact rests an injured ankle beside a closed ramp; well-dressed guests use a nearby unobstructed entry.',['Donor entrance']),
('06-B','The Unpaid Department','doge_parks','Your contact {name} hurts their back volunteering after DOGE staffing cuts and must stop helping. They’ve been injured performing a job the government proved nobody needed.','Outside contact sits with a sore back beside cleaning equipment and an empty service desk.',['Position unnecessary']),
('06-C','The Stage of Free Speech','campaign_money','Your contact {name} strains a shoulder hauling equipment for a protest against billionaire political spending. Free speech, it turns out, is heavier when nobody delivers it for you.','Outside contact rests a sore shoulder beside speakers on a handcart; a distant donor event has professional stagehands.',['Money talks'])
]
for suffix,title,topic,setup,scene,overlays in ALLY:
 u=by['ALLY-'+suffix];u.update(title=title,setup=setup)
 stamp(u,topic,scene,overlays)

sources['tips']={'subject':'No tax on tips and overtime','fact_and_limits':'The 2025 law provides deductions for qualified tips and qualified overtime with limits. It does not remove every tax, turn wages into tips, or require employers to increase pay.','url':'https://www.irs.gov/newsroom/one-big-beautiful-bill-how-to-take-advantage-of-no-tax-on-tips-and-overtime'}

ORDERS=[
('SHUTDOWN-A','Essential, Unpaid','campaign_money','Shutdown bulletin: Congress has run out of agreement. Federal workers are asked to keep being essential on an optional-salary basis.','The shutdown ends. Paychecks may resume pretending this was a scheduling problem.','Closed public-service windows below a warmly lit political fundraiser; an ordinary worker waits with a lunch bag.',['Government shutdown']),
('SHUTDOWN-B','Open for Contributions','campaign_money','The government shuts down. Services close; political fundraising emails continue arriving. Apparently the donation button is essential infrastructure.','Services reopen. Nobody has needed to reboot the donation button.','Dark public-service monitors beside one brightly glowing, blank campaign-donation screen.',['Government closed','Donate']),
('SHUTDOWN-C','Hostage Negotiations','snap','Congress fails to keep the government open. Your next meal has been invited to participate in the negotiations. It wasn’t given a chair.','The shutdown ends. The food budget is released without a thank-you note.','An ordinary empty dinner plate sits below oversized negotiation chairs in an illustrative cutaway; no claim of an actual meeting.',['Government shutdown']),
('MILITARIZE-A','Freedom of Movement, Subject to Movement','war_name','Checkpoint bulletin: officials invoke Trump’s border-security rhetoric to add another road inspection. The officer asks why free citizens are always trying to go somewhere.','The extra inspections end. Your destination is once again a place rather than an explanation.','Van waits behind a new portable barrier while an officer consults a blank checklist.',['Security inspection']),
('MILITARIZE-B','America, Please Hold','war_name','Trump-style extreme vetting demands proof you belong here. The van’s registration, your accent and your patience are called forward separately.','The restriction expires. Your patience is still waiting to have its documents returned.','Traveler holds blank documents at a checkpoint; the van waits behind them with luggage visible.',['Travel restriction']),
('MILITARIZE-C','The Domestic Border','war_name','Officials bring “extreme vetting” to another stretch of road. Your crew explains you’re already in America. The guard says that’s what everyone says.','The order ends. The road stops interviewing its users.','An ordinary domestic road is split by a portable inspection booth; a traveler points toward the same landscape on both sides.',['Extreme vetting']),
('GAG-A','A Very Dangerous Index','schooling','Book-panic bulletin: officials copy Trump’s ban on classroom “indoctrination.” The approved reading list arrives pre-approved, with instructions on how freely to discuss it.','The panic recedes. A librarian quietly moves the books back where people can accidentally learn things.','Librarian stands beside a locked book cart and a large blank approved-reading sheet.',['Approved free thinking']),
('GAG-B','Patriotic History, Selected Scenes','history','Trump’s flattering-history directive reaches the museum. The museum’s new slavery exhibit consists of a very attractive curtain.','The directive fades. Someone opens the curtain; history has been there the entire time.','Opaque curtain covers the entrance to a historical exhibit; an adult visitor studies the conspicuous covered space.',['History, improved']),
('GAG-C','The Pronoun Emergency','schooling','Officials announce another emergency about gender in schools. The roof still leaks. Nobody has established the puddle’s biological sex.','The bulletin expires. The puddle has acquired a second bucket and no political affiliation.','A classroom hallway has two buckets beneath roof leaks while a huge blank policy binder sits dry on a shelf.',['Emergency guidance']),
('TARIFFS-A','Foreign Aid, Domestic Wallet','tariffs','Trump’s tariff message says foreign countries will pay. Your grocery bill has volunteered to be a foreign country.','The tariff event ends. Your wallet would like its citizenship restored.','Plain grocery bag sits between an empty wallet and a blank customs invoice.',['Foreign countries pay']),
('TARIFFS-B','The Importer Is You','tariffs','Another tariff bulletin promises to punish overseas producers. At the shop, the punishment is itemized beneath your bread.','The event expires. The bread is released into your custody.','Cashier shows a blank receipt long enough to curl around one modest loaf.',['Tariff surcharge']),
('TARIFFS-C','Protected From Purchasing','tariffs','Officials announce protection for American consumers through higher import taxes. The food remains safely on the shelf, beyond the reach of American consumers.','The tariff event passes. You and the shelf resume diplomatic relations.','Traveler stretches toward groceries while a tall blank price card visually stands between them.',['Protected market']),
('TAXCUTS-A','Education Has Left the Building','education','The education department is eliminated. Officials promise to preserve learning. The exit sign has been assigned the remaining teaching hours.','The order ends. The exit sign returns to its specialist subject.','Empty education office with a lone illuminated, unlettered exit-sign panel above a tiny lectern.',['Department closed']),
('TAXCUTS-B','School of Hard Cuts','education','The education department closes. Students are directed to the school of hard knocks. It is already charging for the door.','The order expires. The school of hard knocks keeps your application fee.','An office door is closed while a student-facing service counter has a blank fee jar and no staff.',['School of hard knocks']),
('TAXCUTS-C','Discover It Yourself','education','Officials eliminate the education department. The announcement calls it empowerment. The teacher begins explaining long division with the remaining half of a pencil.','The order ends. The pencil requests a colleague.','Teacher holds half a pencil before a large empty board; a blank closure notice is pinned beside the door.',['Empowerment']),
('DEREGULATE-A','Peace, Now With More War','war_name','The government promotes “Department of War” branding. The spokesperson calls it peace through strength. Your van would settle for peace through a working alternator.','The reorganization event ends. The van has retained its original, less threatening name.','Official renaming ceremony on a roadside screen; the van’s hood sits open in the foreground.',['Department of War','Peace through strength']),
('DEREGULATE-B','Two Names, One Purchase Order','war_name','Trump’s War Department branding brings another reorganization. Officials order new signs before checking the equipment. Your van has nominated its rattle for a leadership position.','The event expires. The rattle is demoted to a maintenance issue.','A pristine blank sign is delivered beside neglected machinery while a traveler listens to the van.',['Reorganization']),
('DEREGULATE-C','The Branding Campaign','war_name','Officials explain that calling defense “war” will project strength. The crew tries calling the van “a tank.” The suspension files an immediate objection.','The branding event ends. The van accepts a return to civilian duties.','Travelers glance at their battered van beside a grand military-name unveiling shown on an unlettered display.',['Department of War'])
]
for suffix,title,topic,activation,expiration,scene,overlays in ORDERS:
 u=by['ORDER-'+suffix];u.update(title=title,activation=activation,expiration=expiration)
 stamp(u,topic,scene,overlays)
 u['political_basis']['fictionalization']='Simulated game order and effects riff on documented political rhetoric/actions. It is not a claim that this bulletin or every described restriction was enacted in reality.'

ACTS=[
('FORAGE-A','No Payslip Required','benefit_work','A local guide leads a foraging walk while neighbors discuss Congress’s food-aid work rules. The blackberry bush has failed to install a document-upload portal.','Gather what the guide identifies','You gather some food. The bush accepts evidence that you are hungry. Washington may want to audit it.','Guide points out familiar edible berries to current travelers; a blank phone form sits unused beside a basket.'),
('FORAGE-B','The Redundant Botanist','doge_parks','A botanist laid off in the DOGE cuts shows your crew what is safe to gather. She points to a dangerous lookalike. “That advice used to be considered waste.”','Follow the botanist’s guidance','You collect some food and avoid the lookalikes. The savings remain difficult to eat.','Former public botanist shows travelers two clearly different plants without a harvesting tutorial or readable labels.'),
('FORAGE-C','A Public Option With Thorns','campaign_money','At a foraging walk, the guide says lobbyists get dinner for discussing food policy. You’ll be getting dinner from a bush. The bush has fewer sponsors.','Gather with the guide','You bring back some food. None of it demands a seat on the committee.','Ordinary travelers gather with a guide while a distant, elegant catered reception is glimpsed beyond a venue fence.'),
('GLEAN-A','The Tariff Harvest','tariffs','A farmer lets your crew glean produce while grumbling about tariffs on tractor parts. His crops grew locally. Their operating expenses have been around the world.','Glean the remaining produce','You work the row and collect food, sore from the effort. The farmer waves a repair bill. “China missed another payment.”','Farmer holds a blank machinery invoice beside a tractor while travelers consider a row of leftover produce.'),
('GLEAN-B','The Free-Market Carrot','snap','A farmer offers your crew leftover produce while the radio covers Congress’s food-aid cuts. “Plenty of food,” she says. “We’re running short of permission to eat it.”','Glean with permission','You gather food and wear yourselves out. Every carrot makes it past the eligibility committee.','Farmer gestures to unharvested produce beside an ordinary radio and an unlettered benefits notice.'),
('GLEAN-C','Independence Day Labor','benefit_work','A farmer lets the crew glean after neighbors discuss SNAP work requirements. A crooked carrot has been rejected by the buyer. It would like credit for its hours underground.','Collect the rejected produce','You collect food at the cost of sore muscles. The carrot’s employment record remains spotless.','One conspicuously bent carrot sits atop a crate beside a row the crew may glean.'),
('FOODWORK-A','Your Dinner, Your Assignment','snap','A pantry offers food for a three-hour sorting shift. Congress has cut projected food-aid spending. The volunteer calls your crate “the local implementation plan.”','Sort donations for food','You finish tired and take the food. Congress can call it self-sufficiency; you have seen the donation labels.','Pantry coordinator offers a sorting station beside clearly separated incoming donations and outgoing groceries.'),
('FOODWORK-B','The Donor Leftovers','campaign_money','A community kitchen offers food for sorting leftovers from a super-PAC dinner. Unlimited political speech appears to include an extraordinary amount of chicken.','Sort the leftovers','You earn food and lose patience. A drumstick finally reaches someone without a donor badge.','Kitchen worker stands between untouched catered trays and plain takeaway containers; travelers have not started work.'),
('FOODWORK-C','American-Made Hunger','tariffs','A pantry offers food for a sorting shift. A volunteer points to tariff-inflated grocery invoices. “China pays,” she says, holding out an empty till. “We’re waiting.”','Sort food for the pantry','The work costs patience but earns food. The till remains impressively international in its emptiness.','Volunteer displays an empty cash drawer beside a sorting bench and plain grocery invoices.'),
('CASHWORK-A','Family Values Payroll','tips','A diner offers $18 for a three-hour cleanup. The owner praises Trump’s “no tax on tips” promise. You ask about the wage. He points at the tip jar.','Take the cleanup shift','You collect the $18 and lose some patience. The empty tip jar enjoys a completely tax-free afternoon.','Owner gestures to an empty tip jar beside dishes waiting to be cleared; travelers are still considering the job.'),
('CASHWORK-B','The Self-Made Stage','campaign_money','A venue offers $18 to unload staging for a billionaire’s political fundraiser. The banner says he built everything himself. It needs six people to lift it.','Unload the stage equipment','You finish the shift with $18 and less patience. The billionaire takes the stage alone, preserving the illusion.','Large blank banner and stage crates await lifting while a wealthy donor rehearses near an assembled podium.'),
('CASHWORK-C','Infrastructure Week, Again','local_funding','A shop offers $18 to unload a delivery delayed by broken local access roads. The owner calls it Trump’s Infrastructure Week. The driver calls it Thursday.','Unload the delivery','You collect $18, tired of the speeches. The delivery truck has completed more public works than the slogan.','Delivery truck waits by a damaged access road; shop owner offers work beside untouched crates.'),
('BARTERTIRE-A','Independent Spending','campaign_money','At the community exchange, a resident offers a tire for supplies. A nearby radio discusses unlimited political spending. You check whether the tire comes with a senator.','Trade supplies for the tire','The tire is yours. No senator included, but it may actually get you somewhere.','Resident presents an intact spare beside a modest grocery bundle and an ordinary radio.'),
('BARTERTIRE-B','A Trade War With Snacks','tariffs','A resident offers a spare tire for supplies. Neither of you wants another tariff-inflated shop bill. The negotiation begins over a tin of beans, without threatening Canada.','Barter for the spare tire','Supplies traded, tire acquired. Peace holds between the glove box and the grocery bag.','Two ordinary people compare an intact tire and plain groceries across a community swap table.'),
('BARTERTIRE-C','Roundtable Negotiations','campaign_money','A senator’s donor-dinner ad plays at the exchange. A resident offers a tire for supplies. “I can also listen to your concerns,” she says, “but the tire’s extra.”','Make the exchange','You leave with a tire. Your roundtable has tread and no fundraising target.','A spare tire lies flat on a swap table like an oddly practical conference table.'),
('BARTERBATTERY-A','Energy Independence','energy','Someone offers a battery for supplies at the exchange. Trump promises American energy independence on the radio. You ask if that includes starting the van.','Trade supplies for the battery','You get the battery. The van’s energy policy now has terminals.','Resident holds a replacement battery beside a small grocery bundle; a radio sits nearby.'),
('BARTERBATTERY-B','The Tariff-Free Conversation','tariffs','A resident offers a spare battery for supplies. After hearing the shop’s tariff quote, direct diplomacy between two broke people is looking remarkably sophisticated.','Make the battery trade','Supplies leave; a battery arrives. China has been spared another bill it wasn’t going to pay.','Exchange table holds a plain battery, groceries and an unopened printed shop quote.'),
('BARTERBATTERY-C','The Grid Is Negotiable','energy','At the exchange, a resident offers a battery for supplies. Politicians on the radio debate an energy emergency. The resident has brought the part you actually need.','Barter for the battery','The battery is yours. The debate continues without providing any voltage.','Resident taps a real battery while an unlettered radio display glows beside empty political leaflets.'),
('BARTERSUPPLIES-A','A Very Small Redistribution','snap','A resident offers food for your spare tire. Congress is discussing who deserves assistance. The resident mainly wants a tire that holds air.','Trade the spare for supplies','You get supplies. Eligibility was established by looking at the tire.','Resident sets plain groceries opposite an intact spare; no money or goods have changed hands yet.'),
('BARTERSUPPLIES-B','The International Tire Standard','tariffs','Someone offers supplies for your tire. Trump calls tariffs a brilliant negotiating tool. The trader points at the food. “Or we could both leave with something.”','Swap the spare for supplies','You leave with supplies. The tire enters its new market without a presidential address.','Two travelers and a local regard a tire and groceries arranged like a modest negotiating table.'),
('BARTERSUPPLIES-C','The Budget That Feeds Someone','health_budget','A resident offers supplies for a spare tire while Congress’s budget fight plays on a radio. You manage to move resources without discovering anyone is undeserving.','Exchange the tire for supplies','The trade is done. Your budget has produced lunch, putting it ahead of several televised proposals.','Simple swap table in front of a radio; groceries are unmistakably practical, the spare visibly intact.'),
('REST-A','Restoration of Executive Function','doge_parks','The crew can rest for a day. After another DOGE efficiency story, someone suggests firing the tired people. This would leave the van impressively understaffed.','Take a day to rest','Rest restores what it can. You retain the staff, including the person who knows where the keys are.','Current crew rests in or beside the parked van; one traveler holds the keys while a muted radio sits aside.'),
('REST-B','The Trickle-Down Nap','campaign_money','The crew can stop for a day. A billionaire on the radio praises relentless work. Nobody can hear his staff carrying the microphone.','Rest for the day','You recover within the day’s limits. Productivity declines in the radio audience; snoring receives a majority.','Travelers settle at a modest rest stop while a blank radio display shows a distant donor broadcast through a separate overlay.'),
('REST-C','National Security, Airplane Mode','signal','The crew can take a day off the road. You silence the phones before another headline about the cabinet’s Signal chats. Let someone else accidentally run the Pentagon.','Rest and mute the phones','The crew rests. No airstrike plans are found in the sandwich order. A quiet diplomatic success.','Phones face down beside wrapped food while current crew rests by the van; no content embedded on screens.')
]
for suffix,title,topic,setup,action,outcome,scene in ACTS:
 u=by['ACT-'+suffix];u.update(title=title,setup=setup,action=action,outcome=outcome)
 stamp(u,topic,scene)

CONDITIONS=[
('CLEAR-A','Unregulated Sunshine','energy','Clear skies. The radio credits Trump’s energy agenda. The sun declines the cabinet appointment.','The clear weather continues. The sun has not returned the communications team’s calls.','The clear spell ends. Apparently the sun serves at its own pleasure.','Clear sky over the van; a radio inside carries a boastful announcement through a separate overlay.'),
('CLEAR-B','Forecast Approved','sharpie','Clear weather gives the crew a break. Nobody has needed to improve the forecast with a Sharpie.','The sky remains clear without editorial assistance.','The weather changes. The marker is referred to the complaints department.','Open road beneath clear sky, with an unused marker lying beside a folded weather map.'),
('CLEAR-C','The Public Option','doge_parks','Clear skies. DOGE fired forecasters; the remaining staff called this one. The sunshine has been given until Monday to explain what it accomplished.','The forecast continues to match the sky. An unfashionable government outcome.','The clear spell ends; check the actual new forecast, not the applause.','Sunlit van and an ordinary weather radio beside an unlettered forecast map.'),
('STORM-A','The Map Is Dry','sharpie','A storm lashes the van. You consider the Sharpiegate approach, but there isn’t enough marker to cross out the sky.','Rain continues. The forecast map is the only dry territory you control.','The storm passes. The sky has corrected itself without a press conference.','Rain beats against the van while a traveler holds a small marker beneath an enormous storm front.'),
('STORM-B','The Energy Announcement','energy','Heavy rain hits as the radio celebrates climate-policy rollbacks. The announcement is waterproof. The passenger-side window isn’t.','The storm continues. You move the radio away from the leak so optimism can remain operational.','The rain eases. The leak remains available for the next victory speech.','Travelers catch a roof drip in a cup while moving a radio out of its path.'),
('STORM-C','Patriotically Wet','gulf','The storm does not care what Trump calls the Gulf. Water arrives under both names and immediately ignores your towel.','Rain keeps coming. You have recognized its sovereignty over the seat cushion.','The storm ends. The towel requests disaster assistance.','Wet van interior with two unlettered maps and one saturated towel between them.'),
('HEAT-A','The Freedom to Sweat','heat_preemption','A heat wave settles over the route. Politicians call mandatory water breaks overreach. Your sweat appears to be a grassroots protest.','The heat persists. The protest has spread to every seat.','The heat eases. The seats begin negotiating a return to normal relations.','Travelers sit in shade beside the van, with visibly hot upholstery and an ordinary water bottle.'),
('HEAT-B','Drill, Baby, Swelter','energy','The radio plays “Drill, baby, drill” over another heat warning. The dashboard has begun producing its own plastic reserves.','The heat continues. The steering wheel remains too patriotic to touch.','Temperatures ease. The dashboard declines further energy development.','Heat haze bends the road; a traveler cautiously hovers a hand above the hot steering wheel.'),
('HEAT-C','Executive Air Conditioning','heat_preemption','A heat wave arrives. On the radio, an official opposes worker heat rules from a room you can hear being air-conditioned.','The heat persists. The studio thermostat remains outside the debate.','The heat breaks. The official’s jacket has survived without comment.','Crew seeks shade outside the van; a distant broadcast image shows a suited official beside visible cooling vents, with no text.'),
('COLD-A','A Private Climate','energy','Cold grips the route. The fossil-fuel lobby calls its dinner an energy-security event. You’d like to attend the room-temperature portion.','The cold continues. Energy security still appears to be indoors.','The cold eases. Your fingers regain the ability to disagree with a press release.','Travelers bundle up by the van while a warmly lit donor-dinner image appears on a blank display.'),
('COLD-B','Frozen Federal Assets','doge_parks','The cold settles in beside a reduced-hours public visitor center. DOGE saved on heating the office by removing the people who opened it.','Cold persists. The locked visitor center is achieving excellent energy savings.','Conditions improve. The door remains a separate policy problem.','Cold travelers outside a visibly closed visitor center, with a blank reduced-hours notice on the locked door.'),
('COLD-C','Personal Responsibility, Insulated','campaign_money','The temperature drops. A donor’s billboard praises personal responsibility. Your blanket would like to know whether that comes in fleece.','The cold continues. You add another layer without forming a super PAC.','The cold eases. Warmth arrives without requesting your billing address.','Crew shares appropriate blankets beside the van under a blank luxury-donor billboard.'),
('SMOKE-A','Now With More Freedom','energy','Smoke hangs over the road. The radio celebrates rolling back environmental rules. You close the windows on the celebration.','The smoke persists. Freedom has developed a taste you can’t get out of your mouth.','The smoke clears. Breathing briefly returns to being a background activity.','Smoky road beyond tightly closed van windows; travelers check conditions on a blank phone.'),
('SMOKE-B','Visible Deregulation','energy','Smoke reduces visibility. Officials say environmental rules hold business back. The horizon appears to have been released from its obligations.','Smoke continues. Your next landmark has exercised its freedom to disappear.','The air clears enough to see farther. The horizon resumes public service.','The van faces a road fading into smoke, with the next road sign only a silhouette and no legible text.'),
('SMOKE-C','The Museum of Clean Air','history','Smoke settles over the route. A museum follows Trump’s flattering-history directive with an exhibit celebrating American industry. You are inhaling the deleted scenes.','The smoke persists. The souvenir has entered through a gap in the window seal.','The air clears. You decline the gift shop’s take-home edition.','Smoky air surrounds an industrial-history display outside a closed van window; the panel remains blank for overlays.'),
('ILLNESS-A','An Unfunded Body','health_budget','Illness hits the journey. Congress calls health-budget cuts a saving. Your body would like to see the return address for whatever it caught.','The illness continues. Your body remains inconveniently outside the budget’s success metrics.','The illness clears. Recovery has happened despite the running commentary.','An unwell current traveler rests beside plain care supplies and a muted budget-news broadcast.'),
('ILLNESS-B','Natural Selection of a Chair','raw_milk','Illness interrupts the trip. A MAHA advertisement offers to help your body heal itself. The crew starts with a chair that makes fewer promises.','The illness persists. The miracle advertisement remains in excellent health.','The illness clears. The chair asks for no testimonial.','Affected traveler rests in a modest chair while a glossy, unlettered wellness poster hangs far behind.'),
('ILLNESS-C','The Medical Work Requirement','benefit_work','Illness slows the crew. The radio explains Congress’s new Medicaid work requirements. Your immune system would like its overtime recorded.','You remain unwell. The immune system has not taken a lunch break or received a supervisor’s signature.','The illness clears. Your body submits its completed assignment by getting out of the chair.','Unwell traveler under a blanket with a blank work-requirements form deliberately set aside.'),
('HUNGER-A','Trickle-Down Lunch','snap','Food is running short. Congress has cut projected food-aid spending. Someone holds an empty bowl under the radio, just in case the savings trickle down.','Hunger continues. You turn the bowl around to improve reception.','Food relieves the hunger. It arrives through an actual person, a supply the radio had overlooked.','Empty bowl held below an ordinary radio inside the van; no magic food or actual benefit enrollment implied.'),
('HUNGER-B','The Protected Pantry','tariffs','Food runs low while Trump’s tariff pitch plays on the radio. The crew has been successfully protected from the contents of several shops.','Hunger persists. China has not sent sandwiches.','You eat. International trade briefly becomes a matter of chewing.','Travelers look from a sparse food bag toward a shop window filled with unreachable groceries and blank price tags.'),
('HUNGER-C','Donor-Class Calories','campaign_money','Supplies are low. A political fundraiser advertises dinner with a senator. The crew would settle for dinner without one.','Hunger continues. Access to the senator appears to be the expensive part of the chicken.','You get enough food. The absence of a senator improves the meal.','Hungry travelers compare a modest food bag with a blank donor-dinner leaflet showing an extravagant plate.'),
('HEATEXPOSURE-A','The Cost of Local Control','heat_preemption','Prolonged heat is now harming the crew. Opponents of mandatory heat breaks promised freedom. Your body has mistaken it for permission to shut down.','Heat is still causing harm. Get protection and attend to the crew; the slogan has no cooling setting.','Heat exposure is relieved. Your body accepts the change without a constitutional challenge.','Affected travelers sit in available shade with water and clear heat distress, never a comic corpse.'),
('HEATEXPOSURE-B','Policy-Proof Temperature','sharpie','Heat exposure is causing damage. The Sharpiegate approach has failed: you can change a line on a map, but the steering wheel still burns.','The heat remains harmful. The marker stays comfortably cool in the glove box.','Exposure is relieved. You return the marker to a job within its qualifications.','Traveler avoids a hot steering wheel while a capped marker rests on an unaltered blank forecast map.'),
('HEATEXPOSURE-C','An Energy Success Story','energy','The heat has become dangerous. A roadside “energy dominance” banner flaps above the van. It is providing the most useful part of the policy: shade.','Heat is still harming the crew. Stay protected; the banner is not medical care.','Exposure eases. The banner’s brief public-service career ends.','Travelers shelter beneath the shade of a large blank energy-event banner; no graphic injury.'),
('COLDEXPOSURE-A','The Private Warmth Sector','campaign_money','Cold exposure is now causing harm. The political donor’s promise of opportunity offers several square feet of cardboard. You investigate its insulation policy.','The cold remains harmful. Seek protection; donor enthusiasm has no measurable heat output.','Exposure is relieved. Your hands regain the ability to put the leaflet down.','Cold travelers seek real shelter while one keeps a stiff, blank donor leaflet folded inside a bag, never portrayed as adequate care.'),
('COLDEXPOSURE-B','Essential Services, Optional Hours','doge_parks','Cold exposure is harming the crew near a closed public rest facility. The staffing cuts saved a salary. The door is still costing everybody else.','The cold remains dangerous. Find protection; the locked door will not reconsider its staffing level.','Exposure eases after protection. The closed facility retains its perfect attendance record.','Crew reaches an available sheltered spot beside a locked, unstaffed public building with a blank hours notice.'),
('COLDEXPOSURE-C','Heating the Debate','energy','Prolonged cold is now harming the crew. A politician on the radio promises to heat up the energy debate. You hold a hand over the speaker. Nothing.','The cold is still causing harm. Get actual protection; the speech has reached its third insulated anecdote.','Exposure is relieved. The radio can return to being merely irritating.','Affected traveler holds a hand near a radio before turning toward real shelter offered by the scene.' )
]
for suffix,title,topic,onset,continuing,recovery,scene in CONDITIONS:
 u=by['COND-'+suffix];u.update(title=title,onset=onset,continuing=continuing,recovery=recovery)
 stamp(u,topic,scene)

sources['travel_ban']={'subject':'2025 entry restrictions and extreme vetting','fact_and_limits':'The June 2025 proclamation restricted entry by nationals of specified countries, with exceptions. These fictional domestic checkpoints exaggerate the rhetoric; they do not describe the law or its actual procedures.','url':'https://www.whitehouse.gov/presidential-actions/2025/06/restricting-the-entry-of-foreign-nationals-to-protect-the-united-states-from-foreign-terrorists-and-other-national-security-and-public-safety-threats/'}

# Crossing outcomes narrate the existing resolution; they introduce no new action.
CROSSINGS=[
('01-A','American Enough','tariffs','At the checkpoint, an officer demands an American-made van. You open the hood. He starts an international incident with the alternator.',
 'He waves you through. The alternator is admitted on a temporary work visa.',
 'You take the detour. The alternator remains suspiciously good at its job.',
 'Your permit clears the van. The alternator is included as a dependent.',
 'The payment clears the barrier. Your money passes the nationality test.',
 'He takes the payment and keeps the barrier down. Apparently tariffs work here too.',
 'The officer refuses passage. You take the detour, with the alleged foreign threat still charging the battery.',
 'Officer peers suspiciously into the open hood of the stopped van; an ordinary alternator becomes the focal point. No dismantling or damage.'),
('01-B','Geographic Contraband','gulf','The checkpoint officer spots “Gulf of Mexico” on your map. Trump renamed it. The officer holds the map by one corner, like it might leak Mexicans.',
 'He lets you pass after folding the offending Gulf out of sight. The ocean has been contained.',
 'You take the detour. Both names continue to contain the same water.',
 'Your permit is accepted. Apparently the map has diplomatic immunity.',
 'The payment gets you through. He becomes surprisingly relaxed about where things come from.',
 'He takes the payment but refuses passage. You have purchased a very patriotic delay.',
 'He refuses passage. You detour around the checkpoint; the Gulf stays where it was.',
 'Officer holds a folded blank map at arm’s length beside the waiting van. The tiny coastal area, not any human ethnicity, is the visual target.'),
('01-C','Where Was the Van Born?','travel_ban','The checkpoint officer announces Trump-style extreme vetting. He asks where the van was born. You point to the manufacturer’s plate. “Long-form certificate,” he says.',
 'You are waved through. The van’s citizenship interview has not improved its suspension.',
 'You detour. The van is now accused of taking jobs from American vans.',
 'Your permit is accepted. The van is apparently a naturalized cargo space.',
 'The payment opens the barrier. The van has qualified for the investor route.',
 'The payment is taken; the barrier stays down. The van’s investment has become foreign aid.',
 'Passage is refused. You take the detour before he asks the spare tire for its grandparents.',
 'An officer bends toward a blank manufacturer plate while the travelers wait beside the van; a towering stack of blank forms dwarfs the tiny plate.'),
('02C-A','The Department of Barrier','war_name','The crossing office has copied Trump’s “Department of War” makeover. The new nameplate is enormous. Behind it, two workers are holding up the barrier by hand.',
 'You cross while the workers hold the barrier. The nameplate offers strong moral support.',
 'You take the detour. The new department has secured a decisive victory over traffic.',
 'The permit clears you. A worker lifts the barrier with his actual defense budget: both arms.',
 'The payment gets the barrier raised. The department has discovered arms funding.',
 'The payment disappears into the office. The workers still have only the original two arms each.',
 'You cannot complete this crossing. The expedition ends beneath a nameplate large enough to hide the problem.',
 'Two tired crossing workers physically support a raised barrier beside an oversized blank office nameplate; the van remains on the approach.'),
('02C-B','Merit-Based Concrete','dei','The crossing boss boasts that he removed DEI from the inspection checklist. He points to a crack in the concrete. “That’s a diversity of surfaces.”',
 'You make it across. The concrete remains unpersuaded by the personnel policy.',
 'You detour. The boss has successfully excluded your van from the bridge.',
 'Your permit is accepted. Paperwork has passed a load-bearing test that paperwork really shouldn’t be taking.',
 'The payment clears the crossing. Merit has turned out to have a cash option.',
 'The payment is taken, but passage is refused. You have funded another investigation into the concrete’s pronouns.',
 'This crossing ends the expedition. The boss’s review recommends less diversity in the complaint form.',
 'Boss proudly displays a blank inspection sheet beside an obvious concrete crack, with the waiting van safely before the crossing.'),
('02C-C','China Will Fix It','tariffs','At the crossing, workers have priced replacement steel after Trump’s tariffs. The supervisor keeps refreshing his inbox for China’s payment. A cone has been promoted to bridge repair.',
 'You cross. The cone remains the project’s most dependable appointment.',
 'You take the detour. China has not responded to the supervisor’s fifth reminder.',
 'Your permit clears you. The supervisor briefly considers invoicing it for steel.',
 'The payment clears the barrier. The supervisor puts it in the folder meant for China’s contribution.',
 'The payment is taken without passage. You have been mistaken for another country with spare money.',
 'The crossing cannot be completed. Your expedition ends; the cone receives another term.',
 'Crossing worker holds a blank steel quote beside a solitary cone and an unfinished repair area. Keep the van safely on the approach.'),
('02D-A','The Purity Lane','schooling','The checkpoint has borrowed Trump’s patriotic-education rules. The officer asks whether you love America. You say yes. He demands proof that you have never wanted anything fixed.',
 'You pass. Loving the country has been temporarily separated from liking the queue.',
 'You detour. The officer records your route choice as a curriculum problem.',
 'Your permit clears you. Apparently paperwork loves America enough for everybody.',
 'The payment clears the lane. Patriotism has a suggested donation.',
 'He takes the payment and refuses passage. Your donation was insufficiently sincere.',
 'You cannot complete the crossing. The expedition ends with the country still insisting it has no problems.',
 'Officer presents a blank loyalty questionnaire to the waiting travelers; a huge blank patriotic banner nearly conceals a broken lane marker.'),
('02D-B','A Small Government Office','doge_parks','DOGE-style efficiency has removed the crossing’s desk clerk. The replacement touchscreen asks you to contact the desk clerk. Someone has helpfully drawn him on a sticky note.',
 'The barrier finally opens. The drawing has outperformed its contractor.',
 'You detour. The touchscreen asks you to rate the unavailable employee.',
 'Your permit clears the crossing. The touchscreen congratulates the drawing.',
 'The payment opens the barrier. The payment system was spared the efficiency cuts.',
 'The payment is accepted; passage is not. Only half the government has been automated.',
 'This crossing ends the expedition. The touchscreen offers a satisfaction survey with one working button.',
 'Empty clerk booth with a blank touchscreen and a sticky note showing only a crude human sketch. The payment reader remains conspicuously lit.'),
('02D-C','National Security, Forwarded','signal','The checkpoint’s national-security team has copied the bombing-plan group chat: the barrier code goes to everyone except the guard. A man selling peanuts knows it.',
 'You cross. The peanut seller has become the best-informed person in national security.',
 'You detour. The guard asks whether your radio has received his instructions.',
 'Your permit clears you. The guard photographs it for a group chat you sincerely hope is smaller.',
 'The payment clears the barrier. The guard finally receives useful instructions from a device.',
 'The payment is taken, but the barrier stays down. The guard blames an unauthorized peanut.',
 'The crossing cannot be completed. Your expedition ends while the guard asks the snack cart for clearance.',
 'Confused guard checks a blank phone beside a lowered barrier; a peanut vendor farther back casually checks his own phone.'),
('03-A','Free Speech Has a Cover Charge','campaign_money','At the final crossing, the official calls campaign spending “speech.” He looks at your van full of actual speakers. “We meant the folding kind.”',
 'You cross. Your voices have briefly cleared a market that prefers banknotes.',
 'You take the diversion. Apparently freedom of speech includes a longer walk to the microphone.',
 'Your permit clears you. For once, paper gets heard without having a face printed on it.',
 'The payment opens the barrier. Your money has delivered a concise and persuasive address.',
 'The money is taken; passage is refused. Your speech has been entered into somebody’s wallet.',
 'The crossing ends your expedition. The official describes the silence as a healthy marketplace of ideas.',
 'Official peers into the ordinary van, then gestures toward a cash tray. The travelers’ case folder sits plainly in view.'),
('03-B','Approved Version of the Journey','history','The crossing office follows Trump’s demand for a more flattering American history. The officer weighs your case folder. “That’s a lot of country that needs improving.”',
 'You cross with the case. The country’s flattering biography has acquired an awkward appendix.',
 'You detour. The officer starts describing your absence as public satisfaction.',
 'Your permit clears you. The folder remains inconveniently full.',
 'The payment opens the barrier. Your story has found a sponsor.',
 'The payment is kept, but passage is refused. Sponsorship does not include editorial control.',
 'You cannot complete this crossing. The expedition ends; the official version is already shorter.',
 'Officer weighs a thick, unlettered case folder beside a conspicuously thin blank official-history booklet, with the van waiting behind.'),
('03-C','The Department of Arrivals','education','The crossing office is copying Trump’s plan to dismantle the Education Department: abolish the desk, announce better service. Your appointment is being held on an upturned bucket.',
 'You cross. The bucket receives no credit for its public service.',
 'You detour. The bucket is now managing both arrivals and complaints.',
 'Your permit clears you. The bucket has retained institutional knowledge.',
 'The payment gets you through. A private contractor immediately offers to rent you a second bucket.',
 'The payment is kept, but passage is refused. The contractor bills for bucket access.',
 'The crossing cannot be completed. The expedition ends; the bucket has been shortlisted for promotion.',
 'An official balances blank appointment papers on an upturned bucket where a desk used to stand. Keep the van before the barrier.')
]
for suffix,title,topic,setup,passed,diverted,permit,bribe_ok,bribe_no,failed,scene in CROSSINGS:
 u=by['CROSS-'+suffix];u.update(title=title,setup=setup)
 terminal='refused_passage' if suffix.startswith('01-') else 'terminal_failure'
 u['outcomes'].update(passage=passed,diversion=diverted,permit_receipt=permit+' One Receipt is used.',permit_tag=permit+' Your permit tag remains available.',bribe_success=bribe_ok,bribe_failure=bribe_no,**{terminal:failed})
 stamp(u,topic,scene)

REPAIRS=[
('TIRE-A','Trickle-Down Tread','campaign_money','A tire bursts beside a billionaire-funded tax-cut billboard. You find the tread beneath “prosperity.” So that’s the part that trickles down.',
 'You fit the spare. Wealth has not redistributed itself, but weight has.',
 'The shop fits the replacement. The billionaire’s billboard remains strangely unwilling to co-sign.',
 'The replacement arrives and is fitted. The delivery driver brings rubber; trickle-down economics sent nothing.',
 'You barter supplies for a used tire and fit it. An exchange with two actual beneficiaries feels suspiciously radical.',
 'You finish the agreed work and fit a used tire. The self-made van required other people again.',
 'Stopped van with a failed tire below a blank luxury political billboard. A strip of tread lies beneath the smiling suited figure.'),
('TIRE-B','Protected From Tires','tariffs','Your tire fails. A replacement quote includes Trump’s tariffs. You check the old tire for patriotism. It is still mostly hole.',
 'The spare is fitted. Its country of origin remains less important than its supply of tire.',
 'The shop fits the replacement. America is one tire richer and you are considerably poorer.',
 'The replacement is delivered and fitted. China does not accompany the driver with your change.',
 'You trade supplies for a used tire and fit it. The tire has already survived one economy.',
 'You finish the arranged work for a used tire and repair. Bringing manufacturing home apparently starts with sweeping the garage.',
 'Traveler compares a blown tire with a blank quote on a phone; tire, wallet and waiting van form the scene.'),
('TIRE-C','A More Flattering Tire','history','The tire splits. On the radio, Trump wants museums to emphasize American greatness. You turn the good side outward. The van remains difficult to drive as an exhibit.',
 'You fit the spare. Correcting the problem works better than curating it.',
 'The shop fits a replacement. The old tire retires before writing its own museum label.',
 'A replacement arrives and is fitted. The delivery driver declines a tour of the good side.',
 'You barter supplies for a used tire and fit it. Its difficult history includes several useful years of being round.',
 'You complete the arranged work and fit the used tire. The old one can finally focus on its legacy.',
 'Traveler points to the good side of the van’s visibly failed tire. The tire remains attached and the repair unresolved.'),
('BATTERY-A','Five Accomplishments, Zero Volts','doge_parks','The battery dies. Inspired by DOGE, a crewmate asks it to list five accomplishments. It has powered every departure. Management asks what it did today.',
 'The spare battery is fitted. Its first accomplishment is making the dashboard stop being a mirror.',
 'The shop fits a replacement. You approve electricity without hiring a billionaire to investigate it.',
 'The replacement is delivered and fitted. The driver has accomplished something visible before sending an email.',
 'You barter supplies for a used battery and fit it. Experience survives another efficiency initiative.',
 'You finish the arranged work and fit a used battery. The four-hour productivity review includes actual productivity.',
 'Open hood above the dead battery; a traveler holds a blank notepad like an absurd performance appraisal. Do not show a fitted replacement yet.'),
('BATTERY-B','Energy Dominance, Locally Unavailable','energy','The battery dies halfway through Trump’s “American energy dominance” speech. The slogan finishes on your phone. National abundance has terrible coverage.',
 'The spare battery is fitted. The van achieves the radical policy objective of starting.',
 'The shop fits the replacement. Energy dominance arrives with a bill in your name.',
 'The battery is delivered and fitted. The driver carries more usable energy than the entire speech.',
 'You trade supplies for a used battery and fit it. Nobody says “dominance.” The engine starts anyway.',
 'The agreed work earns a used battery and repair. Your arms supply the energy policy.',
 'Dark dashboard, silent radio and a phone still playing a speech beside an open hood. All displays blank for overlays.'),
('BATTERY-C','The Qualified Candidate','dei','Your battery fails. The radio blames another problem on DEI. Under the hood, the positive and negative terminals have worked together for years. Neither will comment.',
 'You fit the spare. Positive and negative resume their suspiciously inclusive arrangement.',
 'The shop fits the replacement. Nobody asks the voltage for its political affiliations.',
 'The replacement arrives and is fitted. The driver connects both terminals despite the radio’s concerns.',
 'You barter supplies for a used battery and fit it. The van selects its candidate by starting.',
 'You finish the arranged work and fit a used battery. Competence turns out to involve connecting things.',
 'Two clearly visible battery terminals beneath the van’s open hood, with a traveler glancing from them toward a blank radio.'),
('ALTERNATOR-A','China Is Running Late','tariffs','The alternator fails. The replacement costs more after Trump’s tariffs. The mechanic on the radio asks who’s paying. You say China. He asks for a shorter joke.',
 'You fit the onboard spare. China is excused from this particular invoice.',
 'The shop fits the replacement. Your money crosses the counter with remarkable international confidence.',
 'The replacement arrives and is fitted. The driver asks for you by name, which settles who is paying.',
 'You barter supplies for a used alternator and fit it. The food accepts responsibility for the tariff.',
 'You do the arranged work for a used alternator and repair. China’s shift looks exactly like your crew.',
 'Traveler calls a mechanic beside the stopped van; a blank part quote and a small wallet lie by the exposed alternator.'),
('ALTERNATOR-B','The One-Person Department','doge_parks','The alternator quits. You’ve been asking one tired part to charge the battery and power everything, which is also DOGE’s staffing plan.',
 'You fit the spare. The new department head is immediately given all the old department’s work.',
 'The shop fits a replacement. Restoring capacity is briefly mistaken for waste.',
 'The replacement arrives and is fitted. The delivery driver is spared a presentation about doing more with less.',
 'You trade supplies for a used alternator and fit it. The van reluctantly agrees to retain an experienced worker.',
 'You finish the arranged work and fit a used alternator. The crew handles the workload that the efficiency slogan declined.',
 'Failed alternator framed beneath the open hood; multiple unlettered cables lead from it toward the van’s devices.'),
('ALTERNATOR-C','Charging for Access','campaign_money','The alternator dies. A political fundraiser on the radio offers dinner with a senator. You’d prefer access to twelve volts, but nobody has bought those a ballroom.',
 'You fit the spare. The van accepts a contribution within the engine compartment.',
 'The shop fits the replacement. Your payment secures access to electricity and absolutely no senator.',
 'The replacement arrives and is fitted. The driver does not call the delivery a private audience.',
 'You barter supplies for a used alternator and fit it. The food produces a more measurable return than a donor dinner.',
 'You complete the arranged work and fit the used part. Sweeping a garage buys more movement than meeting a senator.',
 'Travelers study the failed alternator beside a radio carrying an unseen fundraiser announcement; no invented dinner attendance.'),
('FUELPUMP-A','Drill, Baby, Nothing','energy','The fuel pump fails. Trump’s “Drill, baby, drill” plays on the radio. Finding more oil is an unusually loud answer to the wrong broken part.',
 'You fit the spare pump. Fuel reaches the engine without finding a new oilfield.',
 'The shop fits a replacement. Distribution briefly gets more attention than drilling.',
 'The replacement arrives and is fitted. The driver repairs the route between the fuel you have and the engine that needs it.',
 'You barter supplies for a used pump and fit it. The tank’s existing reserves finally reach the public.',
 'You complete the arranged work and fit a used pump. Your arms solve a problem the drilling speech could not reach.',
 'Stopped van with a fuel-pump diagram on a blank repair sheet; traveler points between the actual tank area and engine. No spilling fuel.'),
('FUELPUMP-B','A Very Patriotic Obstruction','tariffs','The fuel pump fails. Trump’s tariff pitch says barriers protect American industry. The engine is currently protected from gasoline and has no follow-up questions.',
 'You fit the spare pump. Fuel negotiates passage into the engine.',
 'The shop fits a replacement. You pay for the pump, including the protection from cheaper pumps.',
 'The pump is delivered and fitted. The barrier between you and your money also comes down.',
 'You trade supplies for a used pump and fit it. Domestic leftovers reopen the fuel route.',
 'The agreed work earns a used pump and repair. The crew provides the labor; China remains unavailable for shifts.',
 'Traveler holds a blank pump quote beside the stopped van, with the empty tool position and closed wallet making the unresolved repair clear.'),
('FUELPUMP-C','The Internal Group Chat','signal','The fuel pump stops sending fuel to the engine. After the leaked bombing-plan chat, you check the exhaust. Perhaps the important stuff went to the wrong recipient.',
 'You fit the spare pump. The engine is finally included in the distribution list.',
 'The shop fits a replacement. Fuel goes to the engine, an apparently elite level of operational security.',
 'The replacement arrives and is fitted. The driver manages to deliver a sensitive object to its intended recipient.',
 'You barter supplies for a used pump and fit it. The engine receives the briefing in a format it can burn.',
 'You complete the arranged work and fit a used pump. The van resumes keeping its combustion plans inside the engine.',
 'Traveler peers skeptically at the harmless tailpipe before returning to a blank fuel-system diagram; the van is safely stopped, with no leak or explosion.')
]
for suffix,title,topic,setup,spare,buy,delivery,barter,work,scene in REPAIRS:
 u=by['REPAIR-'+suffix];u.update(title=title,setup=setup)
 for c,t in zip(u['choices'],[spare,buy,barter,work]):c['outcome']=t
 u['choices'][1]['roadside_outcome']=delivery
 u['outcomes'].update(not_completed='The repair is not complete. The van remains broken.',cashless='Radio contacts can arrange work in exchange for a used part and repair. No cash or spare is needed.')
 stamp(u,topic,scene)

OPENINGS=[
('JOURNALIST-A','Seattle: Added to the Chat','signal','You leave Seattle with five companions and a reporter’s notebook. Trump’s team sent bombing plans to a journalist by accident. You’re hoping your government answers questions nearly as easily.','The van pulls out. You check the route twice, placing it ahead of the national-security team.','Journalist checks a blank map beside the Seattle departure van while five companions load ordinary luggage. A phone stays firmly face down.'),
('JOURNALIST-B','Seattle: The Unapproved Edition','history','Trump wants a more flattering version of American history. You leave Seattle to report the current version. Five companions climb into the van; nobody’s circumstances have been improved by editing.','You head for D.C. The notebook opens to a blank page, still the most optimistic thing in the van.','Seattle departure with a journalist opening an unlettered notebook while five companions arrange visibly worn belongings in the van.'),
('JOURNALIST-C','Seattle: Follow the Money, Slowly','campaign_money','You’re a reporter leaving Seattle with five companions to follow political money to D.C. The billionaires took planes. Your investigation begins by splitting the price of gas.','The van sets off. Following the money would be easier if it occasionally used a rest stop.','Journalist and five companions stand by a fueled departure van, counting ordinary shared travel money rather than a new reward.'),
('ORGANIZER-A','Portland: The Public Workforce','doge_parks','DOGE keeps firing public workers and calling it efficiency. You organize five companions for a trip from Portland to D.C. Fitting everyone’s luggage into one van is your first suspiciously public service.','You set out. Nobody has been fired for pointing out that the cooler needs room.','Portland organizer coordinates five companions fitting luggage and a modest cooler into the van without extra staff.'),
('ORGANIZER-B','Portland: A Very Small Super PAC','campaign_money','Billionaires get unlimited independent political spending. Your Portland crew gets a jar for gas money. You’re taking five companions to D.C. to see whether six actual mouths can compete with one very large wallet.','You head east. The gas jar is your treasurer, and it has already called an emergency meeting.','Organizer and five companions gather around a modest gas-money jar beside the departure van; no implied new funds.'),
('ORGANIZER-C','Portland: Proof of Doing Things','benefit_work','Congress wants more work paperwork from people seeking benefits. You organize five companions for D.C. Between shifts, bills and this trip, everybody is exhausted. You begin documenting the apparently hypothetical labor.','You leave Portland. Organizing the receipts has become the only job that doesn’t pay even badly.','Portland organizer places blank work and expense papers in an existing case folder while five companions finish loading the van.'),
('WHISTLEBLOWER-A','San Francisco: Essential Until Deleted','doge_parks','DOGE calls missing staff “savings.” You know what those people did. You leave San Francisco with five companions and your case for D.C. The van has fewer departments, but you’re keeping the brakes.','You set out. Nobody suggests improving fuel efficiency by removing the driver.','Whistleblower and five companions inspect the ordinary departure van in San Francisco; the brake pedal is visible through an open door.'),
('WHISTLEBLOWER-B','San Francisco: The Wrong Recipient','signal','You’re a whistleblower heading from San Francisco to D.C. with five companions. Officials say government information must be handled carefully. The bombing-plan group chat apparently had a different onboarding video.','You start the trip. The case folder stays shut, an ambitious new standard for information security.','Whistleblower closes an unlettered case folder in the San Francisco departure van while five companions load luggage.'),
('WHISTLEBLOWER-C','San Francisco: An Inconvenient Appendix','history','Trump wants history to emphasize American greatness. Your whistleblower case concerns the small print. Five companions join you for D.C.; the folder fits badly between everybody’s actual living expenses.','You leave San Francisco. The official story has room for improvement, which is awkward for its editors.','Whistleblower squeezes an unlettered case folder beside ordinary luggage and existing bills as five companions board the van.'),
('LOBBYIST-A','Los Angeles: Access, Economy Class','campaign_money','You know how Washington sells access. Unfortunately, your five companions know how much gas costs. You leave Los Angeles for D.C. with lobbying experience and a donor network consisting of the glove box.','The van heads east. Loose change speaks, mostly when you hit a pothole.','Lobbyist and five companions board an ordinary Los Angeles departure van; a few existing coins lie in the open glove box.'),
('LOBBYIST-B','Los Angeles: The Unfunded Briefing','tariffs','You’ve heard the tariff pitch: foreign countries pay. Your crew pays for the supplies. You leave Los Angeles with five companions to explain this accounting problem in D.C., preferably to someone holding the calculator.','You set out. China has declined to split the gas.','Lobbyist shows five companions an ordinary unlettered shop receipt beside groceries being loaded into the departure van.'),
('LOBBYIST-C','Los Angeles: Dinner Not Included','campaign_money','Washington hosts fundraisers where the chicken comes with a senator. You’re taking five companions from Los Angeles to D.C. Your lobbying budget currently covers the chicken, if everybody respects the drumsticks.','You leave. For once, nobody in the van has to pretend the senator is the interesting part of dinner.','Lobbyist and five companions share a modest takeaway meal beside the Los Angeles departure van; no elite venue or hired help.'),
('STAFFER-A','Sacramento: The Brand Survives','war_name','Trump added “Department of War” to the government’s branding. You’re a staffer leaving Sacramento with five companions to ask D.C. for actual help. You name the van “Reliable.” Nothing mechanical happens.','You set out. The new nameplate weighs less than a working spare and accomplishes considerably less.','Staffer holds a small blank van nameplate while five companions load the Sacramento departure vehicle; its actual mechanical state stays unchanged.'),
('STAFFER-B','Sacramento: Inbox Zero, Services Zero','doge_parks','You’ve seen DOGE turn staffing cuts into a success announcement. You leave Sacramento with five companions to explain what disappeared with the staff. Unlike the announcement, your van has to carry its contents somewhere.','You head for D.C. The crew declines to improve legroom by declaring two passengers redundant.','Staffer and five companions carefully fit into the departure van in Sacramento, with every seat occupant visibly real and no extra staff.'),
('STAFFER-C','Sacramento: Let Them Eat Guidance','benefit_work','Congress’s new benefits work rules need explaining. You’re a staffer taking five companions from Sacramento to D.C. Everyone has worked. Nobody has yet managed to eat the guidance document.','The trip starts. You pack the forms beside the food, where they can observe the difference.','Staffer puts an unlettered policy folder beside ordinary food in the Sacramento departure van while five companions finish packing.'),
('SATIRIST-A','San Diego: The Ocean Kept Its Job','gulf','Trump renamed the Gulf. You’re a satirist leaving San Diego with five companions to ask D.C. whether changing the name of broke also helps. The van votes for “temporarily presidential.”','You set out. The newly prestigious budget still requires cheap gas.','Satirist and five companions examine a blank map and their existing travel money beside a San Diego departure van.'),
('SATIRIST-B','San Diego: Please Stop Helping','signal','The government accidentally sent bombing plans to a journalist. You’re a satirist heading from San Diego to D.C. with five companions. You would like to arrive before reality takes all the good lines.','You leave. The crew agrees to keep trip planning out of the national-security chat.','Satirist gathers five companions around a blank route map outside the San Diego departure van; one ordinary phone is deliberately put away.'),
('SATIRIST-C','San Diego: Straight Man Needed','war_name','Trump calls defense “war” and says it promotes peace. You’re taking five companions from San Diego to D.C. to ask what food should be renamed before you can afford it.','The van pulls away. Renaming breakfast “economic confidence” has left everyone hungry for details.','Satirist and five companions pack a modest breakfast beside a blank naming-ceremony image on a phone at the San Diego departure point.')
]
for suffix,title,topic,setup,outcome,scene in OPENINGS:
 u=by['OPEN-'+suffix];u.update(title=title,setup=setup,outcome=outcome)
 stamp(u,topic,scene)

HEARINGS=[
('C-A','The Two-Minute Citizens','campaign_money','At the D.C. hearing, the chair gives you two minutes. His political donors get three hours over dinner. Apparently chicken needs more representation.',
 ['The first procedural round drains your stamina. The chair explains how valuable everyone’s time is, at length.',
  'Another round of procedure. The donor-dinner menu arrives before anyone has addressed your case.',
  'The last procedural round. The chair asks for brevity, then reads the dessert options.'],
 'You endured the procedure. One vote now decides the case. For a moment, the menu goes face down.',
 'You cannot endure the remaining procedure. Your hearing ends before the vote; the dinner reservation survives.',
 'Actual arriving travelers at a small public microphone; the chair keeps a blank case folder beneath an elaborate, unlettered dinner menu.'),
('C-B','Competence, Selectively Required','signal','You reach the D.C. hearing. The chair demands impeccable evidence. A clerk hands him the wrong folder. After the bombing-plan group chat, at least this one contains sandwiches.',
 ['The opening procedural round drains stamina. The chair says the folder was never classified.',
  'The second round addresses procedure. A clerk removes the sandwiches from the official record.',
  'The final procedural round. The chair insists nobody was ever in danger of lunch.'],
 'The procedure is complete. One vote will decide your case, provided they have found the correct folder.',
 'You run out of stamina before the vote. The chair has retained enough energy to deny knowing the sandwiches.',
 'Hearing chair opens a folder containing a neatly wrapped lunch while the actual arriving travelers wait with their case.'),
('C-C','The Approved Account','history','At the hearing, the chair invokes Trump’s call for a more flattering American history. Your case concerns the present. He seems annoyed that history is still being manufactured.',
 ['The first procedural round drains stamina. Your difficulties are reclassified as discouraging presentation.',
  'The second round debates tone. The chair asks whether the unpaid bill could sound more grateful.',
  'The final procedural round. The bill remains the same size despite extensive editing.'],
 'You make it through procedure. One vote will decide whether the room can cope with the unflattering version.',
 'You cannot finish the hearing. No vote follows. The chair’s preferred account is now conveniently shorter.',
 'Actual arriving travelers face a hearing chair who tries covering an ordinary blank bill with an ornate, blank patriotic brochure.'),
('D-A','The Public Input Slot','campaign_money','At the D.C. hearing, a political donor’s envelope gets its own chair. You stand at the public microphone. Unlimited independent spending has developed excellent posture.',
 ['The first procedural round drains stamina. The chair thanks the envelope for making time.',
  'The next round concerns access. A clerk adjusts the envelope’s microphone.',
  'The final procedural round. The envelope has said nothing and is still considered the room’s leading contributor.'],
 'You endured the rounds. One vote decides the case. The envelope is allowed to observe without folding its arms.',
 'You cannot endure the remaining rounds. Your hearing ends before a vote; the envelope is invited to stay.',
 'An absurdly formal hearing chair is occupied by a sealed blank envelope, while the actual arriving travelers stand at an ordinary public microphone.'),
('D-B','Efficiency Hearing','doge_parks','DOGE-style cuts have removed the hearing’s note-taker. The chair announces that no complaints have been recorded since. You are invited to address the empty chair.',
 ['The opening procedure drains stamina. The empty chair receives a commendation for reducing paperwork.',
  'Another procedural round. Your testimony passes through an impressive chain of nobody.',
  'The final procedural round. The chair cites the missing minutes as evidence of broad agreement.'],
 'You make it through. One vote will decide the case; somebody will finally have to count something.',
 'You cannot finish the rounds. No vote occurs. The chair adds your silence to the efficiency figures.',
 'Actual arriving travelers address a vacant clerk’s chair below a self-satisfied hearing official; a blank minute book remains visibly closed.'),
('D-C','Peace Through Volume','war_name','The hearing follows Trump’s “Department of War” branding. The chair calls shouting “peace through strength.” Your case is placed beside a gavel that appears to need its own defense budget.',
 ['The first procedural round drains stamina. The chair pounds the gavel to establish a calm environment.',
  'The next round begins louder. A clerk holds down the water glasses in support of stability.',
  'The final procedural round. The chair announces de-escalation with both hands on the gavel.'],
 'You endure the procedure. One vote decides the case. The table gets a brief ceasefire.',
 'You cannot finish the rounds. Your hearing ends before the vote; the table remains under active management.',
 'Oversized gavel held by a fictional hearing chair, with a clerk steadying water glasses and the actual arriving travelers waiting at the microphone.')
]
for suffix,title,topic,setup,transitions,vote,exhausted,scene in HEARINGS:
 u=by['HEARING-'+suffix];u.update(title=title,setup=setup,transitions=transitions,before_vote=vote,exhausted=exhausted)
 stamp(u,topic,scene)

ENDINGS=[
('VICTORY-A','The Quarter PAC','campaign_money','You win the final vote. A lobbyist asks which super PAC backed you. You show the laundry change. He looks underneath it for the billionaire.','Recorded surviving arrivals celebrate modestly at the hearing while a fictional lobbyist studies a few ordinary coins; no new law-signing ceremony.'),
('VICTORY-B','An Unflattering Success','history','You win the hearing. The chair calls it a triumph of American greatness. He is already editing out the part where you had to argue with him.','Hearing chair tries to take the foreground while actual surviving arrivals exchange a knowing look beside their case folder.'),
('VICTORY-C','The Government Did a Thing','doge_parks','You win the final vote. For once, government responds to the people in front of it. Somewhere in the room, a DOGE enthusiast starts looking for something to unplug.','Actual surviving arrivals share relief at the hearing; a fictional efficiency enthusiast studies the public microphone cable suspiciously.'),
('VOTEFAIL-A','The Wrong Kind of Speech','campaign_money','The final vote goes against you. You brought evidence, testimony and an actual journey. The donor envelope brought none of these and gets invited to dinner.','Recorded surviving arrivals collect their case after the failed vote; a clerk carries a sealed blank donor envelope toward a catered side room.'),
('VOTEFAIL-B','A Less Divisive Ending','history','You lose the final vote. The chair invokes Trump’s preferred, uplifting version of America. Your case is removed from the table before the photograph.','Hearing chair arranges a flattering photo while the actual arriving travelers’ blank case folder is moved just outside its frame.'),
('VOTEFAIL-C','Efficiency Achieved','doge_parks','The final vote rejects your case. A DOGE admirer praises the savings. The government has avoided doing something, after making everybody travel here to ask.','Actual surviving arrivals gather their ordinary bags after the hearing while a fictional efficiency official points proudly at a blank zero-cost chart.'),
('SANITY-A','The Brain Declines the Briefing','signal','You cannot keep going; your sanity is exhausted. Officials who put bombing plans in a group chat will continue lecturing everyone about judgment. Yours has at least recognized an emergency.','Journey case and a safely set-aside radio beside the actual stopped vehicle; show only recorded present travelers, without depicting invented injuries.'),
('SANITY-B','Renaming the Problem','war_name','Your sanity gives out and the journey ends. Trump renamed defense “war.” You try calling this “strategic wellness.” It changes exactly as much.','Unlettered journey notebook beside the actual vehicle at its recorded stopping point; an oversized blank official-name card appears in a clearly editorial inset.'),
('SANITY-C','Five Things You Can No Longer Do','doge_parks','Your sanity is exhausted; the expedition ends. DOGE wanted five weekly accomplishments. You have reached the point where “listen to another announcement” cannot make the list.','Travel radio is switched off beside an unlettered five-line note in the actual stopped van. No suicide imagery or invented death.'),
('DESTROYED-A','America First, Van Last','tariffs','The van is destroyed. The journey ends with American travelers unable to travel. The tariff pitch promised to protect domestic industry; your contribution now appears to be scrap.','Actual destroyed vehicle at the recorded location; no added collision, fire or casualties. A blank repair quote lies among ordinary belongings.'),
('DESTROYED-B','The Department of Scrap','war_name','The van is destroyed and the expedition ends. Calling it a tank did no more than calling defense “war.” At least the scrap yard uses names that describe the contents.','Recorded destroyed vehicle shown honestly, with a grand blank nameplate lying incongruously nearby; no invented tow or scrapyard arrival.'),
('DESTROYED-C','An Extremely Efficient Vehicle','doge_parks','The van is destroyed. It now uses no fuel, needs no driver and delivers no services. A DOGE presentation could make this the best-performing vehicle you ever owned.','Recorded destroyed van beside an editorially juxtaposed blank upward-arrow efficiency chart, with no invented victims.'),
('COLD-A','The Insulated Donor','campaign_money','Cold exposure ends the journey. The political fundraiser’s dinner is kept warm under little silver lids. Apparently even a potato can obtain better representation.','Cold at the recorded stopping place; an explicitly editorial inset contrasts a catered potato beneath a silver cloche. No invented death tableau.'),
('COLD-B','Essential Door, Optional Staff','doge_parks','Cold exposure ends the expedition. DOGE’s pitch was that fewer public workers meant savings. A locked door has no heating bill and an excellent record of refusing help.','Recorded cold stopping place, with a separate illustrative cutaway of an unstaffed public entrance; do not imply a particular denied service caused the ending.'),
('COLD-C','Energy Dominance, Bring a Coat','energy','Cold exposure ends the journey. Trump’s energy-dominance speech continues in a heated room. The microphone is wearing better weather protection than the public.','Actual cold conditions at the stopping place, with a small editorial inset of a foam-covered microphone in a heated briefing room.'),
('HEAT-A','Local Control of the Thermostat','heat_preemption','Heat exposure ends the expedition. Politicians who oppose mandatory worker heat breaks make their case indoors. The thermostat has been allowed to regulate without a public hearing.','Recorded hot stopping place contrasted with a plainly editorial inset of an indoor politician beside an unlettered thermostat.'),
('HEAT-B','The Forecast Has Been Corrected','sharpie','Heat exposure ends the journey. Sharpiegate offered a cheaper approach to bad forecasts: change the map. The sun remains infuriatingly difficult to reach with a marker.','Recorded heat at the stopping place; a capped marker beside a blank weather map in the van, without altered temperature or invented death.'),
('HEAT-C','Drill, Baby, Stop','energy','Heat exposure ends the expedition. The radio is still celebrating Trump’s energy dominance. Somewhere, the air conditioner cooling that announcement is working very hard.','Actual heat at the stopping point; a broadcast image of a cooled studio appears only as an illustrative inset, with no embedded words.'),
('COLLAPSE-HUNGER-A','Trickle-Down, Mouth Open','snap','Hunger ends the expedition. Congress cut projected food-aid spending and called it savings. You cannot eat the forecast, although it now contains a very impressive number.','Empty food container at the actual stopping point beside a blank government budget chart; no invented fatality or benefit denial.'),
('COLLAPSE-HUNGER-B','Access to Dinner','campaign_money','Hunger ends the journey. Political fundraisers still offer access to lawmakers over dinner. It is a strange country where the chicken comes with someone you must convince you need chicken.','Empty ordinary plate in the actual van, with a clearly editorial inset of an elaborate donor dinner place setting.'),
('COLLAPSE-HUNGER-C','A Well-Protected Loaf','tariffs','Hunger ends the expedition. Trump’s tariff pitch said foreign countries would pay. The food stayed on the shelf, impeccably protected from domestic consumption.','Sparse food storage in the actual stopped van; a separate illustrative shop shelf remains full behind blank price cards.'),
('COLLAPSE-VEHICLE-A','The Vehicle Has Rebranded','war_name','Vehicle condition ends the expedition. Trump gave defense a fiercer name. You could try the same with the van, but “unstoppable” would now be blocking traffic.','Actual stopped and damaged vehicle at the recorded location; blank grandiose name card kept separate from the body, with no invented destruction.'),
('COLLAPSE-VEHICLE-B','A Job Creator','tariffs','The van’s condition ends the journey. Tariffs were meant to revive American manufacturing. Staring at the repair costs, you wonder how many new jobs you personally are supposed to take.','Actual damaged vehicle beside a blank repair quote and the travelers’ existing work notes; no invented bill paid.'),
('COLLAPSE-VEHICLE-C','The Maintenance Department Is Gone','doge_parks','Vehicle condition ends the expedition. The DOGE theory says less support produces better performance. Your van has submitted its rebuttal by refusing to move.','Actual stopped vehicle at the recorded location with visibly worn components appropriate to its state; no invented crash or fixed road position.'),
('COLLAPSE-WEATHER-A','Weather, Unedited','sharpie','Weather ends the expedition. Sharpiegate demonstrated how to change a forecast with a pen. Nobody demonstrated how to drive across the correction.','Actual recorded weather and stopping place with a blank route map and small capped marker inside the van.'),
('COLLAPSE-WEATHER-B','The Sovereign Weather System','gulf','Weather ends the journey. Trump renamed the Gulf; the weather has declined to update its paperwork. It continues arriving without proof of citizenship.','Actual weather at the recorded stopping point, with a blank map folded beside the window; no invented coastal location.'),
('COLLAPSE-WEATHER-C','The Prosperity Forecast','energy','Weather ends the expedition. The fossil-fuel speech promises an economy without environmental obstacles. Outside the van, an obstacle has become the entire view.','Actual obstructing weather seen from the stopped van; a radio remains on the dashboard with no readable lettering.'),
('COLLAPSE-BREAKDOWN-A','The Invoice Won','tariffs','The unresolved breakdown ends the expedition. Trump said the tariffs would be paid overseas. The repair quote has spent the whole discussion looking directly at you.','Actual broken van with an unresolved blank repair quote; do not show a replacement fitted or money paid.'),
('COLLAPSE-BREAKDOWN-B','A Working Nameplate','war_name','The breakdown ends the journey. Washington can rename defense “war” and call the job done. Your van’s newly impressive title still needs a functioning part beneath it.','Actual unresolved mechanical failure with a pristine blank nameplate beside the tools; no invented successful repair.'),
('COLLAPSE-BREAKDOWN-C','Efficiency’s Roadside Office','doge_parks','The breakdown ends the expedition. DOGE sells the idea that a system works better with less support. The van is now conducting a very long stationary demonstration.','Actual broken vehicle at its recorded stopping place, with an unlettered efficiency brochure deliberately outside the repair tools.'),
('COLLAPSE-DISEASE-A','Illness Has No Upload Button','benefit_work','Illness ends the expedition. Congress wants work documented before more people can keep benefits. Your body has been doing nothing but fight. It has neglected to create a login.','Unlettered work-requirements form sits unused beside the actual health-care belongings at the stopping place; no invented benefit enrollment or death.'),
('COLLAPSE-DISEASE-B','The Budget Looks Healthy','health_budget','Disease ends the journey. Washington can describe projected health-spending cuts as savings. The budget looks much healthier when none of the sick people are drawn on the chart.','Actual journey health belongings contrasted with a clean blank budget chart in an editorial inset; no invented diagnosis or casualty.'),
('COLLAPSE-DISEASE-C','A Second Opinion From the Slogan','raw_milk','Illness ends the expedition. The MAHA pitch promised health through suspicion of safeguards. The disease has offered no opinion on government and an extremely detailed opinion on bodies.','Actual care belongings and stopped vehicle; a separate editorial image of an unlettered raw-milk stall makes no claim that milk caused this particular illness.'),
('COLLAPSE-CROSSING-A','Money Clears Customs','campaign_money','The crossing ends the expedition. In American politics, enormous sums move freely as independent speech. Your actual people have encountered a less accommodating barrier.','Actual failed crossing and present travelers, with a sealed donor envelope in a clearly editorial inset; no fall, collision or death unless recorded.'),
('COLLAPSE-CROSSING-B','The Office of Not Getting Through','war_name','The crossing ends your journey. Trump’s renaming strategy could make the barrier sound stronger. It already stops ordinary people perfectly; the problem may be who counts as the enemy.','Actual crossing barrier and failed journey state, with an oversized blank official nameplate as the visual focal point.'),
('COLLAPSE-CROSSING-C','The Improved Record','history','The crossing ends the expedition. Trump wants a more flattering national story. The version in which your van never gets through will probably need a smaller font.','Actual failed crossing, paired with an editorially oversized blank official-history book and a tiny inset page; no invented catastrophe.'),
('COLLAPSE-PANIC-A','Nothing to See, Too Much to Hear','signal','Panic ends the expedition. Officials who misplaced bombing plans will continue explaining that everyone else is overreacting. You switch off the briefing; it is the first thing to stop escalating.','Radio switched off at the actual stopping point, with the present travelers shown without invented medical intervention or death.'),
('COLLAPSE-PANIC-B','The Reassurance Department','war_name','Panic ends the journey. The government has renamed defense “war” to make everyone feel safer. The word “relax” could probably use a less heavily armed communications team.','Actual stopped vehicle and set-aside radio; editorial inset of an overbuilt blank reassurance podium, without armed threats toward travelers.'),
('COLLAPSE-PANIC-C','You Have Been Invited to Worry','schooling','Panic ends the expedition. Another political bulletin declares a national emergency over classroom pronouns. A leaking roof has yet to qualify for this level of concern.','Actual stopping place, with an illustrative cutaway of a school bucket catching a roof leak beside a thick blank policy binder.'),
('INCOMPLETE-A','The Committee Is Still Somewhere Ahead','campaign_money','This journey is unfinished. In Washington, the donor dinner is still on the calendar. The chicken has secured an appointment; your case is still on the road.','Open route notebook at the actual current location, with an explicitly editorial donor-dinner place setting inset; no implied arrival or terminal loss.'),
('INCOMPLETE-B','American History, Draft Saved','history','The journey is unfinished. Trump’s flattering version of America can wait for an ending. Your current draft still contains people who need things fixed.','Open, unlettered journey notebook beside the actual vehicle at its recorded location; no completed hearing or invented fate.'),
('INCOMPLETE-C','Delivery Pending','tariffs','The expedition is unfinished. Trump’s tariff pitch promises that someone overseas will pay. Your crew is still waiting for that country to offer a useful driving shift.','Actual current vehicle and open route map, with only the currently present roster; no new driver or asserted terminal outcome.'),
('FALLBACK-A','An Unclassified Ending','signal','This account does not say how the journey ended. After the bombing-plan group chat, check the recipient list. The government may have sent the conclusion to a man who sells patio furniture.','Unlettered journey notebook missing its final entry; a blank misdirected-message screen appears as an editorial gag, without asserting any traveler’s location or fate.'),
('FALLBACK-B','The Official Blank Space','history','The ending is missing from this account. Trump’s preferred, flattering history has competition: this version contains no recorded mistakes at all. An editor somewhere is delighted.','Open blank final page of a journey notebook with an absurdly ornate blank official-history cover beside it; neutral background, no invented ending.'),
('FALLBACK-C','The Most Efficient Record','doge_parks','The record does not establish how the journey ended. DOGE-style accounting could call that a reduction in bad outcomes. It certainly uses less ink than finding out.','Incomplete unlettered journey log beside a nearly unused pen and blank efficiency chart; no implied victory, defeat, death or D.C. arrival.')
]
for suffix,title,topic,epilogue,scene in ENDINGS:
 u=by['END-'+suffix];u.update(title=title,headline=title,epilogue=epilogue)
 stamp(u,topic,scene)

sources['five_things']={'subject':'DOGE five-accomplishments emails','fact_and_limits':'In February 2025 federal employees received an OPM demand for approximately five recent accomplishments; Musk threatened that nonresponse would be treated as resignation. Agency instructions and the threatened consequences were contested. Fictional employers and objects parody the request.','url':'https://www.bangordailynews.com/2025/02/24/nation/federal-workers-confront-mass-confusion-as-musks-deadline-to-list-accomplishments-looms/'}
sources['weather_staff']={'subject':'DOGE-era weather-service firings','fact_and_limits':'AP reported February 2025 probationary firings at NOAA and the National Weather Service, including local forecasters. The fictional weather scene does not assert that any particular forecast was missed or that these firings remain in effect.','url':'https://halifax.citynews.ca/2025/02/27/hundreds-of-weather-forecasters-fired-in-latest-wave-of-doge-cuts/'}
for id in ['CARE-07-A','REPAIR-BATTERY-A','END-SANITY-C']:
 u=by[id];stamp(u,'five_things',u['scene']['description'])
u=by['COND-CLEAR-C'];stamp(u,'weather_staff',u['scene']['description'])
u['sources'].append({'title':sources['five_things']['subject'],'url':sources['five_things']['url'],'qualification':sources['five_things']['fact_and_limits']})
for suffix in ['A','B','C']:
 u=by['ORDER-MILITARIZE-'+suffix];overlays=[o['text'] for o in u['scene']['overlays']];stamp(u,'travel_ban',u['scene']['description'],overlays)
 u['political_basis']['fictionalization']='Fictional domestic travel restrictions in the game exaggerate real entry-ban rhetoric; these are not actual legal procedures.'

(ROOT/'support.json').write_text(json.dumps(units,ensure_ascii=False,indent=2)+'\n')
print('Updated',sum(bool(u.get('political_basis')) for u in units),'support packages with political premises and aligned scenes')
