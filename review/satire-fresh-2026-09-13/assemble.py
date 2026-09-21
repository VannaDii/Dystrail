import json,re,collections
from pathlib import Path
ROOT=Path(__file__).resolve().parent
parts={n:json.loads((ROOT/(n+'.json')).read_text()) for n in ['roads-a','roads-b','towns','support','functional']}
narrative=sum([parts[n] for n in ['roads-a','roads-b','towns','support']],[])
all_units=narrative+parts['functional']
by={u['id']:u for u in all_units}
sections=[
 ('start','Start here',[],1),
 ('classic','Road — Classic',[u for u in narrative if u['category']=='Road encounters' and u['mode']=='Classic'],72),
 ('deep','Road — Deep End',[u for u in narrative if u['category']=='Road encounters' and u['mode']=='Deep End'],57),
 ('shared','Road — Shared',[u for u in narrative if u['category']=='Road encounters' and u['mode']=='Shared'],102),
 ('towns','Town conversations',parts['towns'],132),
 ('care','Crew care',[u for u in narrative if u['category']=='Crew-care scenarios'],24),
 ('allies','Outside allies',[u for u in narrative if u['category']=='Ally-departure vignettes'],18),
 ('crossings','Crossings',[u for u in narrative if u['category']=='Crossing packages'],12),
 ('orders','Executive orders',[u for u in narrative if u['category']=='Executive-order bulletins'],18),
 ('repairs','Breakdowns & repairs',[u for u in narrative if u['category']=='Breakdown and repair packages'],12),
 ('routines','Gather, work, barter, rest',[u for u in narrative if u['category']=='Gathering, work, barter, and rest'],24),
 ('conditions','Weather & health',[u for u in narrative if u['category']=='Weather, illness, hunger, and exposure'],27),
 ('departures','Departure introductions',[u for u in narrative if u['category']=='Character departure introductions'],18),
 ('hearing','The hearing',[u for u in narrative if u['category']=='Hearing introductions and transitions'],6),
 ('endings','Endings',[u for u in narrative if u['category']=='Ending packages'],45),
 ('reference','Facts & functional copy',parts['functional'],77)
]

def line(text,role='body',link=None):
    text=str(text).replace('\n',' ').strip()
    return {'text':text,'role':role,**({'link':link} if link else {})}

def render(u):
    rows=[line(u['id']+' · '+u['title'],'heading')]
    if u.get('content_gate')=='deep_only' and u['category']!='Road encounters':rows.append(line('Deep End only','meta'))
    if u.get('town'):rows.append(line(u['town'],'meta'))
    if u.get('setup'):rows.append(line(u['setup']))
    if u.get('fact'):rows.append(line('Fact: '+u['fact']))
    if u.get('commentary'):rows.append(line(u.get('speaker','Fictional resident')+' — “'+u['commentary'].strip('“”')+'”'))
    if u.get('continuing'):rows.append(line('Continuing: '+u['continuing']))
    if u.get('critical'):rows.append(line('Critical warning: '+u['critical']))
    if u.get('onset'):rows.append(line(u['onset']))
    if u.get('activation'):rows.append(line(u['activation']))
    if u.get('effect'):rows.append(line('Game effect: '+u['effect']))
    for c in u.get('choices',[]):
        rows.append(line(c['label']+' → '+c['outcome'],'choice'))
        if c.get('roadside_outcome'):rows.append(line('Roadside purchase → '+c['roadside_outcome'],'choice'))
        if c.get('availability') and u['category']=='Breakdown and repair packages':rows.append(line(c['availability'],'meta'))
    if u.get('action'):
        rows.append(line(u['action']+' → '+u['outcome'],'choice'))
    elif u.get('outcome'):rows.append(line(u['outcome']))
    for n,text in enumerate(u.get('transitions',[]),1):rows.append(line(f'Round {n}: '+text))
    if u.get('before_vote'):rows.append(line('Before the vote: '+u['before_vote']))
    if u.get('exhausted'):rows.append(line('If exhausted: '+u['exhausted']))
    outcomes=u.get('outcomes',{})
    if u['category']=='Crew-care scenarios':outcomes={k:v for k,v in outcomes.items() if k in ['companion_lost','player_lost']}
    for k,text in outcomes.items():rows.append(line(k.replace('_',' ').capitalize()+' → '+text,'choice'))
    if u.get('final_ally'):rows.append(line('If this was the last outside ally: '+u['final_ally']))
    if u.get('recovery'):rows.append(line('Recovery / relief: '+u['recovery']))
    if u.get('expiration'):rows.append(line('When it expires: '+u['expiration']))
    if u.get('epilogue'):rows.append(line(u['epilogue']))
    if u.get('text'):rows.append(line(u['text']))
    s=u.get('scene',{})
    if s:
        desc=s.get('description')
        if not desc:
            pieces=[str(s.get(k,'')).strip() for k in ['action','visual_gag'] if s.get(k)]
            desc=' '.join(dict.fromkeys(pieces)) or str(s.get('setting',''))
        rows.append(line('Scene: '+desc,'scene'))
        overlays=s.get('overlays',[])
        if overlays:
            vals=[]
            for o in overlays:
                if isinstance(o,str): vals.append(o)
                else: vals.append('“'+str(o.get('text',''))+'”'+(' — '+str(o['placement']) if o.get('placement') else ''))
            rows.append(line('Editable overlays: '+'; '.join(vals),'scene'))
        # Outcome-specific art guards remain in the structured pack; the mobile
        # review shows the scene itself without repeating production instructions.
    for n,s in enumerate(u.get('sources',[]),1):
        if isinstance(s,str):s={'url':s}
        url=s.get('url')
        if not url:continue
        label=s.get('title') or s.get('claim') or 'Source'
        if len(label)>130:label=label[:127]+'…'
        date=s.get('date')
        if date:label+=f' ({date})'
        rows.append(line('Source: '+label,'source',url))
        qualification=s.get('qualification')
        if qualification and u in parts['functional']:rows.append(line(qualification,'meta'))
    return rows

intro=[
 line('Dystrail · Fresh satire draft','title'),
 line('Revised political satire draft. This workshop contains 567 narrative packages and 77 factual or functional records. All narrative sections are ready for editorial review.'),
 line('Six ordinary people set out in one van to make their case in D.C. Road scenes must establish how they become involved: a stop, a work offer, a purchase, a request for help, or trouble along the route. They do not automatically have customers, employers, staff or authority.'),
 line('How to review','heading'),
 line('Choose a content-type tab. Each entry has its stable ID, playable copy, scene direction and any editable overlays. Choices precede the arrow; their results follow. A, B and C are alternative scenes, not a sequence.'),
 line('Classic keeps the political absurdity lighter. Deep End is an explicit opt-in to darker satire about attacks on trans rights, reproductive restrictions, surveillance abuse, detention, and denied care. Shared packages must fit Classic; Deep End material must never enter a shared pool or a Classic fallback.'),
 line('Every narrative package needs a recognizable government action, political promise, policy or public scandal, an ordinary consequence, and a punchline that depends on that connection. Generic jokes about a boss, an app or poor service do not qualify. Approved English then moves into game resources, scene production and Spanish, Italian and Arabic localization.'),
 line('The encounters, dialogue, bulletins and hearing are fictional. Linked sources document the real political hook. Town facts retain dates and distinguish proposals from enacted changes. Detailed source qualifications and outcome-specific art instructions remain with the structured content pack.'),
 line('Art direction','heading'),
 line('Every package suggests a display scene. Use distinct assets wherever the joke needs them; scene count is not constrained by the current art bank. Images contain no readable text, logos, numbers or lettering. Signs, book titles, messages, captions and interface words are separate localized overlays.'),
 line('Before a choice, show the offer or problem. Show completed work, payments, exchanges and repairs only after the matching outcome. Use only present travelers; external allies are contacts outside the van. Departures, deaths and the player’s actual appearance remain authoritative.'),
 line('Scope','heading'),
 line('Road: 72 Classic, 57 Deep End, 102 shared. Towns: 132. Care: 24. Outside allies: 18. Crossings: 12. Orders: 18. Repairs: 12. Routine activities: 24. Conditions: 27. Departure introductions: 18. Hearing: 6. Endings: 45.'),
 line('The 12 new western road families have proposed mechanical branches for design review. Existing families retain their ordered effects. Each town has three different sourced angles; fictional resident speech is separated from the facts.'),
 line('Current-game corrections','heading'),
 line('The 18 executive-order slots now cover the six orders the simulation actually runs: Shutdown, Travel Ban Lite, Book Panic, Tariff Tsunami, Education Department Eliminated, and War Department Reorganization. The draft ledger keeps the old editorial IDs as traceable slots.'),
 line('The legal-fund item currently grants credibility, while tariff protection checks a separate inventory tag. Its description does not promise protection unless that tag is actually present.'),
 line('Reference records retain their dated population/source basis. A proposal, warning, attempted cancellation or dated report is not rewritten as a confirmed current loss.')
]
out=[]
for key,title,group,target in sections:
    ready=(key=='start' or len(group)==target and all(u.get('scene') and (key=='reference' or u.get('political_basis')) for u in group))
    rows=intro if key=='start' else [line(title,'title'),line(f'{len(group)} draft packages. Images have no embedded text; wording listed as an overlay is rendered separately.','meta')]+[r for u in sorted(group,key=lambda x:x['id']) for r in render(u)]
    out.append({'key':key,'title':title,'target':target,'count':len(group),'ready':ready,'rows':rows,'text':'\n'.join(r['text'] for r in rows)+'\n'})
(ROOT/'doc-sections.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n')
(ROOT/'narrative-pack.json').write_text(json.dumps({'status':'English draft for user review; not integrated or translated','units':sorted(narrative,key=lambda x:x['id'])},ensure_ascii=False,indent=2)+'\n')
(ROOT/'complete-pack.json').write_text(json.dumps({'status':'English draft for user review; not integrated or translated','units':sorted(all_units,key=lambda x:x['id'])},ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'narrative':len(narrative),'functional':len(parts['functional']),'scenes':sum(bool(u.get('scene')) for u in all_units),'doc_chars':sum(len(x['text']) for x in out),'sections':[{'key':x['key'],'count':x['count'],'ready':x['ready']} for x in out]}))
