import {test,expect,Page} from '@playwright/test';
import {baseline,importState,savedState,fastMode,snap} from './helpers';
import {atTown} from './geography';

const checkpoint=(page:Page)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);

test('every action in the turn stays on The Trail newest first through navigation and reload',async({page})=>{
 const gs=await baseline(page);atTown(gs,'Spokane');
 gs.stats={...gs.stats,supplies:8,sanity:8};gs.inventory.spares.battery=0;
 gs.encounter_cooldown=100;gs.crew_care.last_check_day=100;gs.weather_state.neutral_buffer=100;
 gs.journal[0].message='A previous turn stays in the journal.';
 gs.turn_journal_start=gs.journal.length;
 gs.journal.push({...structuredClone(gs.journal[0]),action_kind:'town',title:'Arrived at Spokane',message:'The crew arrives in Spokane.',before:{...gs.stats},after:{...gs.stats},resources:[],details:[]});
 await importState(page,gs);await fastMode(page,true);
 const trail=page.getByRole('tabpanel',{name:'The Trail',exact:true});
 await expect(trail.locator('.turn-receipt')).toHaveCount(1);
 await expect(trail).not.toContainText('A previous turn');
 await page.getByRole('button',{name:'Talk to locals',exact:true}).click();
 await page.getByRole('button',{name:'Back to town',exact:true}).click();
 await page.getByRole('button',{name:'Trade with locals',exact:true}).click();
 await page.getByRole('button',{name:'Get a battery',exact:true}).click();
 await page.getByRole('button',{name:'Back to town',exact:true}).click();
 await page.getByRole('button',{name:'Take a paid unloading shift',exact:true}).click();
 await expect(page.locator('.aftermath-panel')).toBeVisible();
 await page.getByRole('button',{name:'Back to town',exact:true}).click();
 const completed=await checkpoint(page),entries=completed.journal.slice(completed.turn_journal_start).reverse();
 expect(entries).toHaveLength(4);
 const checkEntries=async()=>{
  await expect(trail.locator('.turn-receipt')).toHaveCount(4);
  await expect(trail.locator('.receipt-heading')).toHaveText(entries.map((e:any)=>e.message));
  await expect(trail.locator('.turn-receipt').first().locator('[data-stat="play.cash"] strong')).toHaveText('+$18');
  await expect(trail.locator('.turn-receipt').nth(1).locator('[data-stat="store.items.battery.name"] small')).toHaveText('1 left');
  expect(await trail.locator('.receipt-mark .journey-icon').count()).toBe(4);
 };
 await checkEntries();
 await page.getByRole('tab',{name:'Journal',exact:true}).click();
 const days=[...new Set(completed.journal.map((entry:any)=>entry.day))];
 await expect(page.locator('.journal-day')).toHaveCount(days.length);
 await expect(page.locator('.journal-day > .journal-story .journal-event-heading').filter({hasText:'A previous turn stays in the journal.'})).toBeVisible();
 await expect(page.locator('.journal-raw-entry')).toHaveCount(completed.journal.length);
 const workDay=page.locator(`.journal-day[data-day="${completed.journal.at(-1).day}"]`);
 await workDay.locator(':scope > .journal-raw > summary').click();
 await expect(workDay.locator('.journal-raw-entry').last().locator('[data-stat="play.cash"] strong')).toHaveText('+$18');
 expect((await checkpoint(page)).journal).toEqual(completed.journal);
 await workDay.locator(':scope > .journal-raw > summary').click();
 await page.getByRole('tab',{name:'The Trail',exact:true}).click();await checkEntries();
 await page.reload();await checkEntries();
 const manual=await savedState(page);await importState(page,manual);await checkEntries();
 await page.getByRole('button',{name:'Talk to locals',exact:true}).click();
 await page.getByRole('button',{name:'Back to town',exact:true}).click();await checkEntries();
 expect((await checkpoint(page)).journal).toEqual(completed.journal);
 expect((await checkpoint(page)).stats).toEqual(completed.stats);
 await snap(page,'turn-history-newest-first');
 await page.getByRole('button',{name:'Leave town',exact:true}).click();
 await expect.poll(async()=>(await checkpoint(page)).journal.length).toBeGreaterThan(completed.journal.length);
 await page.getByRole('button',{name:'Pause travel',exact:true}).click();
 await expect(page.locator('#main')).not.toHaveAttribute('data-screen','traveling');
 const traveled=await checkpoint(page);
 expect(traveled.turn_journal_start).toBeGreaterThanOrEqual(completed.journal.length);
 expect(traveled.journal.slice(0,completed.journal.length)).toEqual(completed.journal);
 await expect(trail.locator('.turn-receipt')).toHaveCount(1);
 await expect(trail.locator('.receipt-heading')).not.toHaveText('');
 expect(traveled.journal.at(-1).title).toBe('Traveled');
 await expect(trail).not.toContainText('The crew arrives in Spokane.');
});

test('routine travel always has a visible title, including empty and transition-only messages',async({page})=>{
 const gs=await baseline(page);gs.turn_journal_start=0;
 const cases=[
  {title:'Traveled',message:'',expected:'Traveled'},
  {title:'Last action',message:'Traveled',expected:'Traveled'},
  {title:' \t ',message:'The next encounter is ready. You made progress along the route. Encounter!',expected:'Traveled'},
  {title:'Arrived at Spokane',message:'',expected:'Arrived at Spokane'},
  {title:'Traveled',message:'A wheel shakes loose.',expected:'A wheel shakes loose.'},
 ];
 gs.journal=cases.map(({title,message})=>({...structuredClone(gs.journal[0]),action_kind:'travel',title,message,before:{...gs.stats},after:{...gs.stats},resources:[],details:[]}));
 await importState(page,gs);
 const headings=page.locator('.turn-receipt .receipt-heading');
 await expect(headings).toHaveText(cases.toReversed().map(c=>c.expected));
 for(const heading of await headings.all())await expect(heading).toBeVisible();
 await page.reload();await expect(headings).toHaveText(cases.toReversed().map(c=>c.expected));
 expect((await checkpoint(page)).journal).toEqual(gs.journal);
 await snap(page,'trail-entry-titles');
});
