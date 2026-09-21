import {test,expect} from '@playwright/test';
import {baseline,importState,savedState,snap,waitForLaunch} from './helpers';

const orders=['Shutdown','TravelBanLite','BookPanic','TariffTsunami','DoEEliminated','WarDeptReorg'];
const families=['ORDER-SHUTDOWN','ORDER-MILITARIZE','ORDER-GAG','ORDER-TARIFFS','ORDER-TAXCUTS','ORDER-DEREGULATE'];
const keys=['shutdown','travel_ban_lite','book_panic','tariff_tsunami','doe_eliminated','war_dept_reorg'];
function bulletin(base:any,index:number){
 const gs=structuredClone(base);gs.current_order=orders[index];gs.exec_order_days_remaining=2;
 gs.logs.push(`exec.start.${keys[index]}`);
 gs.visual_content.policy_bulletins=[{id:`policy/${gs.logs.length-1}`,order:orders[index],unit:`${families[index]}-A`,received_day:gs.day,received_minute:gs.clock_minutes,acknowledged:false}];
 return gs;
}
const current=(page:any)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);

test('all policy illustrations and localized impact readouts hold the pending action',async({page})=>{
 const base=await baseline(page);
 for(let i=0;i<orders.length;i++){
  const gs=bulletin(base,i);gs.crew_care.pending='staffer';gs.crew_care.reason=0;gs.crew_care.strain={staffer:1};
  await importState(page,gs);
  await expect(page.locator('#main')).toHaveAttribute('data-screen','policy-bulletin');
  await expect(page.locator('#bulletin-title')).toBeFocused();
  await expect(page.locator('.bulletin-illustration svg')).toHaveAttribute('data-cell',String(i));
  const before=await current(page);
  await expect(page.locator('.policy-bulletin>.conditions-hud .policy-indicator')).toBeVisible();
  for(const locale of ['en','es','it','ar']){
   await page.evaluate(locale=>localStorage.setItem('dystrail.locale',locale),locale);
   await page.reload();await waitForLaunch(page);
   await expect(page.locator('.bulletin-copy')).not.toContainText(/workshop\.|policy_bulletin\./);
   expect(await page.locator('.policy-bulletin').evaluate(el=>el.scrollWidth<=el.clientWidth+1)).toBe(true);
   expect((await current(page)).stats).toEqual(before.stats);
   await snap(page,`bulletin-${keys[i]}-${locale}`);
  }
  await page.locator('#bulletin-continue').click();
  await expect(page.locator('.policy-bulletin')).toHaveCount(0);
  await expect(page.locator('#main')).toHaveAttribute('data-screen','crew-care');
  const after=await current(page);expect(after.stats).toEqual(before.stats);expect(after.rng_bundle).toEqual(before.rng_bundle);
  expect(after.crew_care).toEqual(before.crew_care);expect(after.clock_minutes).toBe(before.clock_minutes);
  expect(after.visual_content.policy_bulletins[0].acknowledged).toBe(true);
  await page.evaluate(()=>localStorage.setItem('dystrail.locale','en'));await page.reload();await waitForLaunch(page);
 }
});

test('bulletin acknowledgement survives manual save, offline reload and import without charging twice',async({page,context})=>{
 const base=await baseline(page);const gs=bulletin(base,3);
 gs.crew_care.pending='staffer';gs.crew_care.reason=0;gs.crew_care.strain={staffer:1};
 await importState(page,gs);
 const pending=await savedState(page);await importState(page,pending);
 await expect(page.locator('.policy-bulletin')).toBeVisible();
 await context.setOffline(true);await page.reload();await waitForLaunch(page);
 await expect(page.locator('.policy-bulletin')).toBeVisible();
 const before=await current(page);
 await page.locator('#bulletin-continue').focus();await page.keyboard.press('Enter');
 await expect(page.locator('#main')).toHaveAttribute('data-screen','crew-care');
 await page.reload();await waitForLaunch(page);
 await expect(page.locator('.policy-bulletin')).toHaveCount(0);
 const acknowledged=await current(page);expect(acknowledged.stats).toEqual(before.stats);expect(acknowledged.rng_bundle).toEqual(before.rng_bundle);
 await context.setOffline(false);
 const saved=await savedState(page);await importState(page,saved);
 await expect(page.locator('.policy-bulletin')).toHaveCount(0);
 const legacy=structuredClone(base);legacy.current_order='Shutdown';legacy.exec_order_days_remaining=2;delete legacy.visual_content;
 await importState(page,legacy);await expect(page.locator('.policy-bulletin')).toHaveCount(0);
});

test('already expired bulletins explain their status and preserve queued encounters',async({page})=>{
 const base=await baseline(page);const gs=bulletin(base,0);gs.current_order=null;gs.exec_order_days_remaining=0;
 gs.crew_care.pending='staffer';gs.crew_care.reason=0;gs.crew_care.strain={staffer:1};
 await importState(page,gs);
 await expect(page.locator('.bulletin-expired')).toContainText('already ended');
 await expect(page.locator('.bulletin-body>.policy-details')).toHaveCount(0);
 await page.locator('#bulletin-continue').click();await expect(page.locator('#main')).toHaveAttribute('data-screen','crew-care');
});
