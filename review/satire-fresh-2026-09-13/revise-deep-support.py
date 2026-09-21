import json
from pathlib import Path

ROOT=Path(__file__).resolve().parent
units=json.loads((ROOT/'support.json').read_text())
by={u['id']:u for u in units}
SOURCES={
 'emergency':('Emergency-abortion guidance and medical exceptions','CMS withdrew its abortion-specific emergency-care guidance in May 2025 while stating that EMTALA remains enforceable. Texas clarified in June 2025 that qualifying danger need not be imminent. This fictional hearing targets institutional delay and legal uncertainty, not a legal requirement to wait for a patient to be dying.','https://www.cms.gov/newsroom/press-releases/cms-statement-emergency-medical-treatment-labor-act-emtala'),
 'flock':('Flock database misuse to stalk women','An August 2026 records investigation documented an Alpharetta officer using Flock to monitor a former partner; he resigned while a criminal investigation remained open. This is officer misuse, not a finding that Flock employees stalked her or a criminal conviction.','https://www.wired.com/story/a-georgia-cop-used-flock-to-track-2-other-cops-his-ex-and-her-friend/'),
 'passport':('Federal passport sex-marker policy','State Department policy issues M/F markers matching its sex-at-birth policy and warns conflicting applications may be delayed. Existing passports are not automatically invalidated. This fictional checkpoint exaggerates a documentation conflict, not an actual domestic passport requirement.','https://travel.state.gov/en/passports/apply/unique-needs/sex-markers.html'),
 'detention':('Detention medical care beyond intake','GAO reported in June 2026 that a December 2025 review at Camp East Montana found missing comprehensive assessments and treatment plans for people with diabetes or HIV. The findings are specific to that facility and period.','https://files.gao.gov/reports/GAO-26-108886/index.html'),
 'medicare':('Medicare Advantage prior-authorization denials','A 2022 HHS OIG report found that 13 percent of sampled June 2019 Medicare Advantage prior-authorization denials met Medicare coverage rules. It is not a current rate for all claims; the report includes subsequent recommendation follow-up.','https://www.oig.hhs.gov/reports/all/2022/some-medicare-advantage-organization-denials-of-prior-authorization-requests-raise-concerns-about-beneficiary-access-to-medically-necessary-care/')
}

def stamp(u,topic,topics,scene):
 subject,fact,url=SOURCES[topic]
 u['political_basis']={'subject':subject,'real_hook':fact,'source_url':url,'fictionalization':'Original fictional checkpoint, hearing, dialogue and people. It is not a report of these events, a description of actual checkpoint powers or advice for obtaining care.'}
 u['sources']=[{'title':subject,'url':url,'qualification':fact}]
 u['source_hook']=subject
 u['evidence_status']='Sourced political basis; invented scene and dialogue'
 u['status']='Deep End rights and health revision for review'
 u['content_gate']='deep_only'
 u['content_topics']=topics
 u['scene']['description']=scene
 u['scene']['overlays']=[]
 u['illustration']=scene

CROSS=[
('A','The Privacy Camera','flock',['stalking','police_surveillance'],
 'The checkpoint’s Flock camera is advertised as protection. A woman ahead asks who protects her when an officer uses it to stalk her. The guard points to a second camera.',
 ['You pass. The second camera is apparently supervising the first camera’s personal growth.',
  'You take the diversion. The privacy notice promises to record your concerns, along with your number plate.',
  'Your permit is accepted. The guard briefly experiences identifying someone without stalking them.',
  'The payment clears the barrier. Privacy remains unavailable, but passage has a price.',
  'The payment is taken and passage refused. The camera has preserved an excellent record of the customer experience.',
  'The crossing ends your expedition. The protection camera continues protecting the guard from having to remember anything.'],
 'Waiting van before a checkpoint camera. A separate woman questions a guard who points toward another camera instead of answering. Blank plates and screens; no private location data.'),
('B','Two Documents, One Person','passport',['trans_discrimination','identity_documents'],
 'At the checkpoint, a trans traveler’s renewed federal passport conflicts with her state ID. The guard says the computer cannot reconcile them. She suggests looking up from the computer.',
 ['You clear the crossing. The woman ahead remains one person, despite the office’s impressive ability to manufacture two problems.',
  'You divert. The guard says the alternate road is less direct, a subject on which he has considerable experience.',
  'Your permit clears you. The guard accepts that some documents are allowed to agree with their owner.',
  'The payment opens the barrier. The computer recognizes money with touching flexibility.',
  'The payment is taken; passage is refused. Money has been recognized. People remain under review.',
  'The crossing ends your expedition. The computer displays no conflict with that outcome.'],
 'A separate trans traveler holds two blank identity cards with matching portraits while a guard stares at a blank monitor. Current crew and van wait behind; no body inspection or readable personal data.'),
('C','Screened for Needing Treatment','detention',['detention','denied_healthcare'],
 'The checkpoint advertises ICE-style medical screening. A diabetic man asks about treatment. The guard points to his intake form. “We’ve already established that you need some.”',
 ['You pass. The clinic’s treatment plan remains a very thorough description of why it needs a treatment plan.',
  'You divert. The intake desk describes your departure as reduced demand for services.',
  'Your permit is accepted. The crossing can process paper considerably faster than it processes people.',
  'The payment clears the barrier. The cash receives immediate attention from a specialist.',
  'The payment is taken without passage. The office has completed its financial intake.',
  'The crossing ends your expedition. The intake form is complete; the service it describes remains missing.'],
 'A separate adult at a checkpoint clinic gestures toward an unused treatment chair while a guard displays a blank completed intake form. No medical procedure, distress spectacle or new crew passenger.')
]
for variant,title,source,topics,setup,outs,scene in CROSS:
 u=by['CROSS-02D-'+variant];u.update(title=title,setup=setup)
 passed,diverted,permit,ok,no,failed=outs
 u['outcomes'].update(passage=passed,diversion=diverted,permit_receipt=permit+' One Receipt is used.',permit_tag=permit+' Your permit tag remains available.',bribe_success=ok,bribe_failure=no,terminal_failure=failed)
 stamp(u,source,topics,scene)

HEARINGS=[
('A','The Appendix Exception','emergency',['abortion_restrictions','emergency_care'],
 'At the abortion-ban hearing, a nurse explains why hospital lawyers keep getting called during emergencies. The chair announces a study. She asks whether his appendix also waits for peer review.',
 ['The first round drains stamina. The chair asks the nurse to define urgent. She points to the clock. He requests a second opinion.',
  'The second round establishes that time is critical. The committee takes a recess to decide which time.',
  'The last procedural round. The study receives an extension. Nobody has worked out how to give one to a patient.'],
 'You finish the procedure. One vote decides your case. The study will get considerably longer to make up its mind.',
 'You cannot finish the rounds, so there is no vote. The chair schedules a study of why people keep running out of time.',
 'A nurse points toward an ordinary wall clock while a hearing chair gestures to a thick blank study proposal. Only the actual arriving travelers stand at the public microphone. No patient or medical procedure is depicted.'),
('B','Nothing to Hide','flock',['stalking','police_surveillance'],
 'At the hearing on Flock misuse, the chair says innocent people have nothing to hide. A clerk starts describing his own camera records. The chair discovers executive privacy before the first sandwich shop.',
 ['The first procedural round drains stamina. The chair distinguishes public safety from information about his lunch.',
  'The next round discusses oversight. He demands oversight of whoever overlooked his right to have no oversight.',
  'The last procedural round. His sandwich has been sealed from the public record.'],
 'You endure the procedure. One vote will decide the case. The chair’s lunch has already received full protection.',
 'You cannot finish the rounds. The hearing ends before a vote; the chair calls this a useful reduction in surveillance.',
 'Hearing chair hurriedly closes a folder as a clerk points to a harmless sandwich-shop icon on a blank map. Actual arriving travelers remain at the microphone; no real addresses or private routes.'),
('C','Prior Authorization to Be Heard','medicare',['denied_healthcare','disability'],
 'The hearing concerns Medicare Advantage denials of qualifying care. The chair asks whether the patients appealed. A nurse holds up the appeal form. “Yes. It made a full recovery.”',
 ['The opening round drains stamina. The chair asks whether the appeal form needs another appeal form.',
  'The second round reviews the review. The nurse adds a chair for the paperwork; it has been here longest.',
  'The final procedural round. The paperwork is declared fit to return to work.'],
 'You finish the rounds. One vote now decides the case. The form is excused from further examination.',
 'You cannot finish the rounds, so no vote follows. The chair asks the paperwork whether it can continue without you.',
 'A nurse witnesses the hearing with a thick stack of blank appeal forms occupying a chair. Actual arriving travelers face the committee; no patient is depicted receiving or losing treatment.')
]
for variant,title,source,topics,setup,transitions,vote,exhausted,scene in HEARINGS:
 u=by['HEARING-D-'+variant];u.update(title=title,setup=setup,transitions=transitions,before_vote=vote,exhausted=exhausted)
 stamp(u,source,topics,scene)

by['HEARING-D-A']['sources'].append({'title':'Texas SB31 medical-exception clarification','url':'https://capitol.texas.gov/tlodocs/89R/billtext/html/SB00031F.HTM','qualification':'Enacted June 2025 text states that qualifying risk need not be imminent. The satire does not represent death or imminent danger as a statutory prerequisite.'})

(ROOT/'support.json').write_text(json.dumps(units,ensure_ascii=False,indent=2)+'\n')
print('Revised three Deep End crossings and three hearing packages')
