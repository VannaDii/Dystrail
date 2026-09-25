import {test,expect,Page} from '@playwright/test';
import {baseline,importState,snap,waitForLaunch,fastMode,openMenu} from './helpers';
import {readFileSync,writeFileSync} from 'node:fs';
test.setTimeout(90_000);
const checkpoint=(page:Page)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!));
const reportOf=(page:Page)=>checkpoint(page).then(s=>s.state.boss.hearing);
function arrival(state:any,sanity=7){
 state.boss={ready:true,reached:true,attempted:false,victory:false,presentation:'Arrival',hearing:null};
 state.current_encounter=null;state.breakdown=null;state.ending=null;state.ally_notice=null;
 state.crew_care.pending=null;state.route_services.stop=null;state.stats.sanity=sanity;
 state.policy='balanced';state.rng_bundle=null;state.seed=4242;return state;
}
function committed(state:any,rounds:number[],initial=7,base=.55,draw:number|null=42,guarantee=false){
 arrival(state,initial);let sanity=initial;
 const records=rounds.map((influence,i)=>{const before=sanity;sanity=Math.max(0,sanity-2);return {influence,sanity_before:before,sanity_after:sanity,continuation_roll:!sanity||i===2?null:i<rounds.length-1?10:80};});
 const adjusted=sanity?guarantee?1:base*rounds.reduce((a,b)=>a+b,0)/rounds.length/100:null;
 const outcome=!sanity?'Exhausted':adjusted!>=1?'Secured':draw!<adjusted!*100?'Passed':'Failed';
 state.boss.attempted=true;state.boss.victory=['Passed','Secured'].includes(outcome);
 state.boss.hearing={rules_version:1,starting_stats:{...state.stats},day:state.day,minute:state.clock_minutes,base_chance:base,policy_guarantee:guarantee,rounds:records,adjusted_chance:adjusted,vote_roll:['Secured','Exhausted'].includes(outcome)?null:draw,outcome};
 state.boss.presentation={RoundRolling:0};state.stats.sanity=sanity;return state;
}
async function next(page:Page){await page.getByRole('button',{name:'Continue',exact:true}).click();}
test('arrival, once-only resolution and skipped verdict recover unchanged',async({page})=>{
 const state=arrival(await baseline(page));state.clock_minutes=1110;await importState(page,state);
 await expect(page.locator('.hearing')).toHaveAttribute('data-hearing-phase','Arrival');
 await expect(page.locator('.journey-scene')).toHaveAttribute('data-time','dusk');
 for(const reduced of ['no-preference','reduce'] as const){
  await page.emulateMedia({reducedMotion:reduced});
  for(const fast of [false,true]){await fastMode(page,fast);await page.waitForTimeout(2700);await expect(page.locator('.hearing')).toHaveAttribute('data-hearing-phase','Arrival');expect((await checkpoint(page)).state.boss.attempted).toBe(false);}
 }
 await page.emulateMedia({reducedMotion:'no-preference'});await fastMode(page,false);
 await snap(page,'hearing-arrival');
 await page.getByRole('button',{name:'Enter the hearing room',exact:true}).click();
 await expect(page.locator('.hearing')).toHaveAttribute('data-hearing-phase','Preparation');
 await expect(page.getByRole('button',{name:'Rest before the hearing',exact:true})).toBeEnabled();
 await expect(page.locator('.hearing-microphone')).toHaveCount(0);
 await expect(page.locator('.hearing-brief')).not.toContainText(/\d|%|multiplier|probability/);
 for(const member of state.party.members.filter((m:any)=>m.status==='Active')){const portrait=page.locator(`.character-portrait[data-member="${member.persona}"]`);await expect(portrait).toHaveCount(1);await expect(portrait.locator('figcaption')).toHaveText(member.name);}
 await snap(page,'hearing-preparation');
 writeFileSync(test.info().outputPath('hearing-preparation-checkpoint.json'),JSON.stringify(await checkpoint(page),null,2));
 await page.getByRole('button',{name:'Begin hearing',exact:true}).dblclick();
 const resolved=await checkpoint(page);const report=resolved.state.boss.hearing;
 expect(report.rounds.length).toBeGreaterThanOrEqual(1);expect(report.rounds.length).toBeLessThanOrEqual(3);
 expect(resolved.state.stats.sanity).toBe(7-report.rounds.length*2);
 const visible=await page.locator('.hearing').evaluate(el=>({count:Number(el.getAttribute('data-revealed-rounds')),sanity:Number(el.querySelector('[data-hearing-sanity]')?.textContent),phase:el.getAttribute('data-hearing-phase')}));
 expect(visible.phase).not.toBe('Verdict');expect(visible.sanity).toBe(report.rounds[visible.count-1]?.sanity_after??7);const journal=resolved.state.journal.length;
 await page.reload();await waitForLaunch(page);expect(await reportOf(page)).toEqual(report);
 await page.getByRole('button',{name:'Skip to verdict',exact:true}).click();
 await expect(page.locator('.hearing')).toHaveAttribute('data-hearing-phase','Verdict');
 await expect(page.locator('.hearing-rounds > li[data-round]')).toHaveCount(report.rounds.length);await snap(page,'hearing-skipped-summary');
 await page.getByRole('button',{name:'View scorecard',exact:true}).click();
 await expect(page.locator('.hearing-scorecard')).toBeVisible();await page.reload();await waitForLaunch(page);
 const after=await checkpoint(page);expect(after.state.boss.hearing).toEqual(report);expect(after.state.journal.length).toBe(journal);expect(after.state.stats).toEqual(resolved.state.stats);
 await expect(page.locator('#main')).toHaveAttribute('data-screen','result');
});
test('three staged rounds reveal past costs and recover at every hold',async({page})=>{
 const state=committed(await baseline(page),[120,90,150]);
 state.party.members.find((m:any)=>m.persona==='satirist').status='Departed';
 await importState(page,state);await fastMode(page,true);expect((await checkpoint(page)).travel_speed).toBe('Fast');
 for(let i=0;i<3;i++){
  await expect(page.locator('.hearing')).toHaveAttribute('data-hearing-phase',`RoundResult(${i})`);
  await expect(page.locator('[data-hearing-sanity]')).toHaveText(String(5-i*2));await expect(page.locator('.hearing-rounds > li[data-round]')).toHaveCount(i+1);
  const row=page.locator(`.hearing-rounds > li[data-round="${i+1}"]`);
  await expect(row.locator('[data-stat="hearing.influence"] strong')).toHaveText(`${[120,90,150][i]}%`);
  await expect(row.locator('[data-stat="ux.sanity"] strong')).toHaveText('-2');
  await expect(row.locator('[data-stat="ux.sanity"] small')).toHaveText(`${7-i*2} → ${5-i*2}`);
  await expect(row.locator('[data-stat="hearing.odds"] small')).toContainText('→');
  await expect(page.locator('.hearing-questioning-closed')).toHaveCount(0);
  await expect(page.locator('.hearing-stage [data-member="satirist"]')).toHaveCount(0);await snap(page,`hearing-round-${i+1}`);
  const before=await reportOf(page);await page.reload();await waitForLaunch(page);expect(await reportOf(page)).toEqual(before);expect((await checkpoint(page)).travel_speed).toBe('Fast');
  await expect(page.locator('.hearing')).toHaveAttribute('data-hearing-phase',`RoundResult(${i})`);await next(page);
 }
 await expect(page.locator('.hearing')).toHaveAttribute('data-hearing-phase','Closed');
 await expect(page.locator('.hearing-controls button')).toHaveCount(1);
 await expect(page.locator('.hearing-questioning-closed')).toContainText('The committee is satisfied.');
 expect(await page.locator('.hearing-operator').evaluateAll(nodes=>nodes.every(n=>parseFloat(getComputedStyle(n).fontSize)>=36))).toBe(true);
 await expect(page.locator('.hearing-equation')).toContainText('66%');await expect(page.locator('.hearing-running-odds')).toHaveCount(0);await snap(page,'hearing-closed');
 await page.getByRole('button',{name:'Call the vote',exact:true}).click();
 await expect(page.getByRole('heading',{name:'Motion passes',exact:true})).toBeVisible();await expect(page.locator('.hearing-draw')).toHaveCount(0);await snap(page,'hearing-passed');
});
test('short hearings, failed vote, secured victory and exhaustion have distinct endings',async({page})=>{
 const base=await baseline(page);
 for(const sample of [
  {name:'short-one',rounds:[120],sanity:7,base:.55,draw:42,title:'Motion passes'},
  {name:'short-two',rounds:[120,90],sanity:7,base:.55,draw:78,title:'Motion fails'},
  {name:'secured',rounds:[140,130,120],sanity:7,base:.8,draw:null,title:'Victory secured'},
  {name:'exhausted',rounds:[120,90,150],sanity:5,base:.55,draw:null,title:'The crew cannot continue'},
 ]){
  const state=committed(structuredClone(base),sample.rounds,sample.sanity,sample.base,sample.draw);state.boss.presentation={RoundRolling:sample.rounds.length-1};
  await importState(page,state);await fastMode(page,true);
  if(sample.name==='exhausted'){await expect(page.locator('.hearing')).toHaveAttribute('data-hearing-phase','Verdict');}
  else{await expect(page.locator('.hearing')).toHaveAttribute('data-hearing-phase',`RoundResult(${sample.rounds.length-1})`);await next(page);
   if(sample.draw!==null){await expect(page.locator('.hearing')).toHaveAttribute('data-hearing-phase','Closed');await page.getByRole('button',{name:'Call the vote',exact:true}).click();}}
  await expect(page.getByRole('heading',{name:sample.title,exact:true})).toBeVisible();await expect(page.locator('.hearing-rounds > li[data-round]')).toHaveCount(sample.rounds.length);
  await expect(page.locator('.hearing-draw')).toHaveCount(0);
  const mood=['short-one','secured'].includes(sample.name)?'happy':'defeated';
  await expect(page.locator(`.hearing-stage .character-art[data-expression="${mood}"]`)).toHaveCount(6);await snap(page,`hearing-ending-${sample.name}`);
  await page.getByRole('button',{name:'View scorecard',exact:true}).click();await expect(page.locator('.result-headline')).toHaveText(sample.title);
 }
});
test('reduced motion, guaranteed approval and legacy results preserve outcomes',async({page})=>{
 const base=await baseline(page);const state=committed(structuredClone(base),[50,50,50],7,1,null,true);
 await page.emulateMedia({reducedMotion:'reduce'});await importState(page,state);
 await expect(page.locator('.hearing')).toHaveAttribute('data-hearing-phase','RoundResult(0)');
 expect(await page.locator('.hearing-rounds > li[data-round]').first().evaluate(el=>getComputedStyle(el).animationName)).toBe('none');
 await page.getByRole('button',{name:'Skip to verdict',exact:true}).click();await expect(page.getByRole('heading',{name:'Victory secured',exact:true})).toBeVisible();
 await expect(page.locator('.hearing-equation')).not.toContainText('×');await expect(page.locator('.hearing-resolution')).toContainText('votes are promised');
 const report=await reportOf(page);await page.reload();await waitForLaunch(page);expect(await reportOf(page)).toEqual(report);
 const legacy=structuredClone(base);legacy.boss={ready:true,reached:true,attempted:true,victory:true};await importState(page,legacy);
 await expect(page.locator('#main')).toHaveAttribute('data-screen','result');expect((await checkpoint(page)).state.boss.hearing).toBeNull();
});

test('rest returns to preparation; manual saves and modal pauses preserve committed questioning',async({page})=>{
 const errors:string[]=[];page.on('pageerror',error=>errors.push(error.message));
 const state=arrival(await baseline(page),5);state.boss.presentation='Preparation';state.camp.rest_cooldown=0;
 await importState(page,state);await page.getByRole('button',{name:'Rest before the hearing',exact:true}).click();
 await expect(page.locator('#main')).toHaveAttribute('data-screen','camp');
 await page.getByRole('button',{name:/^(Take a day to rest|Rest for the day|Rest and mute the phones)$/}).click();await expect(page.locator('.aftermath-panel')).toBeVisible();
 const rested=(await checkpoint(page)).state;expect(rested.stats.sanity).toBeGreaterThan(5);expect(rested.boss.attempted).toBe(false);
 await next(page);await expect(page.locator('.hearing')).toHaveAttribute('data-hearing-phase','Preparation');
 const staged=committed(rested,[120,90,150]);staged.boss.presentation={RoundRolling:1};
 await importState(page,staged);await fastMode(page,false);
 await openMenu(page);await page.locator('#save-open-btn').click();
 await expect(page.locator('.drawer')).toBeVisible();const cursor=(await checkpoint(page)).state.boss.presentation;
 await page.locator('.drawer').getByRole('button',{name:'Save Now',exact:true}).click();
 await page.waitForTimeout(3200);expect((await checkpoint(page)).state.boss.presentation).toEqual(cursor);
 await page.keyboard.press('Escape');await expect(page.locator('.drawer')).toHaveCount(0);
 await expect(page.locator('.hearing')).toHaveAttribute('data-hearing-phase','RoundResult(1)');
 await expect(page.locator('#hearing-next')).toBeFocused();expect(await reportOf(page)).toEqual(staged.boss.hearing);
 await openMenu(page);await page.locator('#save-open-btn').click();await page.locator('.drawer').getByRole('button',{name:'Load',exact:true}).click();
 await expect(page.locator('.hearing')).toHaveAttribute('data-hearing-phase','RoundRolling(1)');
 expect(await reportOf(page)).toEqual(staged.boss.hearing);await page.getByRole('button',{name:'Skip to verdict',exact:true}).click();
 await expect(page.locator('.hearing')).toHaveAttribute('data-hearing-phase','Verdict');
 await page.waitForTimeout(3200);await expect(page.locator('.hearing')).toHaveAttribute('data-hearing-phase','Verdict');
 expect((await checkpoint(page)).state.stats).toEqual(staged.stats);expect(errors).toEqual([]);
});
for(const [lang,label] of [['it','Italiano'],['es','Español'],['ar','العربية']]){
 test(`hearing ${lang} remains legible and resumes offline`,async({page,context})=>{
  const state=committed(await baseline(page),[120,90,150]);state.boss.presentation={RoundResult:1};await importState(page,state);
  const copy=JSON.parse(readFileSync(`i18n/${lang}.json`,'utf8')).hearing;
  await openMenu(page);await page.getByRole('button',{name:'Language',exact:true}).click();await page.getByRole('option',{name:label,exact:true}).click();await page.locator('#game-menu-button').click();
  await expect(page.locator('html')).toHaveAttribute('dir',lang==='ar'?'rtl':'ltr');
  await expect(page.locator('#hearing-title')).toHaveText(copy.followup);await expect(page.locator('.hearing-dialogue')).toHaveText(copy.followup_negative);
  await expect(page.locator('.hearing')).not.toContainText(/hearing\.|\{\w+\}/);
  await snap(page,`hearing-${lang}`);await context.setOffline(true);await page.reload();await waitForLaunch(page);
  expect(await reportOf(page)).toEqual(state.boss.hearing);await expect(page.locator('#hearing-title')).toHaveText(copy.followup);
  expect(await page.locator('.hearing-chair image').evaluate(async el=>{const response=await fetch(el.getAttribute('href')!);return response.ok;})).toBe(true);
  await context.setOffline(false);
 });
}

test('preparation speaks in narrative while preserving costs and the vote cutoff',async({page})=>{
 const base=await baseline(page,true);
 for(const sample of [{supplies:4,cash:2000,cost:'supplies'}, {supplies:3,cash:2000,cost:'buy a bite'}]){
  const state=arrival(structuredClone(base),2);state.policy='aggressive';state.boss.presentation='Preparation';state.stats.supplies=sample.supplies;state.budget_cents=sample.cash;
  await importState(page,state);await expect(page.locator('.hearing-preparation-cost')).toContainText(sample.cost);await expect(page.locator('.hearing-preparation-cost')).not.toContainText(/\d|%|→/);
  await expect(page.locator('.hearing-brief')).not.toContainText(/\d|%|→/);
  if(sample.supplies===4)writeFileSync(test.info().outputPath('policy-preparation-checkpoint.json'),JSON.stringify(await checkpoint(page),null,2));
  expect((await checkpoint(page)).state.stats.sanity).toBe(2);expect((await checkpoint(page)).state.budget_cents).toBe(sample.cash);
  await page.getByRole('button',{name:'Begin hearing',exact:true}).click();
  const resolved=(await checkpoint(page)).state;expect(resolved.boss.hearing.starting_stats.sanity).toBe(3);
  const report=resolved.boss.hearing;await page.reload();await waitForLaunch(page);expect(await reportOf(page)).toEqual(report);expect((await checkpoint(page)).state.stats).toEqual(resolved.stats);expect((await checkpoint(page)).state.budget_cents).toBe(resolved.budget_cents);
 }
 const rounded=committed(structuredClone(base),[100],7,.550000011920929,55);rounded.boss.presentation='Verdict';await importState(page,rounded);
 await expect(page.locator('.hearing-equation')).toContainText('55%');await expect(page.locator('.hearing')).not.toContainText(/Final draw:|Passing draws:|Only completed rounds/);
 await expect(page.getByRole('heading',{name:'Motion passes',exact:true})).toBeVisible();await snap(page,'hearing-draw-boundary');
});


test('all six characters carry their ending expression into the downloaded PNG offline',async({page})=>{
 test.setTimeout(150_000);
 const base=await baseline(page);await page.context().setOffline(true);
 for(const persona of ['journalist','organizer','whistleblower','lobbyist','staffer','satirist']){
  for(const [mood,draw,cell] of [['happy',48,0],['defeated',99,1]] as const){
   const state=committed(structuredClone(base),[117],10,.55,draw);state.persona_id=persona;state.boss.presentation='Complete';
   state.party.members.find((m:any)=>m.persona!==persona).status='Departed';
   await importState(page,state);
   await expect(page.locator('.result-profile .character-art')).toHaveAttribute('data-expression',mood);
   await expect(page.locator('.ending-crew [data-fate="crew.departed"] .character-art')).toHaveAttribute('data-expression','standard');
   await expect(page.locator(`.ending-crew [data-expression="${mood}"]`)).toHaveCount(5);
   await page.locator('#result-share-open').click();
   const preview=page.locator('.share-preview img');await expect(preview).toBeVisible();
   const matching=await preview.evaluate(async (node:HTMLImageElement,{persona,cell})=>{
    await node.decode();
    const source=new Image();source.src=(window as any).dystrailAssetUrls[`static/img/journey/occupant-${persona}-expressions-v1.png`];await source.decode();
    const actual=document.createElement('canvas');actual.width=actual.height=1200;const actualCtx=actual.getContext('2d')!;actualCtx.drawImage(node,0,0);
    const expected=document.createElement('canvas');expected.width=expected.height=284;const expectedCtx=expected.getContext('2d')!;expectedCtx.imageSmoothingEnabled=false;
    expectedCtx.drawImage(source,cell*source.naturalWidth/2,0,source.naturalWidth/2,source.naturalHeight,0,0,284,284);
    const a=actualCtx.getImageData(66,166,284,284).data,b=expectedCtx.getImageData(0,0,284,284).data;
    return a.every((value,index)=>value===b[index]);
   },{persona,cell});
   expect(matching).toBe(true);
   if(persona==='organizer'){
    const [download]=await Promise.all([page.waitForEvent('download'),page.getByRole('link',{name:'Save image',exact:true}).click()]);
    await download.saveAs(test.info().outputPath(`share-${mood}.png`));await snap(page,`hearing-share-${mood}`);
   }
   await page.keyboard.press('Escape');
   expect((await checkpoint(page)).state.boss.hearing).toEqual(state.boss.hearing);
  }
 }
});


test('narrow hearing portraits and result cards stay readable before the next action',async({page})=>{
 await page.setViewportSize({width:320,height:860});
 const state=committed(await baseline(page),[120,90]);state.boss.presentation={RoundResult:1};
 state.party.members.forEach((m:any,i:number)=>m.name=['Alexandria Montgomery','Emerson Fitzgerald','Skyler','Taylor','Morgan','Finley'][i]);
 await importState(page,state);await fastMode(page,true);
 const frames=page.locator('.hearing-stage .character-portrait[data-member]');await expect(frames).toHaveCount(6);
 expect(await frames.evaluateAll(nodes=>nodes.every(n=>{const name=n.querySelector('figcaption')!;return name.scrollWidth<=name.clientWidth&&parseFloat(getComputedStyle(name).fontSize)>=12;}))).toBe(true);
 expect(await page.locator('.hearing-round-stats .stat-card').evaluateAll(nodes=>nodes.every(n=>n.querySelector('strong')!.getBoundingClientRect().width<=n.clientWidth))).toBe(true);
 await next(page);await expect(page.locator('.hearing')).toHaveAttribute('data-hearing-phase','Closed');
 const closing=page.locator('.hearing-questioning-closed');await expect(closing).toContainText('The committee is satisfied.');
 const announcement=await closing.boundingBox(),button=await page.locator('#hearing-next').boundingBox();expect(button!.y).toBeGreaterThanOrEqual(announcement!.y+announcement!.height);
 await page.reload();await waitForLaunch(page);await expect(closing).toContainText('No further questions.');
 await page.evaluate(()=>window.scrollTo(0,0));await snap(page,'hearing-narrow');
});
