import {test,expect} from '@playwright/test';
import {baseline,importState,waitForLaunch,snap} from './helpers';

const current=(page:any)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
const orders=['Shutdown','TravelBanLite','BookPanic','TariffTsunami','DoEEliminated','WarDeptReorg'];
const keys=['shutdown','travel_ban_lite','book_panic','tariff_tsunami','doe_eliminated','war_dept_reorg'];

test('an actual travel activation holds the completed turn until acknowledged',async({page})=>{
 const base=await baseline(page);
 let found=false;
 for(let seed=0;seed<40&&!found;seed++){
  const gs=structuredClone(base);gs.seed=seed;gs.rng_bundle=null;
  gs.current_order=null;gs.exec_order_cooldown=0;gs.exec_order_days_remaining=0;
  gs.day_state.day_initialized=false;gs.encounter_cooldown=100;gs.encounter_chance_today=0;
  gs.weather_state.neutral_buffer=100;gs.stats.supplies=2;
  await importState(page,gs);
  await page.getByRole('button',{name:'Travel',exact:true}).click();
  await expect.poll(async()=>(await current(page)).journal.length).toBeGreaterThan(gs.journal.length);
  await expect(page.locator('#main')).not.toHaveAttribute('data-screen','traveling');
  const after=await current(page);
  if(!after.visual_content.policy_bulletins.length)continue;
  found=true;
  await expect(page.locator('.policy-bulletin')).toBeVisible();
  const pending=after.visual_content.policy_bulletins.find((n:any)=>!n.acknowledged);
  expect(pending).toBeTruthy();
  expect(after.logs).toContain(`exec.start.${keys[orders.indexOf(pending.order)]}`);
  await page.reload();await waitForLaunch(page);
  await expect(page.locator('.policy-bulletin')).toHaveAttribute('data-bulletin',pending.id);
  expect((await current(page)).journal).toEqual(after.journal);
  expect((await current(page)).rng_bundle).toEqual(after.rng_bundle);
  await page.locator('#bulletin-continue').click();
  await expect(page.locator('.policy-bulletin')).toHaveCount(0);
  await expect.poll(async()=> (await current(page)).visual_content.policy_bulletins.find((n:any)=>n.id===pending.id).acknowledged).toBe(true);
  await snap(page,'bulletin-travel-resumed');
 }
 expect(found,'Fixed-seed sweep must exercise a real engine activation').toBe(true);
});

test('a queued automatic route preview remains available after a bulletin',async({page})=>{
 const gs=await baseline(page);gs.day=6;gs.current_order='Shutdown';gs.exec_order_days_remaining=2;
 gs.logs.push('exec.start.shutdown');
 gs.visual_content ??= {edition:1};gs.visual_content.policy_bulletins=[{id:`policy/${gs.logs.length-1}`,order:'Shutdown',unit:'ORDER-SHUTDOWN-A',received_day:6,received_minute:gs.clock_minutes,acknowledged:false}];
 await importState(page,gs);
 await page.evaluate(()=>{
  const saved=JSON.parse(localStorage.getItem('dystrail.autosave.v1')!);
  saved.phase='Map';saved.map_automatic=true;saved.aftermath=null;
  localStorage.setItem('dystrail.autosave.v1',JSON.stringify(saved));
 });
 await page.reload();await waitForLaunch(page);
 await expect(page.locator('.policy-bulletin')).toBeVisible();
 await page.locator('#bulletin-continue').click();
 await expect(page.locator('#main')).toHaveAttribute('data-screen','map');
 await expect(page.locator('.map-countdown')).toBeVisible();
 expect((await current(page)).visual_content.policy_bulletins[0].acknowledged).toBe(true);
});
