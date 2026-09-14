import {test,expect} from '@playwright/test';
import { baseline,importState,openMenu,snap, waitForLaunch } from './helpers';
import {atTown,routes} from './geography';
import fs from 'node:fs';
const checkpoint=(page:any)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!));
const events=JSON.parse(fs.readFileSync('static/assets/data/game.json','utf8'));

test('local conversations retain the actual reward at and below the credibility cap',async({page})=>{
 const original=await baseline(page);
 for(const credibility of [19,20]) {
  const gs=structuredClone(original);atTown(gs,routes.find((r:any)=>r.id===gs.persona_id).stops.find((s:any)=>Math.floor(s.mile/10)%3===0&&s.name!=='D.C.').name);gs.stats.credibility=credibility;
  await importState(page,gs);const talk=page.getByRole('button',{name:'Talk to locals',exact:true});
  await expect(talk.locator('s')).toHaveCount(0);
  await expect(talk.locator('small')).toHaveText(credibility===20?'Receipts +1':'Credibility +1');
  await talk.click();await expect(page.locator('.local-conversation')).toBeVisible();const first=(await checkpoint(page)).state;
  expect(first.stats.credibility).toBe(20);expect(first.route_services.talk_reward).toBe(credibility===20?'Receipt':'Credibility');expect(first.receipts.length-gs.receipts.length).toBe(credibility===20?1:0);
  await page.getByRole('button',{name:'Back to town',exact:true}).click();await expect(talk).toBeEnabled();
  await expect(talk.locator('s')).toHaveText(credibility===20?'Receipts +1':'Credibility +1');
  await page.reload();await waitForLaunch(page);await talk.click();const repeated=(await checkpoint(page)).state;expect(repeated.stats).toEqual(first.stats);expect(repeated.clock_minutes).toBe(first.clock_minutes);expect(repeated.journal).toEqual(first.journal);
 }
 await page.getByRole('button',{name:'Back to town',exact:true}).click();await snap(page,'town-capped-reward');
});

test('outcomes show one heading and consistent cash, condition, evidence and part cards',async({page})=>{
 const gs=await baseline(page);gs.current_encounter=events.find((e:any)=>e.id==='classic_mutual_aid');
 await importState(page,gs);await page.locator('.encounter-choice button').nth(1).click();
 await expect(page.locator('.aftermath-panel #aftermath-title')).toHaveText('Outcome');await expect(page.locator('.aftermath-panel h2')).toHaveCount(1);await expect(page.locator('.aftermath-panel')).not.toContainText('THE CONSEQUENCES');
 await expect(page.locator('.resource-changes .change').filter({hasText:'Cash'})).toContainText('Cash+$10');await expect(page.locator('.receipt-details')).not.toContainText('Cash remaining');await snap(page,'cash-outcome-cards');
 gs.current_encounter=null;gs.breakdown={part:'Battery',day_started:gs.day};gs.inventory.spares.battery=1;gs.vehicle.health=93.27;await importState(page,gs);
 await page.getByRole('button',{name:'Fit your spare Battery',exact:true}).click();
 await expect(page.locator('.resource-changes .change').filter({hasText:'Battery'})).toContainText('-1');await expect(page.locator('.resource-changes .change').filter({hasText:'Vehicle condition'})).toContainText('+6.73%');await expect(page.locator('.receipt-details')).not.toContainText('93.27');await snap(page,'repair-outcome-cards');
});

test('advertised receipts become visible awards and survive offline play and reload',async({page,context})=>{
 const gs=await baseline(page);gs.mods.receipt_find_pct=-100;gs.current_encounter=events.find((e:any)=>e.id==='west_grant_translation');await importState(page,gs);
 const count=page.locator('[data-stat="ux.receipt"] dd');await expect(count).toHaveText('0');
 await expect(page.locator('.encounter-choice').nth(1)).toContainText('Receipts +1');
 await page.getByRole('button',{name:'2) Publish both versions',exact:true}).click();
 await expect(page.locator('.resource-changes .change').filter({hasText:'Receipts'})).toContainText('Receipts+1');await expect(count).toHaveText('1');
 const awarded=(await checkpoint(page)).state;expect(awarded.receipts).toEqual(['west_grant_translation']);
 await page.reload();await waitForLaunch(page);await expect(count).toHaveText('1');expect((await checkpoint(page)).state.receipts).toEqual(awarded.receipts);
 await page.getByRole('button',{name:'Back to the road',exact:true}).click();await page.getByRole('tab',{name:'Journal',exact:true}).click();await expect(page.locator('.journal-day > .journal-story > .resource-changes').first()).toContainText('Receipts+1');
 await page.getByRole('button',{name:'Route',exact:true}).click();await expect(page.locator('.map-scene')).toBeVisible();await expect(count).toHaveText('1');await page.reload();await waitForLaunch(page);await expect(page.locator('.map-scene')).toBeVisible();await expect(count).toHaveText('1');await snap(page,'receipts-map');
 await importState(page,awarded);
 await expect.poll(()=>page.evaluate(()=>(window as any).dystrailOffline?.state)).toBe('ready');await context.setOffline(true);
 const offline=(await checkpoint(page)).state;offline.current_encounter=events.find((e:any)=>e.id==='west_laboratory_overhead');await importState(page,offline);
 await page.getByRole('button',{name:'1) Take a paid equipment-inventory shift',exact:true}).click();await expect(count).toHaveText('2');
 await expect(page.locator('.resource-changes .change').filter({hasText:'Receipts'})).toContainText('Receipts+1');await expect(page.locator('.resource-changes .change').filter({hasText:'Cash'})).toContainText('Cash+$14');
 const second=(await checkpoint(page)).state;expect(second.receipts).toEqual(['west_grant_translation','west_laboratory_overhead']);
 await page.reload();await waitForLaunch(page);await expect(count).toHaveText('2');expect((await checkpoint(page)).state.receipts).toEqual(second.receipts);expect((await checkpoint(page)).state.budget_cents).toBe(second.budget_cents);
 await page.getByRole('button',{name:'Back to the road',exact:true}).click();await page.getByRole('tab',{name:'Journal',exact:true}).click();
 const receiptDay=page.locator(`.journal-day[data-day="${second.journal.at(-1).day}"]`);
 await expect(receiptDay.locator(':scope > .journal-story > .resource-changes')).toContainText('Receipts+2');
 await receiptDay.locator(':scope > .journal-raw > summary').click();
 await expect(receiptDay.locator('.journal-raw-entry [data-stat="ux.receipt"] strong')).toHaveText(['+1','+1']);
 expect((await checkpoint(page)).state.journal).toEqual(second.journal);
 await receiptDay.locator(':scope > .journal-raw > summary').click();
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);await snap(page,'receipts-offline-journal');
 const townState=(await checkpoint(page)).state;atTown(townState,'Madison');await importState(page,townState);await expect(count).toHaveText('2');
 await page.getByRole('button',{name:'Trade with locals',exact:true}).click();await expect(page.locator('.town-trading')).toBeVisible();await expect(count).toHaveText('2');await page.getByRole('button',{name:'Back to town',exact:true}).click();
 await page.getByRole('button',{name:'Visit the Store',exact:true}).click();await expect(page.locator('.outfit-title-row')).toBeVisible();await expect(count).toHaveText('2');
});

test('ally losses have a persistent acknowledgment and remain in the journal',async({page})=>{
 const gs=await baseline(page);gs.stats.allies=3;
 gs.current_encounter={...events[0],id:'test_ally_departure',choices:[{label:'Acknowledge the farewell',effects:{allies:-1,morale:-1,log:'An ally has to return home and can no longer help with the hearing.'}}]};
 await importState(page,gs);await page.getByRole('button',{name:'1) Acknowledge the farewell',exact:true}).click();
 await expect(page.locator('#main')).toHaveAttribute('data-screen','ally-loss');await expect(page.locator('.ally-message')).toContainText('return home');await expect(page.locator('.ally-departure .change').filter({hasText:'Allies'})).toContainText('3 → 2');await expect(page.getByRole('button',{name:'Travel',exact:true})).toBeDisabled();
 const lost=(await checkpoint(page)).state;expect(lost.ally_notice).not.toBeNull();expect(lost.journal.at(-1).after.allies).toBe(2);await page.reload();await waitForLaunch(page);await expect(page.locator('.ally-departure')).toBeVisible();await snap(page,'ally-departure');
 // The same pending loss must wait behind an already opened automatic map.
 await page.evaluate(()=>{const c=JSON.parse(localStorage.getItem('dystrail.autosave.v1')!);c.phase='Map';c.map_automatic=true;localStorage.setItem('dystrail.autosave.v1',JSON.stringify(c));});await page.reload();await waitForLaunch(page);await expect(page.locator('.map-scene')).toBeVisible();await page.getByRole('button',{name:'Resume travel',exact:true}).click();await expect(page.locator('.ally-departure')).toBeVisible();
 await page.getByRole('button',{name:'Back to the road',exact:true}).click();await expect(page.locator('.ally-departure')).toHaveCount(0);const acknowledged=(await checkpoint(page)).state;expect(acknowledged.ally_notice).toBeNull();expect(acknowledged.stats.allies).toBe(2);expect(acknowledged.journal).toEqual(lost.journal);await page.getByRole('tab',{name:'Journal',exact:true}).click();await expect(page.locator('.journal-day > .journal-story .journal-event-heading').filter({hasText:'return home'})).toBeVisible();
});

test('long journals scroll by keyboard without overlapping the footer',async({page})=>{
 const gs=await baseline(page);expect(gs.journal.length).toBeGreaterThan(0);
 gs.journal=Array.from({length:12},(_,i)=>({...structuredClone(gs.journal[0]),title:`Journal check ${i+1}`,day:i+1}));await importState(page,gs);
 await page.getByRole('tab',{name:'Journal',exact:true}).click();const log=page.getByRole('log',{name:'Trail journal',exact:true});
 await expect(log).toHaveCSS('overflow-y','auto');await log.focus();await expect(log).toBeFocused();
 await page.keyboard.press('End');await expect.poll(()=>log.evaluate(el=>el.scrollTop>0 && el.scrollTop+el.clientHeight>=el.scrollHeight-2)).toBe(true);
 const bounds=await log.evaluate(el=>({bottom:el.getBoundingClientRect().bottom,lastBottom:el.lastElementChild!.getBoundingClientRect().bottom,footerTop:document.querySelector('#main>footer')!.getBoundingClientRect().top}));
 expect(bounds.lastBottom).toBeLessThanOrEqual(bounds.bottom+1);expect(bounds.footerTop).toBeGreaterThanOrEqual(bounds.bottom);
 await snap(page,'journal-contained');await page.keyboard.press('Shift+Tab');await expect(log).not.toBeFocused();
});

test('town information, action spacing, journal cards and map credits share the established alignment',async({page},info)=>{
 const gs=await baseline(page);atTown(gs,'Madison');gs.stats.supplies=8;await importState(page,gs);
 await expect(page.locator('.town-vitals')).toContainText('Wisconsin');await expect(page.locator('.town-vitals')).toContainText('286,233');await expect(page.locator('.town-vitals')).toContainText('2025');await expect(page.locator('.town-attraction')).toBeVisible();
 const actionTitles=await page.locator('.route-stop .action-title').allTextContents();expect(actionTitles).toEqual(['Talk to locals']);
 const buttons=page.locator('.route-stop .controls>button');expect(await buttons.allTextContents()).toEqual([expect.stringContaining('Talk to locals'),'Trade with locals','Visit the Store']);
 const rects=await buttons.evaluateAll(es=>es.map(e=>({x:e.getBoundingClientRect().x,w:e.getBoundingClientRect().width,align:getComputedStyle(e).textAlign})));for(const r of rects){expect(r.x).toBeCloseTo(rects[0].x,0);expect(r.w).toBeCloseTo(rects[0].w,0);expect(r.align).toBe('start');}await snap(page,'town-profile');
 await page.getByRole('button',{name:'Visit the Store',exact:true}).click();const heading=await page.locator('.outfit-title-row h1').boundingBox(),leave=await page.getByRole('button',{name:'Leave store',exact:true}).boundingBox();if(info.project.name==='chromium'){expect(heading&&leave).toBeTruthy();expect(Math.abs(heading!.y+heading!.height/2-leave!.y-leave!.height/2)).toBeLessThan(1);}await snap(page,'town-store-alignment');await page.getByRole('button',{name:'Leave store',exact:true}).click();
 await page.getByRole('button',{name:'Take a paid unloading shift',exact:true}).click();await page.getByRole('button',{name:'Back to town',exact:true}).click();await expect(page.locator('.action-availability')).toHaveCSS('text-align','center');await page.getByRole('tab',{name:'Journal',exact:true}).click();
 const worked=(await checkpoint(page)).state,dayEntries=worked.journal.filter((entry:any)=>entry.day===worked.journal.at(-1).day);
 const cash=dayEntries.flatMap((entry:any)=>entry.resources.filter((change:any)=>change.key==='play.cash'));
 expect(cash.length).toBeGreaterThanOrEqual(2);expect(cash[0].after).toBeLessThan(cash[0].before);
 expect(cash.at(-1).after-cash.at(-1).before).toBe(1800);expect(cash.at(-1).after).toBe(worked.budget_cents);
 const dailyCash=cash.at(-1).after-cash[0].before,day=page.locator(`.journal-day[data-day="${worked.journal.at(-1).day}"]`);
 const total=day.locator(':scope > .journal-story > .resource-changes [data-stat="play.cash"] strong');
 if(dailyCash===0)await expect(total).toHaveCount(0);else await expect(total).toHaveText(`${dailyCash>0?'+':'−'}${new Intl.NumberFormat('en',{style:'currency',currency:'USD',maximumFractionDigits:0}).format(Math.abs(dailyCash)/100)}`);
 await day.locator(':scope > .journal-raw > summary').click();await expect(day.locator('.journal-raw-entry')).toHaveCount(dayEntries.length);
 await expect(day.locator('.journal-raw-entry').last().locator('[data-stat="play.cash"] strong')).toHaveText('+$18');
 expect((await checkpoint(page)).state.journal).toEqual(worked.journal);await snap(page,'journal-paid-shift-details');
 await day.locator(':scope > .journal-raw > summary').click();await snap(page,'journal-refresh');
 await page.getByRole('button',{name:'Route',exact:true}).click();await expect(page.locator('.map-heading-baseline .map-attribution')).toContainText('U.S. Census');await expect(page.locator('.map-heading-baseline .map-current-location')).toContainText('At Madison');if(info.project.name==='chromium'){const a=await page.locator('.map-attribution').boundingBox(),b=await page.locator('.map-current-location').boundingBox();expect(Math.abs(a!.y+a!.height-b!.y-b!.height)).toBeLessThan(5);}await snap(page,'map-header-credits');
});

test('legacy danger values are ignored and temporary risk uses the short percent label',async({page})=>{
 const gs=await baseline(page);gs.stats.pants=100;gs.budget_cents=3050;gs.current_order='WarDeptReorg';gs.exec_order_days_remaining=3;await importState(page,gs);
 await expect(page.locator('.policy-indicator')).toContainText('Breakdown risk +10%');await expect(page.locator('.policy-indicator')).not.toContainText('percentage points');await expect(page.locator('.critical-stats .hud-stat')).toHaveCount(7);await expect(page.locator('#main')).not.toContainText(/Pants|pants danger/);expect((await checkpoint(page)).state.stats.pants).toBeUndefined();expect((await checkpoint(page)).state.budget_cents).toBe(3100);await expect(page.locator('.conditions-hud .leg-cash')).not.toContainText('.00');
 await openMenu(page);const footer=page.locator('.offline-status');await expect(footer).toBeVisible();expect(await footer.evaluate(e=>e.parentElement?.lastElementChild===e)).toBe(true);await expect(footer.locator('.offline-readiness')).toHaveCSS('justify-content','center');await snap(page,'offline-centered-footer');
});
