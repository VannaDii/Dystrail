import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {baseline,importState,savedState,snap,waitForLaunch} from './helpers';

const orders=['Shutdown','TravelBanLite','BookPanic','TariffTsunami','DoEEliminated','WarDeptReorg'];
const families=['ORDER-SHUTDOWN','ORDER-MILITARIZE','ORDER-GAG','ORDER-TARIFFS','ORDER-TAXCUTS','ORDER-DEREGULATE'];
const keys=['shutdown','travel_ban_lite','book_panic','tariff_tsunami','doe_eliminated','war_dept_reorg'];
const current=(page:any)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
const copy=(lang:string)=>JSON.parse(readFileSync(`i18n/${lang}.json`,'utf8'));

test('each policy variant has matching localized art, copy, source and saved state',async({page,context})=>{
 test.setTimeout(240000);
 const base=await baseline(page);
 for(const variant of ['B','C'])for(let i=0;i<orders.length;i++){
  const unit=`${families[i]}-${variant}`;
  const gs=structuredClone(base);gs.current_order=null;gs.exec_order_days_remaining=0;
  gs.logs.push(`exec.start.${keys[i]}`,`exec.end.${keys[i]}`);
  gs.visual_content.policy_bulletins=[{id:`policy/${gs.logs.length-2}`,order:orders[i],unit,received_day:gs.day,received_minute:gs.clock_minutes,acknowledged:false}];
  gs.crew_care.pending='staffer';gs.crew_care.reason=0;gs.crew_care.strain={staffer:1};
  await importState(page,gs);
  const before=await current(page);
  for(const lang of ['en','es','it','ar']){
   const expected=copy(lang).visual_copy[unit];
   await page.evaluate(lang=>localStorage.setItem('dystrail.locale',lang),lang);await page.reload();await waitForLaunch(page);
   await expect(page.locator('.policy-bulletin')).toHaveAttribute('data-unit',unit);
   await expect(page.locator('#bulletin-title')).toHaveText(expected.title);
   await expect(page.locator('.bulletin-copy')).toHaveText(expected.activation);
   await expect(page.locator('.bulletin-expired')).toContainText(expected.expiration);
   await expect(page.locator('.bulletin-illustration svg')).toHaveAttribute('data-atlas',`policy-bulletins-${variant.toLowerCase()}`);
   await expect(page.locator('.bulletin-illustration svg')).toHaveAttribute('data-cell',String(i));
   await expect(page.locator('.bulletin-illustration figcaption span')).toHaveText(Object.values(expected.overlays) as string[]);
   expect(await page.locator('.policy-bulletin').evaluate(el=>el.scrollWidth<=el.clientWidth+1)).toBe(true);
   await snap(page,`policy-${unit}-${lang}`);
  }
  await page.evaluate(()=>localStorage.setItem('dystrail.locale','en'));await page.reload();await waitForLaunch(page);
  // The two changed political hooks must follow the selected unit.
  if(unit==='ORDER-GAG-B'||unit==='ORDER-SHUTDOWN-C'){
   const sources=JSON.parse(readFileSync('static/assets/data/visual-sources.json','utf8'));
   await page.locator('.bulletin-body .context-help button').first().click();
   await expect(page.locator(`a[href="${sources[unit].source}"]`)).toBeVisible();
   await page.keyboard.press('Escape');
  }
  // Manual save/import and offline reload retain the complete selected scene.
  await importState(page,await savedState(page));
  await context.setOffline(true);await page.reload();await waitForLaunch(page);
  await expect(page.locator('.policy-bulletin')).toHaveAttribute('data-unit',unit);
  await page.locator('#bulletin-continue').click();
  await expect(page.locator('#main')).toHaveAttribute('data-screen','crew-care');
  const after=await current(page);expect(after.stats).toEqual(before.stats);expect(after.rng_bundle).toEqual(before.rng_bundle);
  expect(after.visual_content.policy_bulletins[0].acknowledged).toBe(true);
  await context.setOffline(false);
  await page.evaluate(()=>localStorage.setItem('dystrail.locale','en'));await page.reload();await waitForLaunch(page);
 }
});

test('unknown imported policy units use safe A content and unrelated languages use English narrative',async({page})=>{
 const gs=await baseline(page);gs.logs.push('exec.start.shutdown');gs.current_order='Shutdown';gs.exec_order_days_remaining=2;
 gs.visual_content.policy_bulletins=[{id:`policy/${gs.logs.length-1}`,order:'Shutdown',unit:'ORDER-GAG-B',received_day:gs.day,received_minute:gs.clock_minutes,acknowledged:false}];
 await importState(page,gs);
 await expect(page.locator('.policy-bulletin')).toHaveAttribute('data-unit','ORDER-SHUTDOWN-A');
 await expect(page.locator('#bulletin-title')).toHaveText(copy('en').workshop['ORDER-SHUTDOWN'].title);
 gs.visual_content.policy_bulletins[0].unit='ORDER-SHUTDOWN-B';await importState(page,gs);
 await page.evaluate(()=>localStorage.setItem('dystrail.locale','de'));await page.reload();await waitForLaunch(page);
 await expect(page.locator('.bulletin-copy')).toHaveText(copy('en').visual_copy['ORDER-SHUTDOWN-B'].activation);
 await expect(page.locator('.bulletin-illustration figcaption')).not.toContainText('visual_copy.');
});
