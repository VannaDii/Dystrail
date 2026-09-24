import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {baseline,importState,waitForLaunch} from './helpers';
const copy=JSON.parse(readFileSync('i18n/en.json','utf8')).encounter_copy;
const units=JSON.parse(readFileSync('../review/recovery/current-source-records.json','utf8')).units.filter((u:any)=>u.id.startsWith('COND-'));
test('all 27 condition records match approved onset, continuing and relief copy',()=>{
 expect(units).toHaveLength(27);
 for(const u of units){const p=u.source_paragraphs;expect(copy[u.id]).toEqual({name:u.title,desc:p[2],continuing:p.find((x:string)=>x.startsWith('Continuing: ')).slice(12),relief:p.find((x:string)=>x.startsWith('Recovery / relief: ')).slice(19)});}
});
test('condition journal wording renders without duplicate resource costs',async({page,context},info)=>{
 test.setTimeout(120000);const base=await baseline(page);
 expect(base.journal).toHaveLength(1);
 for(const u of units){
  const s=structuredClone(base);s.seed=42;s.turn_journal_start=0;
  s.journal=['desc','continuing','relief'].map(field=>({...structuredClone(base.journal[0]),title:copy[u.id].name,message:copy[u.id][field],before:s.stats,after:s.stats,resources:[],details:[]}));
  await importState(page,s);
  for(const field of ['desc','continuing','relief']) await expect(page.locator('.turn-receipts').getByText(copy[u.id][field],{exact:true})).toBeVisible();
  await expect(page.locator('.turn-receipts [data-stat]')).toHaveCount(0);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
 }
 const s=structuredClone(base);s.seed=42;s.stats.supplies=0;s.day=6;s.clock_minutes=600;s.turn_journal_start=null;
 s.activities={foraged_on:null,worked_at:null,local_word:null};
 s.visual_content={edition:1,selections:{'ACT-FORAGE/day/6':'ACT-FORAGE-A','COND-HUNGER/condition/0':'COND-HUNGER-C'},outcomes:{},policy_bulletins:[]};
 await importState(page,s);await page.getByRole('button',{name:'Camp',exact:true}).click();await page.getByRole('button',{name:copy['ACT-FORAGE-A'].choice_0,exact:true}).click();
 const read=()=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
 const after=await read();expect(after.stats.supplies).toBe(2);
 const relief=after.journal.filter((e:any)=>e.message===copy['COND-HUNGER-C'].relief);expect(relief).toHaveLength(1);expect(relief[0].before).toEqual(relief[0].after);expect(relief[0].resources).toEqual([]);
 await page.locator('#outcome-continue').click();await expect(page.getByText(copy['COND-HUNGER-C'].relief,{exact:true})).toBeVisible();
 await context.setOffline(true);await page.reload();await waitForLaunch(page);expect((await read()).journal).toEqual(after.journal);expect((await read()).stats).toEqual(after.stats);
 await page.evaluate(()=>window.scrollTo({top:0,behavior:'instant'}));await page.screenshot({path:info.outputPath('hunger-relief.png'),fullPage:true});await context.setOffline(false);
});
