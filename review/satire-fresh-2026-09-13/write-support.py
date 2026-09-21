import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
BRIEFS = {x['id']: x for x in json.loads((ROOT/'support-brief.json').read_text())}
UNITS = []

def add(identity, **copy):
    b = BRIEFS[identity]
    item = {k:b[k] for k in ('id','family_id','variant','category','mode','regions','runtime_key')}
    item.update(title=b['title'], status='draft for user review', evidence_status='Original fictional scene; gameplay checked against repository', sources=[], variables=[], cast_requirements='Use only currently present travelers; do not infer six survivors.', illustration='New scene art may be commissioned after copy approval.')
    item.update(copy)
    UNITS.append(item)

def save():
    (ROOT/'support.json').write_text(json.dumps(UNITS,ensure_ascii=False,indent=2)+'\n')

# Each row is an authored incident, its continuing state, and its recovery beat.
care = {
 '01': [
  ('Alive with Goodness','{name} tries raw milk from a “Make America Healthy Again” stall. The vendor calls it “alive with goodness.” Judging by the screaming from the toilet, the goodness has a knife.','{name} is still sick from the milk. The vendor has offered a refund in more milk.','{name} receives care and recovers. The next person offering a free sample gets a very clear answer.'),
  ('Freedom from Refrigeration','At a roadside market, {name} drinks raw milk sold as “free from government interference.” Now they’re vomiting. The bottle was also free from refrigeration.','The sickness has stayed with {name}. The stall’s slogan is still in the van, wrapped around the offending bottle.','{name} recovers with care. You discard the remaining milk before it applies for its own passport.'),
  ('Follow the Cow','{name} accepts raw milk from a vendor who says cows don’t need scientists. Now they’re ill. The cow declines to comment, then eats part of the sign.','{name} has not recovered from the raw milk. “Ask the cow” has proved a poor aftercare policy.','Care helps {name} recover. The crew agrees to stop taking medical referrals from livestock.')
 ],
 '02': [
  ('The Marker Forecast','During a paid unloading shift, {name} becomes dehydrated. The boss crossed out the heat warning with a Sharpie, but apparently, the sun doesn’t take corrections. #sharpiegate','{name} is still struggling after the hot shift. The crossed-out warning is back in the van; nobody has managed to cross out the symptoms.','{name} recovers after receiving care. The forecast survives its attempted editing.'),
  ('Executive Shade','{name} becomes dehydrated helping unload an event tent. It’s a conference about worker resilience. The tent’s air-conditioning is already running for the keynote speaker’s bottled water.','{name} is still unwell after working in the heat. The conference has sent a survey asking whether the experience was inspiring.','With care, {name} recovers. You decline the conference’s invitation to do it again for exposure.'),
  ('Premium Water','At a roadside warehouse, {name} takes a short loading job and becomes dehydrated. The manager keeps the drinking water in a locked office. Theft is down. So is {name}.','The dehydration has left {name} weak. The warehouse has sent a reminder that returning workers must bring their own cup.','{name} recovers with care. The warehouse’s invitation to become a team player goes unanswered.')
 ],
 '03': [
  ('Prove You Were Working','After a temp shift, {name} stays up filling out a form proving they worked. It asks how long the form took. They’re now too exhausted to explain the problem.','{name} remains exhausted. A reminder arrives requesting a separate form for the time spent on the first one.','Care and recovery get {name} back on their feet. The unfinished form remains admirably unemployed.'),
  ('The Unpaid Training','{name} loses a night’s sleep completing mandatory training for a one-day job. The final module explains how to avoid bringing work home.','{name} is still worn down from the overnight training. The portal has awarded a digital badge for maintaining boundaries.','{name} recovers with care. You close the training portal before it can assign homework about resting.'),
  ('Automated Efficiency','A temp agency makes {name} retype a résumé the website has already swallowed. By morning they’re exhausted. The company’s chatbot asks if they’ve considered using technology.','{name} has not recovered from the sleepless application marathon. The website has now lost the second résumé.','Care helps {name} recover. The next automated reminder is deleted by a highly qualified human.')
 ],
 '04': [
  ('The Cleanse','{name} tries a roadside wellness tonic advertised to “flush the bad stuff out.” They vomit into the sample bucket. So far, it has removed the tonic.','{name} is still sick from the tonic. The seller says worsening symptoms mean it’s working harder.','{name} receives care and recovers. The tonic completes its own journey into the bin.'),
  ('Influencer Medicine','At a market, {name} accepts a health powder recommended by a man with excellent lighting. Now they’re sick. His medical qualification appears to be a ring light.','{name} remains ill after the powder. The influencer has replied with a discount code for the larger tub.','{name} recovers after care. You unsubscribe from the man and his illuminated qualifications.'),
  ('The Miracle Guarantee','{name} feels sick after tasting a “miracle cleanse” at a roadside stall. The money-back guarantee excludes anyone who has opened the product. The vomiting is apparently a breach of contract.','The cleanse is still making {name} ill. Customer support wants a positive review before discussing the complaint.','Care helps {name} recover. Nobody feels obliged to give the miracle a second chance.')
 ],
 '05': [
  ('Three Patriotic Names','{name} loses sleep navigating detours named Liberty Way, Freedom Drive, and Patriot Parkway. Three declarations of independence. No exit strategy.','{name} is still exhausted from the detours. The old route sign remains clearer than the replacement signs stacked over it.','{name} recovers with care. The route gets checked before anyone follows another inspirational noun.'),
  ('Maps for Subscribers','{name} spends the night comparing road maps after the navigation app puts offline directions behind a subscription. Its free advice is to connect to the internet. In the dead zone.','{name} is still exhausted from reconstructing the route. The app offers a premium trial that requires the same missing connection.','After care, {name} recovers. You keep the checked directions where a billing update can’t eat them.'),
  ('Sponsored Shortcut','{name} stays up untangling a navigation app’s “recommended route.” It circles three partner petrol stations and ends at a billboard saying GO FURTHER.','{name} remains exhausted from checking the route. The app asks whether you enjoyed the journey it invented.','Care helps {name} recover. The next route is chosen for its destination, a neglected little feature.')
 ],
 '06': [
  ('The White Napkin','{name} spends all night disputing a tariff on a van part. The mechanic calls it a trade war. By dawn, they’re waving a white napkin at the card reader as it asks for a tip.','{name} is still exhausted from the price dispute. The invoice has outlasted the napkin.','Care helps {name} recover. The card reader can pursue its diplomatic career without them.'),
  ('The Convenience Fee','{name} loses a night of sleep disputing the van’s repair quote. It includes a convenience fee for coming to the garage. A mobile repair would cost an inconvenience fee.','{name} is still worn down by the dispute. The garage has offered to explain the bill for an explanation fee.','{name} recovers with care. You stop opening new negotiations with the invoice.'),
  ('Already on Hold','{name} stays up contesting a repair-warranty denial. The company’s hold message says it understands that breakdowns don’t keep office hours. Its office opens at nine.','{name} remains exhausted. The warranty company’s callback has become a fresh invitation to hold.','Care helps {name} recover. Nobody confuses the hold music with progress anymore.')
 ],
 '07': [
  ('Permanent Temporary','After a one-day loading job, {name} is kept awake by the boss’s group chat. The shift ended yesterday. The boss has scheduled a midnight poll on work-life balance.','{name} is still exhausted. The temp-job chat has marked their silence as a participation concern.','{name} recovers with care. The chat is muted, surviving a historic loss of engagement.'),
  ('Available at All Times','A temp-job scheduling app wakes {name} every hour to ask if they’re available. By sunrise, they barely are. The app congratulates them on their flexibility.','{name} remains worn down. The scheduling app keeps advertising the freedom to choose which notification wakes them next.','Care helps {name} recover. For once, the scheduling app receives no answer.'),
  ('Read Receipt','{name} loses sleep after a temp supervisor keeps demanding instant replies. The latest message says “No response needed.” Three minutes later: “Did you see my message?”','{name} is still exhausted from the messages. The supervisor has sent a screenshot proving that a previous message existed.','{name} receives care and recovers. The supervisor’s next message is left to experience uncertainty.')
 ],
 '08': [
  ('I Did It All Myself','At a book-event unloading job, {name} throws their back out moving a billionaire’s memoir, I Did It All Myself. The author steps over them on his way to a talk about overcoming obstacles.','{name} is still in pain from unloading the books. The author’s inspirational photograph shows no one carrying them.','Care helps {name} recover. The billionaire’s book remains the heaviest thing he hasn’t lifted.'),
  ('The Charity Photograph','{name} strains their back helping a food pantry move donated boxes. The corporate sponsor arrives to hold an empty one for the photograph. It takes two assistants to find his good side.','{name} is still hurting from the pantry shift. The sponsor’s photograph has appeared under a headline about carrying the community.','{name} recovers with care. The sponsor’s cardboard contribution is recycled.'),
  ('Leaner Operations','Stopping at a food pantry, {name} helps move sacks after its delivery service is cut and strains their back. The replacement plan is called “leaner operations.” They are now leaning against the van.','{name} remains in pain from moving the sacks. The pantry still needs a delivery service, and {name} still isn’t one.','Care helps {name} recover. You retire their brief appointment as a delivery infrastructure project.')
 ]
}
for family,rows in care.items():
    for v,(title,setup,continuing,helped) in zip('ABC',rows):
        source=[]
        if family=='01': source=[{'url':'https://www.cdc.gov/food-safety/foods/raw-milk.html','claim':'Unpasteurized milk can carry harmful germs; this vendor and incident are fictional.','checked':'2026-09-13'}]
        if family=='04': source=[{'url':'https://www.fda.gov/food/dietary-supplements/information-consumers-using-dietary-supplements','claim':'FDA does not approve supplements for safety and effectiveness before marketing; products and sellers here are fictional.','checked':'2026-09-13'}]
        add(f'CARE-{family}-{v}',title=title,setup=setup,continuing=continuing,
          critical='{name} is critically unwell. Pressing on without care now will kill them. If this is your character, the journey will end.',
          choices=[{'label':'Provide care — 2 supplies','outcome':helped,'effects':{'supplies':-2,'morale':1,'clear_incident':True}},
                   {'label':'Leave in shelter','outcome':'{name} stays in shelter and leaves the traveling crew alive. This incident is cleared; they will not continue toward D.C. with you.','availability':'Companions only; unavailable for the player','effects':{'member_status':'departed','morale':-1}},
                   {'label':'Press on','outcome':'You continue without treating this incident. {name} remains unwell; the next care check may be worse.','effects':{'sanity':-1,'strain_retained':True}}],
          outcomes={'helped':helped,'sheltered':'{name} remains alive in shelter and has left the expedition.','deferred':continuing,'companion_lost':'{name} has died after the crew continued without care. Their place in the van is empty.','player_lost':'You have died after continuing without care. Your journey ends here.'},
          variables=['name'],sources=source,cast_requirements='The same active named person and incident persist until recovery, shelter, or death. Critical outcome is triggered only at critical strain. Never shelter the player.',
          integration_note='CARE-08-A uses the authorized new work-injury premise while retaining the injury/care mechanics. All scenario-specific follow-ups require variant-aware integration.')

allies = {
 '01':[
 ('The Hat Commission','Your contact {name} quits helping the trip and joins a red-hat campaign. Their goodbye message includes an affiliate link. You can earn them a commission on the betrayal.'),
 ('A Better Class of Queue','Your contact {name} abandons the cause for a strongman rally. They wanted a leader who cuts through red tape. Their welcome pack includes six waivers and a VIP queue.'),
 ('The Loyalty Subscription','Your contact {name} stops supporting you and signs up for a movement promising to put ordinary people first. Their confirmation email calls them Donor Number 847,219.')],
 '02':[
 ('Efficiency Training','Your contact {name} has been laid off and must stop helping to look for work. Their former employer asks them to train their replacement. It’s the voicemail.'),
 ('Human Capital','Your contact {name} loses their job and leaves your support network to job-hunt. The company announces it is freeing up human capital. The human needs rent.'),
 ('The Thank-You Voucher','Your contact {name} is laid off and can’t keep supporting the trip. Their farewell gift is a discount at the shop that just stopped paying them.')],
 '03':[
 ('Necessary Extras','Your contact {name} has to take evening shifts to pay a repair bill and can’t keep helping. The garage calls the extra charges optional. So is having a working car.'),
 ('Buy Now, Work Forever','A vehicle repair forces your contact {name} to take more shifts and leave your support network. The payment plan is called Drive Free. The installments have other ideas.'),
 ('A Good Trade','Your contact {name} stops helping so they can work extra shifts for a replacement part. They send a photo of the old one. It has retired earlier than they will.')],
 '04':[
 ('Both Sides in Stock','Your contact {name} leaves to sell political merchandise. They stock both factions and call it bridging the divide. The bridge accepts all major credit cards.'),
 ('The Limited Edition','Your contact {name} quits helping to launch anti-billionaire merchandise. The working-class solidarity hoodie costs more than the crew’s last loading shift paid.'),
 ('Terms of Resistance','Your contact {name} leaves to sell protest subscriptions. The first email says your voice matters. The unsubscribe button says leaving requires a phone call.')],
 '05':[
 ('Independent Research','Your contact {name} sends a conspiracy video, tells you to think for yourselves, then blocks the entire crew. Independent thought has been enabled in one direction.'),
 ('The Evidence Cleanse','Your contact {name} stops helping after an influencer claims your trip is a hoax. You offer to call from the van. They say that’s exactly what a van would want.'),
 ('The Secret Source','Your contact {name} withdraws support over an anonymous post claiming to expose everything. Asked who wrote it, they call your question suspicious. The author remains impressively unexposed.')],
 '06':[
 ('The Missed Step','Your contact {name} twists an ankle at a protest and has to stop supporting the trip. The accessible route was blocked by a photo-op about listening to everyone.'),
 ('People on Their Feet','Your contact {name} is injured standing through a long protest and heads home to recover. The speaker had promised to get people on their feet. Reviews are mixed.'),
 ('The Solidarity Lift','Your contact {name} strains a shoulder carrying protest equipment and must withdraw. The campaign director calls everyone indispensable, then asks who can bring the speakers next time.')]
}
last_lines = [
 'That was your last outside contact. The traveling crew remains; your support network is now empty.',
 'No outside allies remain. Nobody in the van has departed because of this message.',
 'Your final outside ally has withdrawn. The travelers still present will have to continue without that network.'
]
for f,rows in allies.items():
    for n,(title,text) in enumerate(rows):
        add(f'ALLY-{f}-{"ABC"[n]}',title=title,setup=text,final_ally=last_lines[n],variables=['name'],cast_requirements='name identifies an external contact, never a member of the named traveling crew. final_ally appears only when the allies counter reaches zero.')

crossings = {
 '01': [
  ('Something to Declare','At the checkpoint, an officer asks whether you have anything to declare. You explain that you’re broke and heading to D.C. He ticks ECONOMY THRIVING.','The officer waves you through. Your poverty has cleared quality control.','You take the detour. The officer’s form records another satisfied customer.','Your permit evidence is accepted. Suddenly the purpose of your trip fits inside the box.','The payment clears the barrier. Money continues to express itself more clearly than people.','The payment is taken, but passage is refused. Your contribution to economic growth has been acknowledged.'),
  ('The Complaint Lane','The checkpoint has a lane for people traveling to complain about the government. It leads to another checkpoint asking why you’re traveling to complain about the government.','You pass. The officer wishes you luck reaching whoever designed his form.','The detour sends you around the checkpoint. Your complaint has entered a different department.','Your permit is accepted. The guard looks almost disappointed that someone has read the instructions.','Payment secures passage. The complaint lane has discovered a fast-track service.','Payment changes hands. The lane still leads nowhere useful.'),
  ('Digital First','A checkpoint scanner refuses your document. The guard blows on it, wipes it on a sleeve, then tries holding it upside down. The poster promises a paperless future.','You pass. The guard celebrates by printing the scanner’s error.','You divert around the checkpoint. The scanner has eliminated your waiting time by eliminating you.','Your permit is accepted. A human performs a dramatic breakthrough called reading.','The payment gets you through. The scanner discovers compatibility.','The payment is accepted; the document still isn’t. The machine has strong boundaries.')
 ],
 '02C': [
  ('Equal Before the Cone','At the second checkpoint, a guard explains that everyone follows the same rules. Behind him, a luxury coach is waved through because its passengers are late for a talk about personal responsibility.','You get through. The cone enjoys a short rest before resuming equality.','You take the longer route. The luxury coach’s passengers will have time to complain about traffic.','The permit is accepted. You qualify for the rare privilege of using the road you were on.','Payment gets the van through. The cone bends to accommodate your argument.','Money is collected. The cone remains exceptionally principled.'),
  ('The Threat Assessment','The second checkpoint searches for dangerous material. A guard lifts your folder of unpaid bills with two fingers. “I understand why you’re angry.”','The van passes. Your bills remain at large.','You take the detour with the bills still aboard. National security has added travel time to the balance.','Your permit clears the check. The guard returns the bills quickly, before any become contagious.','The payment opens the lane. You add another expense to the material under investigation.','The payment is gone. Your folder now contains a fresh reason for the trip.'),
  ('Please Be Patient','The second checkpoint’s public-service announcement asks everyone to remain calm. It plays every thirty seconds. A guard has begun apologizing in time with it.','You pass before the next announcement. Silence feels extravagantly funded.','You detour. The announcement continues giving advice to your empty place in line.','Your permit is accepted. The guard waves you away with the urgency of someone saving a life.','The payment opens the lane. Peace and quiet finally has a published price.','The payment is taken. The next announcement thanks you for your patience.')
 ],
 '02D': [
  ('The Completed Photograph','The bridge is obstructed. Officials have already held its reopening ceremony using a photograph of the other side. A worker is trying to persuade the ribbon to hold a van.','The van gets across. The ribbon receives no engineering credit.','You divert. The official photograph continues enjoying uninterrupted access.','Your permit secures the authorized route. The ceremonial ribbon is finally asked to do something within its abilities.','Payment secures passage. The reopening photograph gains a revenue stream.','Payment is taken, but the obstruction remains. The photograph is still available for viewing.'),
  ('The Bridge Has Logged Out','A bridge contractor’s access terminal says your crossing session has expired. You haven’t crossed yet. The guard recommends logging out of the road and back in.','You cross. The terminal asks how likely you are to recommend bridges.','You divert around the obstruction. The terminal labels this user inactivity.','Your permit is accepted. The guard remembers roads existed before passwords.','The payment gets the van across. Your crossing session has become premium.','The payment is taken. The road still says you aren’t entitled to be on it.'),
  ('Computer Says River','The bridge is visible, but the official map says it was removed for efficiency. The guard points at the screen. The bridge has been asked to stop contradicting it.','You cross the bridge that isn’t there. The official map retains a perfect record.','You divert around the discrepancy. The bridge remains offensively visible in the mirror.','Your permit is accepted as evidence that a crossing might exist. The guard finally looks up.','Payment establishes a temporary relationship with physical reality. You cross.','The payment is taken. The map still refuses to recognize the bridge.')
 ],
 '03': [
  ('Credit Where It Is Due','The final bridge approach is obstructed. A huge plaque lists everyone responsible for the improvement. The workers are trying to use it to cover a hole.','You make it across. The plaque finally supports something besides a career.','You take the diversion. The plaque has more direct access to D.C. than you do.','Your permit opens the approved route. Somebody moves a sponsor’s name out of the way.','Payment secures passage. Your contribution will not fit on the plaque.','The payment is collected, but the crossing stays blocked. The plaque offers no contact details.'),
  ('The Photo Finish','The final bridge’s reopening photos are complete. The road surface isn’t. A worker asks you to wait while an official gets one more shot of himself finishing the job.','You pass after the crossing clears. The official is still deciding which photograph looks most practical.','You take the diversion. The reopening has gone very well for everyone who arrived with a photographer.','Your permit is accepted. The van enters the picture without sponsoring it.','Payment secures the crossing. Nobody asks you to smile for the receipt.','Payment is taken. The official’s photographer asks you to move out of frame.'),
  ('The Listening Exercise','The final bridge obstruction has a public-feedback box. It’s on the far side. A worker says this has produced excellent satisfaction figures.','You get across. The feedback box may now have a problem.','You divert around the obstruction. Another complaint fails to reach the box.','Your permit grants passage. You have met the entrance requirements for an opinion.','Payment secures passage. Having an opinion turns out to require a processing fee.','Payment is taken, but you cannot cross. Satisfaction remains suspiciously high.')
 ]
}
for family,rows in crossings.items():
    for v,row in zip('ABC',rows):
        title,setup,passed,detour,permit,bribe_ok,bribe_bad=row
        outcomes={'passage':passed,'diversion':detour,'permit_receipt':permit+' One Receipt is consumed.','permit_tag':permit+' Your permit tag remains available.','bribe_success':bribe_ok,'bribe_failure':bribe_bad}
        if family=='01':outcomes['refused_passage']='Passage is refused, so you take the detour. The journey continues; this checkpoint does not end the expedition.'
        else:outcomes['terminal_failure']='The crossing failure ends this journey here. The record does not establish that anyone died.'
        add(f'CROSS-{family}-{v}',title=title,setup=setup,outcomes=outcomes,
            mechanics='Journal/resolution text only. Use the actual crossing outcome, applied payment, permit type, detour cost and terminal state. Do not add a choice screen or new costs.',
            cast_requirements='Current van occupants only. Officers/workers are fictional NPCs. Do not name a universal town for this milestone.')

# The older brief lists six legacy data names. These eighteen editorial IDs are
# retained, but their integration targets follow the actual ExecOrder enum.
orders = [
 ('SHUTDOWN','shutdown','Government Shutdown',[
  ('The government is closed. Your bills have elected to remain open.','The shutdown ends. Public services return to the exhausting business of existing.'),
  ('The closure notice says essential work will continue. Apparently the work is essential and the people doing it are a negotiating technique.','The doors reopen. The closure notice comes down without reimbursing anyone for staring at it.'),
  ('An official announces the shutdown beside six microphones. None of those departments appears to have lost power.','The shutdown expires. The microphones are already explaining what an achievement reopening was.')],
  'While active, lose the displayed supply and morale amounts at the start of each game day, including rest days.'),
 ('MILITARIZE','travel_ban_lite','Travel Ban Lite',[
  ('A new order restricts movement. The spokesperson assures everyone it’s only a temporary loss of where they were going.','The travel restriction expires. Roads resume their controversial connection to other places.'),
  ('The government wants to know why ordinary people keep going places. Officials will discuss this at an overseas retreat.','The restriction lifts. You no longer need to contemplate your destination quite so slowly.'),
  ('The new travel policy promises a safer journey by making less of it happen.','The order expires. More of each travel day can once again contain travel.')],
  'While active, lose the displayed sanity amount each day. Travel distance uses the displayed reduced multiplier before daily limits.'),
 ('GAG','book_panic','Book Panic',[
  ('Officials are investigating dangerous books. The first witness admits reading the cover. Under pressure, he revises that to seeing a picture of it.','The panic order expires. Books return to threatening people who open them.'),
  ('A new bulletin warns that reading may expose citizens to unfamiliar ideas. There is no warning about the familiar ones that produced the bulletin.','The order expires. You may once again meet an idea without checking whether it brought identification.'),
  ('The banned-book committee has discovered an index. It is appalled that the suspicious words are organized.','The bulletin expires. Alphabetical order survives another attempt to confuse it with an agenda.')],
  'While active, the displayed sanity cost applies only below the morale protection threshold shown by the game.'),
 ('TARIFFS','tariff_tsunami','Tariff Tsunami',[
  ('The tariff bulletin promises foreign countries will pay. The shopkeeper turns the card reader toward you with worrying confidence.','The tariff order expires. This removes its game penalty; it does not make the shopkeeper apologize.'),
  ('The government has declared a trade victory. Your shopping bag looks as though it lost.','The order expires. The daily supply penalty ends; your previous expenses remain extremely historical.'),
  ('New tariffs are being sold as a negotiating masterstroke. At the shop, your wallet has been appointed chief negotiator.','The tariff order ends. Your wallet resigns its diplomatic post.')],
  'While active, lose the displayed supplies each day unless your carried legal fund provides protection.'),
 ('TAXCUTS','doe_eliminated','Education Department Eliminated',[
  ('In this simulated order, the education department is eliminated. The announcement tells students to discover the future for themselves.','The order expires. The morale penalty ends; the announcement still needs someone to mark its work.'),
  ('The education office has been replaced by a notice about self-reliance. The notice offers no instructions for reading it.','The order ends. The game’s education-related morale penalty stops.'),
  ('The new education policy is written in the past tense. A teacher offers to correct it during her other job.','The order expires. The morale cost stops, leaving the teacher to explain what a temporary decision means.')],
  'While active, lose the displayed morale amount at the start of each game day.'),
 ('DEREGULATE','war_dept_reorg','War Department Reorganization',[
  ('The government reorganizes its machinery. Your van responds by making a noise that was not included in the organizational chart.','The reorganization order expires. Its extra breakdown risk ends; existing damage still needs repair.'),
  ('A department has changed names, badges and procurement procedures. Your van’s parts would prefer someone changed the parts.','The order ends. The extra breakdown risk disappears. No one has rebuilt your van during the announcement.'),
  ('The reorganization promises a leaner machine. From under the bonnet comes a sound that suggests yours has taken the advice personally.','The order expires. The added breakdown risk ends, without awarding you a replacement vehicle.')],
  'While active, apply the displayed bonus to breakdown risk before the game’s risk limits. This does not itself trigger or repair a breakdown.')
]
for old,key,title,rows,effect in orders:
    for v,(activation,expiration) in zip('ABC',rows):
        add(f'ORDER-{old}-{v}',title=title,runtime_key=key,activation=activation,effect=effect,expiration=expiration,
          integration_note=f'Editorial slot retained from the proposed brief; target is current ExecOrder::{key}. Effects render from game data, not joke text.',
          evidence_status='Explicitly simulated executive order, not an assertion that the fictional announcement occurred in history.',
          cast_requirements='May show a radio, blank bulletin screen or fictional briefing. Do not imply the crew is physically in the briefing room.')

repairs = {
 'TIRE': [
  ('The Lifetime Guarantee','A tire gives out beside a billboard promising products built to last a lifetime. Judging by the rubber on the road, the billboard meant a moth.', 'The spare tire is fitted. Its modest ambition to remain round proves useful.','A replacement tire is fitted at the town shop. The bill is less circular than the warranty argument.','The replacement tire arrives by roadside delivery and is fitted. Delivery cost is included in the displayed price.','You trade supplies for a used tire and get it fitted. It has experience doing the whole job.','Radio contacts arrange work for a used tire. Work completed, tire fitted: the van is moving on round objects again.'),
  ('The Dramatic Exit','The tire’s tread leaves without giving notice. The rest of the van has become stationary management.', 'You fit the onboard spare. A replacement has actually arrived before the meeting about the vacancy.','The shop fits a bought replacement. No motivational speech is required to make it turn.','A roadside replacement is delivered and fitted. The displayed price includes bringing the tire to you.','Supplies traded, used tire fitted. The wheel is fully staffed again.','You complete the radio-arranged work and the used tire is fitted. The labor shortage was solved with labor.'),
  ('Air as a Service','A tire goes flat. Your crew searches the roadside for an affordable way to continue. Somewhere, someone is undoubtedly pitching a subscription for keeping air inside things.', 'Your spare is fitted. The van enjoys another period of privately owned air.','You buy a tire at the shop and it is fitted. The receipt does not renew automatically.','The new tire is delivered to the stranded van and fitted. Roadside delivery is part of the quoted cost.','You exchange supplies for a used tire and fit it. Air remains included.','Work arranged over local radio earns a fitted used tire. No cash or spare is required for this route.')
 ],
 'BATTERY': [
  ('The Last Click','The battery dies. The ignition clicks like a manager pretending to type during a pay discussion.', 'You fit the spare battery. The van finally gives an answer other than clicking.','The town shop fits the battery you bought. The starter returns to participating.','A replacement battery is delivered and fitted by the roadside. The displayed quote includes delivery.','Supplies are traded for a used battery, which is fitted. It has more energy than the sales pitch.','Radio contacts exchange a fitted used battery for your completed work. The van has a route back without a credit card.'),
  ('Battery Not Included','The dashboard goes dark. Your phone still has enough charge to show an advert for a luxury electric SUV. It’s offering to change your life from the wrong vehicle.', 'Your spare battery is fitted. The dashboard returns without asking you to upgrade your lifestyle.','You buy a battery at the shop and it is fitted. Your existing van is allowed to continue being yours.','The bought battery reaches you by roadside delivery and is fitted. The delivery fee is already in the price.','Supplies exchanged, used battery installed. Nobody has improved your personal brand.','Your completed radio-arranged job pays for a fitted used battery. The luxury SUV advert can wait.'),
  ('Low Power Mode','The battery gives up. Someone suggests restarting the van, then notices that this is precisely the service the battery has withdrawn.', 'You fit the carried spare. Restarting now becomes a less philosophical suggestion.','The town shop installs the replacement you bought. The engine starts taking instructions again.','A replacement is delivered and fitted at the roadside. The quote includes the journey your battery made.','You barter supplies for a used battery and have it fitted. A practical answer defeats the brainstorming session.','Work arranged through local radio is completed; a used battery is fitted. The van returns to the conversation.')
 ],
 'ALTERNATOR': [
  ('The Team Player','The alternator fails. The battery has been doing its own job and someone else’s. Your crew recognizes the arrangement immediately.', 'The spare alternator is fitted. The battery is allowed to stop covering the entire department.','The town shop fits a replacement alternator you bought. Charging resumes without a resilience seminar.','Your bought alternator is delivered and fitted at the roadside. Delivery is included in the displayed price.','You trade supplies for a used alternator and fit it. The battery’s unpaid promotion ends.','You complete radio-arranged work for a fitted used alternator. The battery finally gets a colleague who does something.'),
  ('We Have Tried Enthusiasm','The alternator stops charging the battery. Someone pats the dashboard encouragingly. The warning light remains unmoved by recognition.', 'Your spare alternator is fitted. The battery accepts electricity in place of praise.','A replacement bought at the shop is fitted. The charging system prefers this to another pat.','The replacement is delivered to the roadside and fitted. The full quote includes delivery.','Supplies buy a used alternator through barter; it is fitted. Recognition is upgraded to actual support.','Radio contacts arrange work for a fitted used alternator. You finish the work; the van receives something more useful than encouragement.'),
  ('The Charging Question','The alternator fails while the crew argues about which gadgets need charging. The van has entered the discussion with a veto.', 'Your spare is installed. The charging argument returns to phones, where it belongs.','The town shop fits the replacement you purchase. The van withdraws its veto.','Roadside delivery brings the replacement; it is fitted. The displayed price covers delivery as well as the part.','Supplies traded, used alternator fitted. The largest device in the group is back online.','You finish the work arranged over radio and receive a fitted used alternator. The van’s request is finally processed.')
 ],
 'FUELPUMP': [
  ('Fuel with No Benefits','The fuel pump fails. There is fuel in the tank, but it isn’t reaching the engine. The van has accidentally modeled trickle-down economics.', 'Your spare fuel pump is fitted. Resources begin reaching the part that does the work.','The town shop fits the replacement you buy. The engine receives its first meaningful distribution.','A replacement pump is delivered and fitted at the roadside. The displayed price includes delivery.','You trade supplies for a used pump and get it fitted. The fuel stops waiting for prosperity to trickle down.','Work arranged through local radio earns a fitted used pump. The fuel is finally circulated.'),
  ('Supply-Side Problem','The fuel pump stops. The dashboard reports fuel. The engine reports no fuel. You have enough experience with official figures to believe both are technically accurate.', 'The onboard spare pump is fitted. The dashboard and engine reach a working agreement.','The town shop installs the replacement you purchase. The fuel figure becomes useful to the engine.','Roadside delivery brings the bought pump, which is fitted. Its delivery charge is included in the quote.','Supplies are traded for a used pump and it is fitted. The official fuel figure now has practical consequences.','You finish radio-arranged work and the used pump is fitted. Two departments finally share a resource.'),
  ('The Last Middleman','The fuel pump fails and the engine stops receiving fuel. A very small middleman has brought the entire trip to a halt. Naturally, replacing it costs money.', 'You fit the pump you already carry. The middleman is replaced by someone in the luggage.','You buy a pump at the town shop and get it fitted. The engine resumes its relationship with the tank.','A bought pump is delivered and fitted beside the road. The quoted cost includes delivery.','You barter supplies for a used pump and have it fitted. The new middleman accepts groceries indirectly.','A radio contact arranges work for a fitted used pump. You complete the exchange without cash or an onboard spare.')
 ]
}
for part,rows in repairs.items():
    for v,row in zip('ABC',rows):
        title,setup,spare,town,road,barter,radio=row
        add(f'REPAIR-{part}-{v}',title=title,setup=setup,
          choices=[{'label':'Fit the onboard spare','outcome':spare,'availability':'Requires the matching spare; consumes it; one hour.'},
                   {'label':'Buy and fit a replacement','outcome':town,'roadside_outcome':road,'availability':'Requires the displayed cash; ninety minutes. Roadside quote includes delivery.'},
                   {'label':'Barter supplies for repairs','outcome':barter,'availability':'Requires and consumes four supplies; two hours.'},
                   {'label':'Radio for work and repairs','outcome':radio,'availability':'No cash or spare needed; four hours, displayed sanity and morale costs.'}],
          outcomes={'not_completed':'The repair has not been completed. The van remains broken and no success should be displayed.','cashless':'No cash or spare? Local radio contacts can arrange work in exchange for a used part and repair.'},
          mechanics='Use repair success only after the game accepts the action. The work option improves vehicle health less than other paths; display the actual recovery and remaining damage.',
          integration_note='Existing part-specific repair surface. Purchase needs distinct in-town and roadside results.')

activities={
 'FORAGE':[
  ('Locally Sourced','A local guide offers to show you what can safely be gathered nearby. The boutique over the road sells the same idea in a basket with a personal origin story.','Follow the guide','You return with food. It has managed to be artisanal without employing a publicist.'),
  ('The Qualified Leaf','At your roadside stop, a guide points out safe food plants. Your crew’s contribution to the expertise is not eating the first interesting leaf.','Gather with the guide','Food gathered, nerves steadier. Everyone leaves the interesting leaf to develop its career independently.'),
  ('Premium Wild','A guide offers to help you gather food safely. A wellness tourist asks whether it’s organic. The guide gestures at the absence of a factory.','Join the guided forage','You gather food and clear your heads. The tourist is still asking where the label goes.')],
 'GLEAN':[
  ('Cosmetically Challenged','A farmer lets your crew glean produce the buyer rejected for looking wrong. One carrot appears to be giving the grading standards the finger.','Glean the field','You take home the agreed share, aching from the work. The carrots remain ugly enough to eat.'),
  ('The Wrong Size','A farmer offers a share for helping gather rejected produce. The supermarket wants matching potatoes. Nobody has explained the dress code to a potato.','Help gather the crop','Your supplies grow; the lifting costs health. The potatoes have escaped a difficult career in modeling.'),
  ('The Harvest Surplus','The roadside farm has edible surplus and permission to glean it. A consultant calls this inefficient allocation. The farmer hands you a crate and allocates efficiently.','Glean with permission','You finish the heavy work and pack your share. The consultant has yet to lift a concept.')],
 'FOODWORK':[
  ('Enough for the Photograph','At a town pantry, the coordinator offers a food parcel for three hours sorting donations. A sponsor wants all its labels facing the camera.','Sort donations for food','You receive the agreed food parcel. Hunger has been temporarily defeated; brand visibility was never in danger.'),
  ('The Board Meeting','A town pantry offers food for sorting its donations. The visiting board is discussing community resilience around a table that someone else stocked.','Take the sorting shift','The work is done and your food parcel is packed. The board votes to thank the table.'),
  ('Goods and Services','A donation center offers the crew food for three hours of sorting. A consultant suggests replacing the offer with points. The cook tells him points take too long to boil.','Work for the food parcel','You finish sorting and collect actual food. The points scheme remains courageously inedible.')],
 'CASHWORK':[
  ('Brief Employment','A town appliance shop offers the crew a three-hour unloading shift for $18. The manager calls this an opportunity to join the family. Nobody asks to be adopted.','Take the unloading shift','You finish, collect $18, and lose a little patience. The family reunion lasted exactly as long as agreed.'),
  ('The Self-Made Shipment','A shop offers $18 to unload motivational books for three hours. Every author apparently did it all alone. None has come to carry a box.','Unload for the agreed pay','The shift ends and the crew’s cash rises by $18. The authors remain entirely self-made in the storeroom.'),
  ('Fragile Leadership','The town’s office-supply shop offers $18 for unloading new executive chairs. They have lumbar support, heated seats and a longer warranty than the job.','Take the three-hour job','Chairs delivered, $18 collected, patience reduced. Management’s backs are covered.')],
 'BARTERTIRE':[
  ('The Round Economy','At the community exchange, a resident offers a spare tire for supplies. It is round, tangible and difficult to describe as a revolutionary digital asset.','Trade supplies for a tire','The agreed supplies are exchanged for the tire. Your investment has a useful hole in the middle.'),
  ('Still Included','A resident at the town exchange offers a spare tire for supplies. The tread and sidewall are included in the same transaction.','Make the tire trade','Supplies traded, spare packed. There is no premium tier of round.'),
  ('Local Currency','The community exchange will trade a spare tire for supplies. A visiting entrepreneur asks who owns the platform. A resident points at the table.','Exchange supplies for the tire','The trade is complete. The table takes no commission.')],
 'BARTERBATTERY':[
  ('Stored Energy','At the community exchange, someone offers a spare battery for supplies. For once, stored energy comes with something more convincing than a pitch deck.','Trade supplies for a battery','The supplies are exchanged and the spare battery is packed. Its power is electrical, a refreshing limitation.'),
  ('The Useful Brick','A resident offers a spare vehicle battery for supplies. Your crew inspects it with more interest than the luxury apartments advertised behind the stall.','Make the battery trade','Trade completed. You have acquired a heavy box that might actually get you somewhere.'),
  ('No Monthly Charge','The exchange offers a spare battery for supplies. Someone asks about monthly charges. The resident says that’s usually the alternator’s problem.','Exchange supplies for the battery','The battery is yours and the supplies are exchanged. No account needs to be created.')],
 'BARTERSUPPLIES':[
  ('Eating the Spare','At the community exchange, a resident offers supplies for a spare tire. The tire isn’t edible, despite how closely the crew has examined its possibilities.','Trade a tire for supplies','The spare leaves your inventory; the agreed food and supplies arrive. Dinner has escaped an experimental phase.'),
  ('A Liquid Asset','The town exchange can turn your spare tire into supplies. Your crew prefers this definition of liquidity to licking condensation off the cooler.','Exchange the spare for supplies','The tire is traded and supplies are packed. Everyone’s investment outlook becomes less circular.'),
  ('Portfolio Rebalancing','At the exchange, you can trade a spare tire for supplies. The crew is overinvested in rubber and underinvested in lunch.','Make the supplies trade','Trade completed. The pantry gains what the spare-parts stash loses. Lunch approves the restructuring.')],
 'REST':[
  ('The Stationary Protest','The crew can take a full day to rest. A productivity podcast calls stopping a failure of ambition. You stop the podcast first.','Take a full rest day','The day passes, recovery applies and the crew’s care strain clears. The podcast has survived without your labor.'),
  ('Out of Office','The crew can spend a day resting. Someone suggests using the time to catch up on everything. Nobody gives that person the notebook.','Rest for the day','The rest day is complete. Health and sanity recover within their limits, and nobody has submitted a progress report.'),
  ('Return on Inactivity','A full rest day is available. The crew considers its revolutionary promise: being tired is sufficient grounds for doing less.','Settle in and rest','You complete the rest day and clear ongoing care strain. Nothing has been optimized except the seating position.')]
}
rules={
 'FORAGE':'Two hours; supplies +2, sanity up to +1. Roadside only; shares a three-day cooldown with gleaning; requires supplies at or below18.',
 'GLEAN':'Two hours; supplies +4, health −1. Roadside only; shared three-day gathering cooldown; requires supplies at or below16 and health above1.',
 'FOODWORK':'Three hours; supplies +4, sanity −1. In town; once per stop shared with cash work; requires supplies at or below16 and positive sanity.',
 'CASHWORK':'Three hours; cash +$18, sanity −1. In town; once per stop shared with food work; requires positive sanity.',
 'BARTERTIRE':'Use current exchange cost/reward and eligibility; once-per-stop community exchange. No tire fitted automatically.',
 'BARTERBATTERY':'Use current exchange cost/reward and eligibility; once-per-stop community exchange. No battery fitted automatically.',
 'BARTERSUPPLIES':'Use current exchange cost/reward and eligibility; once-per-stop community exchange. Consume only the spare traded.',
 'REST':'One day; configured health/sanity recovery, up to one available supply consumed, two-day rest cooldown; clears care strain and ordinary illness penalty. Other daily effects still settle; unavailable during breakdown.'}
for family,rows in activities.items():
    for v,(title,setup,action,outcome) in zip('ABC',rows):
        add(f'ACT-{family}-{v}',title=title,setup=setup,action=action,outcome=outcome,mechanics=rules[family],unavailable='This action is not available in the current state. Show the game’s actual reason and costs before an action; do not display completion.',cast_requirements='Show only current active travelers; a named local worker or guide is an NPC, not a new crew member.')

conditions={
 'CLEAR':[
 ('Unexpected Cooperation','Clear skies. The weather is providing a basic service without asking whether you deserve it.','The sky remains clear. Nobody has sold you an upgrade to the horizon.','The clear spell has ended; check the new conditions before traveling.'),
 ('No Comment from the Sky','The clouds part. For once, an improvement occurs without a politician standing in front of it.','The clear weather holds. Visibility continues without a ribbon-cutting.','Conditions change. The new weather effects apply as shown.'),
 ('Standard Daylight','A clear stretch opens ahead. Daylight is included, even for travelers on the basic plan.','The road stays visible. The sun has not yet introduced a subscriber queue.','The clear spell gives way to new weather. Check its actual effects.')],
 'STORM':[
 ('The Unscheduled Wash','Rain lashes the van. A roadside car-wash advert continues promising something you are receiving against your will.','The storm continues. The van is cleanest where you least need water.','The storm has passed. Its active weather effects end; previous costs remain in the journey record.'),
 ('Outdoor Seating','A storm breaks over the road. Every picnic table becomes an ambitious small pond.','The storm holds. The outdoor seating is now technically waterfront.','The storm clears. The tables can stop competing with the river.'),
 ('Real-Time Feedback','Rain drums on the roof. The van’s insulation handles complaints by transmitting them to everyone inside.','The storm keeps filing its complaint. The roof has no mute button.','The rain eases and the storm ends. The roof’s customer-service shift is over.')],
 'HEAT':[
 ('An Unpaid Sauna','A heat wave settles over the route. The van is offering a sauna experience nobody booked.','The heat continues. Upholstery has become an aggressive form of physical contact.','The heat wave ends. The seats resume being furniture.'),
 ('Melted Optimism','The heat arrives. A roadside billboard advertises a cool, refreshing escape at a price the crew finds less refreshing.','The heat holds. Every shaded parking space is looking like prime real estate.','The heat breaks. Shade can go back to being part of a tree.'),
 ('The Dashboard Forecast','A heat wave turns the dashboard into a surface you respect from a distance. The forecast remains confident that this is weather.','The heat persists. Nobody puts an elbow down without negotiation.','The heat wave ends. The dashboard stops threatening to cook the map.')],
 'COLD':[
 ('Involuntary Togetherness','A cold snap hits. The van’s occupants discover a sudden interest in sitting closer without discussing feelings.','The cold continues. Every draft has found someone personally.','The cold snap ends. Personal space becomes affordable again.'),
 ('The Blanket Economy','Cold weather arrives. A spare blanket is briefly valued more highly than the entire vehicle.','The cold holds. Blanket ownership remains the subject of active diplomacy.','The cold eases. The blanket can retire from foreign affairs.'),
 ('Winter Included','The temperature drops. Your breath becomes visible, providing a free demonstration that the cabin is not adequately impressed by the heater.','The cold stays. You can still see every disappointed exhale.','The cold snap passes. Breathing returns to its less theatrical setting.')],
 'SMOKE':[
 ('The Smell of Progress','Smoke hangs over the route. The roadside billboard promises fresh opportunities. The air is providing something else.','Smoke persists. The horizon is still hiding behind the marketing.','The smoke clears. Its active weather effects end; this does not erase costs already applied.'),
 ('Scenic Route, Currently Missing','Smoke obscures the view. The scenic-route sign is making a promise it can’t currently display.','The smoke remains. Scenery continues to exist somewhere behind it.','The air clears enough for the smoke condition to end. The view becomes available again.'),
 ('Smoked Everything','Smoke drifts across the road. A barbecue advert becomes much less specific about what is being smoked.','The smoky conditions persist. The entire region still appears to be on the menu.','The smoke condition clears. The barbecue advert returns to advertising dinner.')],
 'ILLNESS':[
 ('The Unrequested Souvenir','Ordinary illness hits the crew’s journey. Nobody recalls buying the souvenir, but something has come along for the ride.','The illness persists. The souvenir has poor reviews and no return address.','The ordinary illness penalty clears. This does not by itself revive anyone or resolve a separately pending care incident.'),
 ('Sick-Day Culture','An illness slows the trip. The van has a sick-leave policy considerably more advanced than pretending nobody is sick.','The illness continues. Nobody is awarded employee of the month for coughing enthusiastically.','The ordinary illness clears. The game removes its penalty; other conditions still follow their own rules.'),
 ('Unexpected Passenger','Ordinary illness joins the trip without contributing gas money. It has already made itself unpopular.','The illness hangs on. Its invitation was apparently open-ended.','The ordinary illness penalty ends. No extra person has entered or left the crew.')],
 'HUNGER':[
 ('The Menu in Your Head','Food is running short and hunger is taking a toll. Someone describes a sandwich so carefully it feels like a personal attack.','The hunger continues. Describing the sandwich has added no supplies.','The hunger condition is relieved when the game records adequate food. Previous losses are not automatically restored.'),
 ('Empty-Shelf Expertise','Hunger is affecting the crew. You have become very good at finding the back of the food box.','The hunger persists. The empty box offers no further insights.','The food shortage is relieved. Eating becomes an activity again, subject to the resources actually gained.'),
 ('Food for Thought','Hunger is costing health. A roadside advert says success begins with a positive mindset. The crew would prefer bread.','The hunger continues. The positive mindset still has no nutritional information.','The hunger condition ends once the game records relief. The advert receives none of the credit.')],
 'HEATEXPOSURE':[
 ('No Longer Just Uncomfortable','Prolonged heat without protection is now causing harm. The van’s unwanted sauna has started collecting more than patience.','Heat exposure continues to damage health. The current protection and costs are shown in the weather panel.','Protection or changed conditions ends the exposure penalty when the game records it. Lost health does not return automatically.'),
 ('The Shade Gap','Unprotected heat is now taking a physical toll. The nearest luxury-resort advert contains more shade than the road.','The heat exposure continues. Looking at the resort advert has provided none of its air-conditioning.','The game records relief from heat exposure. The resort advert remains unaffordable and medically uninvolved.'),
 ('Health Is Not a Forecast','Heat exposure is causing actual damage. This has moved beyond being something to complain about at the next stop.','The harmful heat exposure continues. The game’s displayed risk is now a recorded cost.','The exposure penalty stops once protection or conditions provide relief. Check remaining health before continuing.')],
 'COLDEXPOSURE':[
 ('The Cold Bill','Prolonged cold without protection is now damaging health. The free winter experience has found a way to charge you.','Cold exposure continues to take a physical toll. Warm-coat protection and current costs are shown by the game.','The exposure penalty ends when protection or weather provides relief. Existing health loss remains.'),
 ('Outside Has Come Inside','Cold exposure is causing harm. The van’s insulation appears to have outsourced its job to your clothing.','Unprotected cold continues causing damage. The outsourcing arrangement has not improved.','The cold-exposure penalty stops when the game records protection or warmer conditions. Recovery follows the actual health rules.'),
 ('An Unhelpful Thermometer','Cold exposure is damaging health. The thermometer is very accurate and, at this point, contributes little else.','The cold exposure persists. Accuracy has not made the cabin warmer.','The harmful exposure ends once the game records relief. The thermometer is not credited with a rescue.')]
}
for family,rows in conditions.items():
    for v,(title,onset,continuing,relief) in zip('ABC',rows):
        add(f'COND-{family}-{v}',title=title,onset=onset,continuing=continuing,recovery=relief,
            mechanics='Display only on the matching existing condition surface. Weather onset alone is not confirmed injury. Bind actual costs, modifiers and gear protections to the game; never add a new interrupting modal.',
            cast_requirements='Use active crew only; no individual illness/death assignment unless the game identifies that person.')

openings={
 'JOURNALIST':[
  ('Seattle: On the Record','You leave Seattle with five companions, a notebook, and a van containing more receipts than money. Your editor wanted the human angle. It has asked for fuel.','Head for D.C.','Six ordinary travelers set out to make their case. You check the facts; the van supplies the noises.'),
  ('Seattle: Access Journalism','From Seattle, you and five companions set off for D.C. You have questions for powerful people. Powerful people have publicists. You have a cooler that won’t close.','Start the journey','You head east with the crew. Nobody has offered exclusive access to the petrol budget.'),
  ('Seattle: Essential Equipment','The Seattle departure checklist contains press credentials, road supplies and five people reminding you that “a great story” isn’t a reimbursement policy.','Get on the road','The six of you leave for D.C. Your notebook is open; your expense account remains imaginary.')],
 'ORGANIZER':[
  ('Portland: Everyone Fits','In Portland, you fit five companions and the supplies into the van. Organizing people is your specialty. Getting them to agree on which bag is blocking the door may be the advanced course.','Leave for D.C.','The six of you set out. The government will hear your case if the snack committee releases the evidence folder.'),
  ('Portland: A Collective Vehicle','You’re leaving Portland to make your case in D.C. Five companions have joined. Everyone believes in collective action; nobody believes their bag belongs in the awkward corner.','Begin the trip','You leave with six travelers and a workable luggage compromise. Democracy has cleared the first hinge.'),
  ('Portland: A Show of Hands','In Portland, you ask who’s ready to argue with the government. Five hands go up. Then you ask who packed the can opener.','Start toward D.C.','Six of you head east. The movement has strong principles and a developing relationship with tins.')],
 'WHISTLEBLOWER':[
  ('San Francisco: Backup Copies','You leave San Francisco with five companions and evidence the people in charge would rather lose. You’ve made backups. The van would like you to apply that philosophy to its parts.','Drive toward D.C.','The six of you start the trip. Your files are secured more thoroughly than the cup holder.'),
  ('San Francisco: The Wrong Attachment','You found what the official story left out. Now you and five companions are leaving San Francisco to make a case in D.C. Your luggage has been searched three times by the person looking for snacks.','Begin the journey','The evidence and all six travelers set out. The snacks have a separate chain of custody.'),
  ('San Francisco: Internal Channels','You tried the proper channels. They led to a meeting about why you used the proper channels. Now you and five companions are leaving San Francisco in a van with a simpler route.','Set out for D.C.','Six people head east with your evidence. The van has fewer procedures and more warning lights.')],
 'LOBBYIST':[
  ('Los Angeles: Modest Interests','You know how lobbying works. Unfortunately, your clients are ordinary people and your office is a van leaving Los Angeles with five companions. The leather seating is peeling.','Head for D.C.','Six of you begin the trip. You’ll make the case for people who cannot expense a golf course.'),
  ('Los Angeles: Access on a Budget','In Los Angeles, you load a van for D.C. Your lobbying contacts may help; your five companions would also appreciate contacts who own jumper cables.','Start the journey','The crew sets out to argue its case. Your influence is real enough to try, too small to fuel the van.'),
  ('Los Angeles: The Small Client','You leave Los Angeles representing people whose main interest is being able to live. Five companions join you. D.C. generally prefers a more specific invoice.','Set off east','Six ordinary travelers hit the road. You prepare to explain why being alive should count as a sector.')],
 'STAFFER':[
  ('Sacramento: Other Duties','You’re leaving Sacramento with five companions to make your case in D.C. As a staffer, you know how government runs. Someone else gets the photograph; you find the charger.','Leave for D.C.','The six of you set out. There is nobody senior enough to make the van’s noises your responsibility alone.'),
  ('Sacramento: Essential Support','In Sacramento, you load the trip’s papers while five companions load the van. You’ve spent a career keeping meetings functioning. This one has an engine and fewer excuses.','Begin the trip','Six travelers head toward D.C. The first order of business is getting beyond the car park.'),
  ('Sacramento: The Agenda','You prepare the departure checklist in Sacramento. Five companions add urgent amendments concerning snacks. Years of government work have prepared you for this level of constitutional crisis.','Get underway','The six of you set off to make your case. The snack amendments pass without a donor reception.')],
 'SATIRIST':[
  ('San Diego: Difficult Competition','You’re a satirist leaving San Diego with five companions to argue your case in D.C. The government keeps stealing your best ideas and implementing them.','Start the trip','Six of you head east. Your material is plentiful; everything else has to fit in the van.'),
  ('San Diego: The Straight Man','In San Diego, you pack for D.C. Five companions expect you to keep spirits up. The van expects you to explain why that does not cover its operating costs.','Leave for D.C.','You set out together. The government supplies the premise, and the crew has been living through the punchline.'),
  ('San Diego: No Laugh Track','You and five companions leave San Diego with a case for D.C. You’ve brought jokes for the journey. The others have established that “we’re doomed” is not six different jokes.','Get on the road','The six of you start traveling. You aim for better material and, ideally, a functioning arrival.')]
}
for persona,rows in openings.items():
    for v,(title,setup,action,outcome) in zip('ABC',rows):
        add(f'OPEN-{persona}-{v}',title=title,setup=setup,action=action,outcome=outcome,
          cast_requirements='Departure only: player plus five named companions. Use selected persona and correct origin. No extra staff, aides or official authority.',
          integration_note='Proposed departure narrative surface; does not change persona bonuses, starting cash, supplies, route or character names.')

hearings={
 'C':[
  ('Please State Your Name','You reach the D.C. hearing. The chair asks you to explain your circumstances briefly. His introduction has already lasted longer than your last paid break.', ['The opening procedure uses another stretch of your stamina. The chair thanks himself for keeping things moving.','Points of order follow. You discover how many ways a room can say “not yet.”','The final procedural round arrives. Your evidence has had more time in the room than the chair’s attention.'],'The procedural rounds are over. One final vote will decide the result. The raised hands will do what the speeches have avoided.'),
  ('Three Minutes for the Public','At the D.C. hearing, you take your place behind a microphone. The chair checks the clock with the concern of a man who has never been paid by it.', ['The first round tests your stamina. The chair reads the instruction to keep things concise at remarkable length.','The next round turns to procedure. Your case remains present, technically.','The last round consumes another share of your remaining stamina. Somebody requests that a request be recognized.'],'Procedure is complete. The hearing now reaches its single final vote. Your case is finally required to receive an answer.'),
  ('On Behalf of the People','At the D.C. hearing, the chair welcomes ordinary citizens. He pronounces it with the mild surprise of someone finding them in the building.', ['The first stamina round begins. The chair explains how much the institution values public time.','Another round passes through your reserves. Several officials contribute by rearranging their papers.','You face the final procedural round. The microphone has listened consistently, which puts it ahead.'],'The rounds finish. The final vote is next. No result has been decided by the microphone.')],
 'D':[
  ('The Audience Participation Fee','You reach the D.C. hearing. A screen behind the chair welcomes public participation. Beneath it, an aide quietly turns your microphone away from your mouth.', ['The first stamina round begins. The chair reads the public-participation promise while looking at a donor’s empty seat.','The next round consumes more stamina. Your evidence is described as an interesting interpretation of your own circumstances.','The final round begins. The chair congratulates you on surviving an accessible process.'],'Procedure ends. One vote remains. The room must briefly replace its performance of listening with a result.'),
  ('The Official Account','At the D.C. hearing, the official summary is already printed. There is a blank space where your appearance will confirm that consultation took place.', ['The first round tests your stamina. A clerk prepares to file your presence as progress.','The next round asks whether your experience fits the approved description of your experience.','The final round arrives. The summary is waiting patiently for events to catch up with it.'],'The stamina rounds finish. The final vote is still to come; the printed summary is not the outcome.'),
  ('Welcome to the Listening Room','You enter the D.C. hearing. Behind the chair is a huge portrait of someone listening thoughtfully. The room appears to have subcontracted that function to the portrait.', ['The first round drains stamina. The portrait maintains an exemplary expression.','The next round continues the procedure. No living official has yet outperformed the paint.','The final round tests what stamina you have left. The portrait is still excellent at its one job.'],'The procedural rounds are over. A single final vote will determine the result. The portrait gets no vote.')]
}
for mode,rows in hearings.items():
    for v,(title,setup,rounds,vote) in zip('ABC',rows):
        add(f'HEARING-{mode}-{v}',title=title,setup=setup,transitions=rounds,before_vote=vote,
          exhausted='You cannot finish the hearing’s stamina rounds. Do not show a final vote; use the actual recorded ending.',
          mechanics='Three configured stamina rounds, then one vote roll if the player endures. These lines narrate existing resolution; they do not add votes, hearings or choices.',cast_requirements='Use only travelers who actually reached D.C. Government aides here belong to the hearing officials, never the crew.')

endings={
 'VICTORY':[
  ('The Motion Carries','You win the hearing’s final vote. The chair calls it a triumph of the process, demonstrating the process’s ability to take credit even while losing the argument.'),
  ('Please Record This','The final vote goes your way. Your case becomes part of the official record, where it can no longer be dismissed as something said by people outside the room.'),
  ('A Difficult Precedent','You win the hearing vote. Ordinary people have crossed the country, made a case, and obtained a result. Several officials look concerned that this might catch on.')],
 'VOTEFAIL':[
  ('Thank You for Participating','You reach the final vote, but your case does not carry. The chair thanks you for participating. Participation appears to be the one thing the institution was prepared to give away.'),
  ('The Record Shows','The final vote is lost. Your journey and evidence remain part of what happened; the tally does not erase them. The chair is already congratulating the room on listening.'),
  ('Democracy Has Heard You','You lose the hearing’s final vote. The microphone is switched off with considerably more certainty than anyone displayed while answering your case.')],
 'SANITY':[
  ('No Reserves Left','The strain becomes too much to continue. Your journey ends here. A system that demands endless resilience has discovered the inconvenience of finite people.'),
  ('At Capacity','You have no sanity left to spend on the journey. The trip ends. Somewhere a motivational speaker is calling this a mindset problem from a chair someone else carried.'),
  ('Enough','Accumulated strain ends the expedition. The record keeps the distance you traveled and what actually happened. It does not award anyone a prize for being difficult to endure.')],
 'DESTROYED':[
  ('Beyond a Spare Part','The van is destroyed and the journey ends. This is no longer a missing spare or a repair choice. The vehicle’s remaining market value is chiefly a cautionary story.'),
  ('End of the Vehicle','The van cannot be restored through the available repair paths. Its destruction ends the expedition. The road has finally taken a larger share than the vehicle could give.'),
  ('No Trade-In Miracle','The destroyed van ends this journey. No replacement has arrived, no purchase has been made, and no salesperson’s positive attitude can be counted as transport.')],
 'COLD':[
  ('Cold Ends the Journey','Cold exposure ends the expedition. The record determines who remains and what happened to them. The weather has delivered consequences more directly than any office you tried to reach.'),
  ('Below What You Could Bear','Cold exposure is the recorded cause of this ending. The journey stops here. A slogan about toughing it out has supplied exactly as much warmth as ink.'),
  ('The Cold Limit','Exposure to cold ends the trip. The van’s roof was never a guarantee of protection. No official statement is going to raise the temperature in this result.')],
 'HEAT':[
  ('Heat Ends the Journey','Heat exposure ends the expedition. The recorded crew state stands. Calling it a beautiful day could not make it a safe one.'),
  ('Too Much Heat','The journey ends from heat exposure. A roadside advertisement still shows a smiling family enjoying the sunshine. Advertisements have exceptionally low water requirements.'),
  ('Beyond the Forecast','Heat exposure brings the trip to an end. The forecast described the temperature accurately. The journey required more than an accurate description of the problem.')],
 'COLLAPSE-HUNGER':[
  ('Food Was the Requirement','Hunger ends the expedition. Your case needed to reach D.C.; the people carrying it also needed to eat. The government’s website has yet to become edible.'),
  ('Nothing Left to Pack','The journey collapses from hunger. The food box is empty, not symbolic. No amount of official optimism has ever opened a tin that wasn’t there.'),
  ('The Cost of Continuing','Hunger brings the trip to an end. Your record preserves the actual journey and crew. An inspiring story about sacrifice cannot be substituted for lunch.')],
 'COLLAPSE-VEHICLE':[
  ('Transport Was Essential','Vehicle trouble causes the journey to collapse. The record does not necessarily describe a destroyed van; it describes an expedition that can no longer continue because of it.'),
  ('The Van Has the Last Word','The recorded vehicle-related collapse ends the trip. Whatever could be argued about the government, the van had become a more immediate opponent.'),
  ('No Further Service','Vehicle trouble ends this expedition. This outcome does not invent a collision, a death, or a replacement van. The actual breakdown of the plan is sufficient.')],
 'COLLAPSE-WEATHER':[
  ('Conditions Win','Weather causes the journey to collapse. The exact conditions belong to the trip record. The sky has declined to honor the itinerary.'),
  ('The Itinerary Loses','The expedition ends because of weather. Your carefully planned route remains a useful record of what the atmosphere did not agree to.'),
  ('Not Cleared for Continuing','Weather ends the journey. No extra storm or injury is added to the record. The conditions you actually faced have already done enough.')],
 'COLLAPSE-BREAKDOWN':[
  ('Stranded Too Long','An unresolved breakdown causes the journey to collapse. This is not a completed repair. The van has remained where the problem first asked to be taken seriously.'),
  ('The Repair That Didn’t Happen','The expedition ends with breakdown recorded as the cause. No spare was magically fitted and no imaginary mechanic arrived after the screen faded.'),
  ('A Stationary Ending','Breakdown ends this journey. The van’s refusal to continue outlasted the expedition’s ability to deal with it. The warning light has finally been promoted to the whole story.')],
 'COLLAPSE-DISEASE':[
  ('Illness Ends the Journey','Illness causes the expedition to collapse. The crew record determines who survived and whether a named traveler died. Their situation is not a wellness-product testimonial.'),
  ('The Body Has a Say','Disease ends this journey. The actual losses remain in the record; no additional death is invented. The people carrying the case were always more than its delivery system.'),
  ('No Miracle Ending','The expedition ends from disease. No miraculous recovery is added after the fact. The journey’s medical claims have already asked too much of the people living with them.')],
 'COLLAPSE-CROSSING':[
  ('The Road Does Not Continue','A terminal crossing failure ends the expedition. This is the failure recorded by the game, not a routine detour. The road ahead remains a direction rather than an available route.'),
  ('Passage Ends Here','The crossing failure ends the journey. No successful passage has occurred and no unseen rescue is assumed. Your case has encountered an obstacle that does not accept arguments.'),
  ('Beyond This Point','The expedition ends at the recorded terminal crossing. The destination remains elsewhere. A sign indicating a route has proved different from being allowed to take it.')],
 'COLLAPSE-PANIC':[
  ('The Limit Is Reached','Panic causes the journey to collapse. This describes the recorded ending, not a diagnosis of anyone playing. The expedition asked for more than could be carried this time.'),
  ('Unable to Continue','The trip ends with panic recorded as its cause. No extra loss is added to the crew record. An institution’s appetite for being endured is not a measure of your worth.'),
  ('The Journey Stops','Panic ends this expedition. Your actual distance, choices and crew remain in the record. No slogan about staying calm is going to rewrite them.')],
 'INCOMPLETE':[
  ('The Case Is Still Packed','This journey is unfinished or was left behind. Its record stops where you stopped. Nobody has secretly held the hearing while you were away.'),
  ('Not a Verdict','The expedition has no completed ending in this record. Leaving a trip unfinished is not a failed vote, a destroyed van, or evidence that anyone died.'),
  ('The Road Can Wait','This journey remains incomplete. Keep the record of what actually happened. The government is unlikely to run out of material while you decide what comes next.')],
 'FALLBACK':[
  ('The Available Record','This result does not identify a complete ending. The available journey details are shown as recorded. Even a game about government should resist inventing a successful process from missing paperwork.'),
  ('Outcome Not Recorded','The record does not establish how this journey ended. No vote, death or arrival is assumed. The missing information will not be promoted to a confident press release.'),
  ('What We Can Actually Say','A complete cause is unavailable in this result. Use the preserved journey facts and crew state. The rest is left unknown, an unusually restrained use of an official-looking screen.')]
}
for family,rows in endings.items():
    for v,(title,text) in zip('ABC',rows):
        add(f'END-{family}-{v}',title=title,headline=title,epilogue=text,
            cast_requirements='Show only the recorded survivors and actual vehicle state. A missing cause does not imply death, a crash, a completed hearing or D.C. arrival.',
            integration_note='Select by the exact typed ending. Incomplete and fallback are distinct. Use recorded location; outcome text intentionally does not invent one.')

save()
