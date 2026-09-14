import {test,expect,Page} from '@playwright/test';
import { baseline,importState,fastMode,snap, waitForLaunch } from './helpers';

const saved=(page:Page)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
const cardSizes=(page:Page,selector:string)=>page.locator(selector+' .stat-card').evaluateAll(es=>es.map(e=>{
 const r=e.getBoundingClientRect();return {width:r.width,height:r.height,top:r.top};
}));
async function containedCards(page:Page,selector:string){
 const fits=await page.locator(selector).evaluate(e=>{
  const r=e.getBoundingClientRect();return r.left>=0&&r.right<=innerWidth&&[...e.querySelectorAll('.stat-card')].every(c=>{const b=c.getBoundingClientRect();return b.left>=r.left&&b.right<=r.right&&b.top>=r.top&&b.bottom<=r.bottom&&b.width>100;});
 });
 const layout=await page.locator(selector).evaluate(e=>[e,...e.querySelectorAll('.journal-story,.resource-changes,.stat-card')].map(n=>{const r=n.getBoundingClientRect(),s=getComputedStyle(n);return {class:n.className,width:r.width,left:r.left,display:s.display,wrap:s.flexWrap,min:s.minWidth,max:s.maxWidth,flex:s.flex};}));
 expect(fits,JSON.stringify(layout)).toBe(true);
}

test('Trail and Journal share compact right-hand cards and retain historical journey settings',async({page,isMobile})=>{
 const gs=await baseline(page);gs.turn_journal_start=0;
 gs.journal=[{
  day:74,minute:480,pace:'heated',diet:'quiet',place:'Salt Lake City → Denver',
  title:'A day on the trail',message:'The crew reaches the next stretch of road.',action_kind:'travel',
  before:{...gs.stats,supplies:18,sanity:10},after:{...gs.stats,supplies:17,sanity:9},
  resources:[{key:'play.elapsed',before:73,after:74},{key:'play.miles',before:11054,after:11253},{key:'play.vehicle',before:9976,after:9967}],
  details:[]
 }];
 await importState(page,gs);
 const trail=page.locator('.turn-receipt');await expect(trail.locator('.stat-card')).toHaveCount(5);
 const trailCards=await cardSizes(page,'.turn-receipt');
 await containedCards(page,'.turn-receipt');
 if(!isMobile){
  expect(new Set(trailCards.map(r=>Math.round(r.top))).size).toBe(1);
  const text=await trail.locator('.receipt-narrative').boundingBox(),stats=await trail.locator('.resource-changes').boundingBox();
  expect(stats!.x).toBeGreaterThan(text!.x+text!.width);expect((await trail.boundingBox())!.height).toBeLessThan(160);
 }
 await snap(page,'compact-trail-cards');
 await containedCards(page,'.turn-receipt');
 await trail.screenshot({path:`test-results/compact-trail-entry-${test.info().project.name}.png`});
 await page.getByRole('tab',{name:'Journal',exact:true}).click();
 expect(await page.locator('.journal').evaluate(e=>getComputedStyle(e).borderTopWidth)).toBe('0px');
 expect(await page.getByRole('tab',{name:'Journal',exact:true}).evaluate(e=>parseFloat(getComputedStyle(e).borderTopLeftRadius))).toBeGreaterThan(0);
 const entry=page.locator('.journal-day > .journal-story'),journalCards=await cardSizes(page,'.journal-day > .journal-story');
 await containedCards(page,'.journal-day > .journal-story');
 expect(journalCards.map(({width,height})=>({width,height}))).toEqual(trailCards.map(({width,height})=>({width,height})));
 if(!isMobile)expect(new Set(journalCards.map(r=>Math.round(r.top))).size).toBe(1);
 await expect(entry.getByRole('heading',{name:'Day 74',exact:true})).toBeVisible();
 await expect(entry.locator('.journal-context')).toHaveText('08:00HeatedQuiet');
 await expect(entry.locator('.journal-context .journey-icon')).toHaveCount(3);
 const place=await entry.locator('.journal-place').boundingBox(),context=await entry.locator('.journal-context').boundingBox();
 expect(context!.y).toBeGreaterThan(place!.y+place!.height);expect(context!.x).toBe(place!.x);
 await snap(page,'compact-journal-cards');
 await page.getByRole('tab',{name:'Conditions',exact:true}).click();
 await page.locator('.setting-option:has([data-icon=blitz])').click();await page.locator('.setting-option:has([data-icon=doom])').click();
 await page.getByRole('tab',{name:'Journal',exact:true}).click();
 await expect(entry.locator('.journal-context')).toHaveText('08:00HeatedQuiet');
 expect((await saved(page)).journal).toEqual(gs.journal);
 await page.reload();await waitForLaunch(page);
 await expect(entry.locator('.journal-context')).toHaveText('08:00HeatedQuiet');
});

test('the daily Journal accumulates actual first-final totals and preserves every raw action across days',async({page})=>{
 const gs=await baseline(page),initial={...gs.stats,supplies:0,sanity:8,morale:5};
 const change=(key:string,before:number,after:number)=>({key,before,after});
 const action=(day:number,minute:number,kind:string,title:string,message:string,before:any,after:any,resources:any[]=[],details:any[]=[])=>({day,minute,pace:'steady',diet:'mixed',place:'Salt Lake City → Denver',action_kind:kind,title,message,before,after,resources,details});
 const packed={...initial,supplies:10},first={...packed,supplies:9,sanity:7},second={...first,supplies:8,sanity:6},cared={...second,supplies:6,morale:6};
 const older=action(73,780,'camp','Camp','The crew rests.',initial,initial);
 const purchase=action(74,480,'town','Supply purchase','The supplies are packed.',initial,packed,[change('play.cash',12000,10000)]);
 const travel1=action(74,540,'travel','Traveled','',packed,first,[change('play.miles',0,600),change('play.driving_time',0,60),change('play.vehicle',10000,9990)]);
 const travel2=action(74,600,'travel','Traveled','',first,second,[change('play.miles',600,1200),change('play.driving_time',60,120),change('play.vehicle',9990,9980)]);
 const care=action(74,630,'care','Care for Sam','Sam receives care and can help the crew again.',second,cared,[],[['Sam','Traveling']]);
 gs.day=74;gs.clock_minutes=630;gs.stats=cared;gs.pace='blitz';gs.diet='doom';gs.turn_journal_start=1;gs.journal=[older,purchase,travel1,travel2,care];
 const showJournal=async()=>{const tab=page.getByRole('tab',{name:'Journal',exact:true});if(await tab.getAttribute('aria-selected')!=='true')await tab.click();};
 await importState(page,gs);await showJournal();
 const days=page.locator('.journal-day'),day=page.locator('.journal-day[data-day="74"]'),summary=day.locator(':scope > .journal-story');
 await expect(days).toHaveCount(2);await expect(days.first()).toHaveAttribute('data-day','74');
 await expect(summary.locator('.journal-event')).toHaveCount(3);await expect(summary.locator('.journal-event[data-action-count="2"]')).toContainText('Traveled');
 await expect(summary.locator('[data-stat="ux.supplies"] strong')).toHaveText('+6');await expect(summary.locator('[data-stat="ux.supplies"] small')).toHaveText('6 left');
 await expect(summary.locator('[data-stat="ux.sanity"] strong')).toHaveText('-2');await expect(summary.locator('[data-stat="play.cash"] strong')).toHaveText('−$20');
 await expect(summary.locator('[data-stat="play.miles"] strong')).toHaveText('+120.0');await expect(summary.locator('[data-stat="play.miles"] small')).toHaveText('2 h driving');
 await expect(summary.locator('[data-stat="play.vehicle"] strong')).toHaveText('-0.20%');await expect(summary).toContainText('SamTraveling');
 await expect(summary.locator('.journal-context')).toHaveText('10:30SteadyMixed');
 await day.locator(':scope > .journal-raw > summary').click();
 await expect(day.locator('.journal-raw-entry')).toHaveCount(4);
 await expect(day.locator('.journal-raw-entry [data-stat="play.miles"] strong')).toHaveText(['+60.0','+60.0']);
 await expect(day.locator('.journal-raw-entry [data-stat="ux.supplies"] strong')).toHaveText(['+10','-1','-1','-2']);
 expect((await saved(page)).journal).toEqual(gs.journal);await day.locator(':scope > .journal-raw > summary').click();
 await containedCards(page,'.journal-day[data-day="74"] > .journal-story');await snap(page,'daily-journal-totals');

 const foraged={...cared,supplies:8,sanity:7};
 const forage={...action(74,750,'camp','Forage','The crew gathers fresh supplies.',cared,foraged),pace:'heated',diet:'quiet'};
 gs.journal.push(forage);gs.stats=foraged;gs.clock_minutes=750;
 await importState(page,gs);await showJournal();await expect(days).toHaveCount(2);
 await expect(summary.locator('.journal-event')).toHaveCount(4);await expect(summary.locator('[data-stat="ux.supplies"] strong')).toHaveText('+8');await expect(summary.locator('[data-stat="ux.supplies"] small')).toHaveText('8 left');
 await expect(summary.locator('[data-stat="ux.sanity"] strong')).toHaveText('-1');await expect(summary.locator('[data-stat="play.miles"] strong')).toHaveText('+120.0');
 await expect(summary.locator('.journal-event-context [data-icon=heated]')).toHaveCount(1);await expect(summary.locator('.journal-event-context [data-icon=quiet]')).toHaveCount(1);

 const next=action(75,540,'travel','Traveled','',foraged,foraged,[change('play.miles',1200,1800),change('play.driving_time',120,180)]);
 gs.journal.push(next);gs.day=75;gs.clock_minutes=540;
 await importState(page,gs);await showJournal();await expect(days).toHaveCount(3);await expect(days.first()).toHaveAttribute('data-day','75');
 await expect(page.locator('.journal-day[data-day="75"] > .journal-story [data-stat="play.miles"] strong')).toHaveText('+60.0');
 await expect(summary.locator('[data-stat="play.miles"] strong')).toHaveText('+120.0');await expect(summary.locator('[data-stat="ux.supplies"] strong')).toHaveText('+8');
 await expect(page.locator('.journal-raw-entry')).toHaveCount(gs.journal.length);
 await page.reload();await waitForLaunch(page);await expect(days).toHaveCount(3);await expect(summary.locator('[data-stat="ux.supplies"] small')).toHaveText('8 left');
 expect((await saved(page)).journal).toEqual(gs.journal);expect((await saved(page)).turn_journal_start).toBe(1);await snap(page,'daily-journal-next-day');
});

test('real driving rollover keeps all five hours and their daily effects in the completed day Journal',async({page})=>{
 test.setTimeout(60000);
 const gs=await baseline(page);
 // Keep the real travel clock and daily accounting, while isolating road hours
 // from scheduled towns, encounters, breakdowns and changing weather.
 gs.route_services.route_id='uninterrupted-test-road';
 gs.day_state.day_initialized=true;
 gs.weather_state.today='Clear';gs.weather_state.neutral_buffer=100;
 gs.weather_travel_multiplier=1;gs.exec_travel_multiplier=1;
 gs.exec_breakdown_bonus=0;gs.illness_travel_penalty=1;
 gs.vehicle.breakdown_cooldown=100;gs.encounter_cooldown=100;gs.crew_care.last_check_day=100;
 gs.last_encounter_driving_minutes=600;
 gs.stats.hp=10;gs.stats.sanity=10;gs.stats.supplies=16;gs.pace='steady';gs.driving_minutes_total=0;
 expect(gs.clock_minutes).toBe(480);
 await importState(page,gs);await fastMode(page,true);
 let after=gs;
 for(let hour=1;hour<=5;hour++){
  const before=await saved(page);
  await page.getByRole('button',{name:'Travel',exact:true}).click();
  await page.waitForFunction(length=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state.journal.length>length,before.journal.length);
  await page.getByRole('button',{name:'Pause travel',exact:true}).click();
  await expect(page.locator('#main')).toHaveAttribute('data-screen','travel');
  after=await saved(page);
  expect(after.driving_minutes_total-before.driving_minutes_total).toBe(60);
  expect(after.miles_traveled_actual-gs.miles_traveled_actual).toBeCloseTo(hour*60,4);
 }
 expect(after.day).toBe(gs.day+1);expect(after.clock_minutes).toBe(480);expect(after.driving_minutes_total).toBe(300);
 const driving=after.journal.slice(gs.journal.length);
 expect(driving).toHaveLength(5);expect(driving.map((entry:any)=>entry.action_kind)).toEqual(Array(5).fill('travel'));
 await page.getByRole('tab',{name:'Journal',exact:true}).click();
 await snap(page,'daily-journal-real-rollover');
 await test.info().attach('real-rollover-accounting',{body:JSON.stringify({before:{day:gs.day,minute:gs.clock_minutes,miles:gs.miles_traveled_actual},after:{day:after.day,minute:after.clock_minutes,miles:after.miles_traveled_actual},driving},null,2),contentType:'application/json'});
 expect.soft(driving.map((entry:any)=>entry.day),'Each driving record belongs to the day in which its hour was driven.').toEqual(Array(5).fill(gs.day));
 expect.soft(driving.map((entry:any)=>entry.minute),'The completed final hour is recorded at 13:00, before the overnight reset.').toEqual([540,600,660,720,780]);
 await expect.soft(page.locator('.journal-day')).toHaveCount(1);
 const daily=page.locator(`.journal-day[data-day="${gs.day}"] > .journal-story`);
 await expect.soft(daily.locator('[data-stat="play.miles"] strong')).toHaveText('+300.0');
 await expect.soft(daily.locator('[data-stat="play.miles"] small')).toHaveText('5 h driving');
 await expect.soft(daily.locator('.journal-context')).toContainText('13:00');
 const first=after.journal[0],last=driving.at(-1),suppliesDelta=last.after.supplies-first.before.supplies;
 const supplies=daily.locator('[data-stat="ux.supplies"]');
 if(suppliesDelta===0)await expect.soft(supplies).toHaveCount(0);
 else {
  await expect.soft(supplies.locator('strong')).toHaveText(`${suppliesDelta>0?'+':''}${suppliesDelta}`);
  await expect.soft(supplies.locator('small')).toHaveText(`${last.after.supplies} left`);
 }
 expect((await saved(page)).journal).toEqual(after.journal);
});
