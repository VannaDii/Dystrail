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
  const unit=`ALLY-0${family}-${variant}`;const s=structuredClone(base);s.seed=42;s.stats.allies=remaining;s.day=family===1?6:family-1;
  s.visual_content={edition:1,selections:{[`ALLY-0${family}/departure/${s.day}`]:unit},outcomes:{},policy_bulletins:[]};
  const entry=structuredClone(s.journal[0]);entry.title=copy[unit].name;entry.message=copy[unit].desc.replace('{name}','Outside Contact')+(remaining===0?' '+copy[unit].last:'');
  entry.before=structuredClone(s.stats);entry.before.allies=remaining+1;entry.after=structuredClone(s.stats);entry.resources=[];entry.details=[];
  s.ally_notice=entry;s.journal.push(entry);
  await importState(page,s);await expect(page.locator('.ally-message')).toHaveText(entry.message);await expect(page.locator('#screen-title')).toContainText(copy[unit].name);
  await expect(page.getByText(entry.message,{exact:true})).toHaveCount(1);
  const read=()=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
  const before=await read();expect(before.party).toEqual(s.party);
  const illustrated=['ALLY-02-A','ALLY-02-C','ALLY-04-B','ALLY-05-A'].includes(unit);
  await expect(page.locator('[data-external-contact="true"]')).toHaveCount(illustrated?1:0);
  if(illustrated){
   await expect(page.locator('[data-ally-unit]')).toHaveAttribute('data-ally-unit',unit);
   await expect(page.locator('.journey-scene')).toHaveAttribute('data-indoors','true');
   await expect(page.locator('.journey-scene .standing-member')).toHaveCount(0);
   if(remaining===0){
    await context.setOffline(true);await page.reload();await waitForLaunch(page);
    await expect(page.locator('[data-ally-unit]')).toHaveAttribute('data-ally-unit',unit);
    await page.screenshot({path:info.outputPath(`${unit}.png`),fullPage:true});
    await context.setOffline(false);
   }
  }
  if(unit==='ALLY-06-C'&&remaining===0){await context.setOffline(true);await page.reload();await waitForLaunch(page);await expect(page.locator('.ally-message')).toHaveText(entry.message);await page.screenshot({path:info.outputPath('last-outside-ally.png'),fullPage:true});await context.setOffline(false);}
  await page.locator('#outcome-continue').click();await expect(page.locator('.ally-message')).toHaveCount(0);
  const after=await read();expect(after.ally_notice).toBeNull();before.ally_notice=null;before.inventory.tags.sort();after.inventory.tags.sort();expect(after).toEqual(before);
 }
});

test('localized ally notices preserve saved receipts and crew',async({page,context},info)=>{
 test.setTimeout(300000);const base=await baseline(page);
 const units=['ALLY-01-A','ALLY-02-A','ALLY-02-C','ALLY-03-B','ALLY-03-C','ALLY-04-B','ALLY-05-A','ALLY-05-B'];
 const languages=[...readFileSync('src/i18n/locales.rs','utf8').matchAll(/code: "([a-z]+)"/g)].map(m=>m[1]).filter(l=>l!=='en');
 for(const [index,lang] of languages.entries()){
  const translations=JSON.parse(readFileSync(`i18n/${lang}.json`,'utf8')).encounter_copy;
  const retained=JSON.parse(readFileSync(`../review/recovery/recorded-files/review/art-satire/ally-translation-${lang}.json`,'utf8'));
  for(const unit of units){
   const [,family,variant]=unit.split('-'),row=retained.rows[(Number(family)-1)*3+variant.charCodeAt(0)-65];
   expect(translations[unit]).toEqual({name:row[0],desc:row[1],last:retained.final[variant]});
  }
  const unit=units[index%units.length],family=Number(unit.split('-')[1]),text=translations[unit];
  for(const remaining of [0,1]){
   await page.evaluate(()=>localStorage.setItem('dystrail.locale','en'));await page.reload();await waitForLaunch(page);
   const s=structuredClone(base);s.seed=42;s.stats.allies=remaining;s.day=family===1?6:family-1;
   s.visual_content={edition:1,selections:{[`ALLY-0${family}/departure/${s.day}`]:unit},outcomes:{},policy_bulletins:[]};
   const entry=structuredClone(s.journal[0]);entry.title=text.name;entry.message=text.desc.replace('{name}','Ari')+(remaining===0?' '+text.last:'');
   entry.before=structuredClone(s.stats);entry.before.allies=remaining+1;entry.after=structuredClone(s.stats);entry.resources=[];entry.details=[];
   s.ally_notice=entry;s.journal.push(entry);await importState(page,s);
   await page.evaluate(l=>localStorage.setItem('dystrail.locale',l),lang);await page.reload();await waitForLaunch(page);
   await expect(page.locator('.ally-message')).toHaveText(entry.message);await expect(page.locator('#screen-title')).toContainText(text.name);
   expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
   const read=()=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
   const before=await read();expect(before.party).toEqual(s.party);
   await context.setOffline(true);await page.reload();await waitForLaunch(page);
   await expect(page.locator('.ally-message')).toHaveText(entry.message);
   if(remaining===0&&['ar','de','ja'].includes(lang))await page.screenshot({path:info.outputPath(`ally-${lang}.png`),fullPage:true});
   await page.locator('#outcome-continue').click();await expect(page.locator('.ally-message')).toHaveCount(0);
   const after=await read();before.ally_notice=null;before.inventory.tags.sort();after.inventory.tags.sort();expect(after).toEqual(before);
   await context.setOffline(false);
  }
 }
});
