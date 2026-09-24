import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {baseline,importState,waitForLaunch} from './helpers';
const copy=JSON.parse(readFileSync('i18n/en.json','utf8')).encounter_copy;
test('all ally vignettes match the approved source',()=>{
 const units=JSON.parse(readFileSync('../review/recovery/current-source-records.json','utf8')).units.filter((u:any)=>u.category==='Ally-departure vignettes');
 expect(units).toHaveLength(18);
 for(const unit of units){expect(copy[unit.id].name).toBe(unit.title);expect(copy[unit.id].desc).toBe(unit.source_paragraphs[1]);expect(copy[unit.id].last).toBe(unit.source_paragraphs[2].split(': ')[1]);}
});
test('ally notices preserve the traveling crew and acknowledge only once',async({page,context},info)=>{
 test.setTimeout(180000);const base=await baseline(page);
 for(let family=1;family<=6;family++)for(const variant of ['A','B','C'])for(const remaining of [0,1]){
  const unit=`ALLY-0${family}-${variant}`;const s=structuredClone(base);s.seed=42;s.stats.allies=remaining;
  const entry=structuredClone(s.journal[0]);entry.title=copy[unit].name;entry.message=copy[unit].desc.replace('{name}','Outside Contact')+(remaining===0?' '+copy[unit].last:'');
  entry.before=structuredClone(s.stats);entry.before.allies=remaining+1;entry.after=structuredClone(s.stats);entry.resources=[];entry.details=[];
  s.ally_notice=entry;s.journal.push(entry);
  await importState(page,s);await expect(page.locator('.ally-message')).toHaveText(entry.message);await expect(page.locator('#screen-title')).toContainText(copy[unit].name);
  await expect(page.getByText(entry.message,{exact:true})).toHaveCount(1);
  const read=()=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
  const before=await read();expect(before.party).toEqual(s.party);
  if(unit==='ALLY-06-C'&&remaining===0){await context.setOffline(true);await page.reload();await waitForLaunch(page);await expect(page.locator('.ally-message')).toHaveText(entry.message);await page.screenshot({path:info.outputPath('last-outside-ally.png'),fullPage:true});await context.setOffline(false);}
  await page.locator('#outcome-continue').click();await expect(page.locator('.ally-message')).toHaveCount(0);
  const after=await read();expect(after.ally_notice).toBeNull();before.ally_notice=null;before.inventory.tags.sort();after.inventory.tags.sort();expect(after).toEqual(before);
 }
});
