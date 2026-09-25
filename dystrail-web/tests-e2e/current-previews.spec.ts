import {test,expect} from '@playwright/test';
import {baseline,importState,waitForLaunch} from './helpers';
import {atTown} from './geography';
import {mkdirSync,readFileSync} from 'node:fs';
import {resolve} from 'node:path';

test('capture current integrated visual-world previews',async({page,context},info)=>{
 test.setTimeout(90000);
 await page.setViewportSize({width:info.project.name==='mobile'?390:1440,height:1000});
 await page.emulateMedia({reducedMotion:'reduce'});
 const base=await baseline(page);base.seed=42;base.turn_journal_start=null;
 const copy=JSON.parse(readFileSync('i18n/en.json','utf8')).encounter_copy;
 const device=info.project.name==='mobile'?'mobile':'desktop';
 for(const id of ['road','night','town','ally','safe','overhead','fees','trade','pantry','cleanup','crossing','ending']){
  const s=structuredClone(base);s.clock_minutes=id==='night'?1380:720;
  if(['town','trade','pantry','cleanup'].includes(id)) atTown(s,'Spokane');
  if(id==='pantry'||id==='cleanup'){
   const family=id==='pantry'?'ACT-FOODWORK':'ACT-CASHWORK';
   s.activities={foraged_on:null,worked_at:null,local_word:null};
   s.visual_content={edition:1,selections:{[`${family}/town/${s.route_services.stop}`]:`${family}-A`},outcomes:{},policy_bulletins:[]};
  }
  if(id==='crossing'){
   s.party.members[1].status='Departed';s.party.members[2].status='Dead';
   s.crossing_events=[{day:s.day,region:s.region,season:s.season,kind:'checkpoint',permit_used:false,bribe_attempted:false,bribe_success:null,bribe_cost_cents:0,bribe_chance:null,bribe_roll:null,detour_reason:null,detour_taken:false,detour_hours:null,detour_base_supplies_delta:null,detour_extra_supplies_loss:null,terminal_threshold:0,terminal_roll:null,outcome:'passed'}];
   s.visual_content={edition:1,selections:{},outcomes:{},policy_bulletins:[],crossing_presentations:[{event_index:0,unit:'CROSS-02C-B',permit_receipt:false,acknowledged:false}]};
  }
  if(id==='ally'){
   s.day=6;s.stats.allies=0;
   s.visual_content={edition:1,selections:{'ALLY-01/departure/6':'ALLY-01-A'},outcomes:{},policy_bulletins:[]};
   const e=structuredClone(s.journal[0]);e.title=copy['ALLY-01-A'].name;
   e.message=copy['ALLY-01-A'].desc.replace('{name}','Ari')+' '+copy['ALLY-01-A'].last;
   e.before=structuredClone(s.stats);e.before.allies=1;e.after=structuredClone(s.stats);e.resources=[];e.details=[];
   s.ally_notice=e;s.journal.push(e);
  }
  if(['safe','overhead','fees'].includes(id)){
   const unit=id==='safe'?'ENC-C09-C':id==='overhead'?'ENC-C11-A':'ENC-C12-C';
   const source=JSON.parse(readFileSync('../review/recovery/current-source-records.json','utf8')).units.find((u:any)=>u.id===unit);
   s.current_encounter=JSON.parse(readFileSync('static/assets/data/game.json','utf8')).find((e:any)=>e.id===source.runtime_key);
   s.day=6;s.last_encounter_driving_minutes=300;s.driving_minutes_total=300;
   s.visual_content={edition:1,selections:{[`${unit.slice(0,-2)}/road/300`]:unit},outcomes:{},policy_bulletins:[]};
  }
  if(id==='trade'){
   const key=`ACT-BARTERTIRE/town/${s.route_services.stop}`;
   s.route_services.trading=true;s.route_services.traded_at=s.route_services.stop;
   s.visual_content={edition:1,selections:{[key]:'ACT-BARTERTIRE-C'},outcomes:{[key]:0},policy_bulletins:[]};
   s.stats.supplies=7;s.inventory.spares.tire=2;
  }
  if(id==='ending')s.ending={type:'collapse',cause:'hunger'};
  await importState(page,s);await page.reload();await waitForLaunch(page);
  if(id==='road'){
   await context.setOffline(true);await page.reload();await waitForLaunch(page);
   await expect(page.locator('.standing-member')).toHaveCount(6);
   const assets=await page.locator('.standing-member image').evaluateAll(async images=>Promise.all(images.map(async image=>{
    const href=image.getAttribute('href')!;const response=await fetch(href);return {href,ok:response.ok,bytes:(await response.blob()).size};
   })));
   expect(assets).toHaveLength(6);
   for(const asset of assets){expect(decodeURIComponent(asset.href)).toContain('/cast-v2/');expect(asset.ok).toBe(true);expect(asset.bytes).toBeGreaterThan(1000);}
   await context.setOffline(false);
  }
  if(id==='pantry'||id==='cleanup'){
   const unit=id==='pantry'?'ACT-FOODWORK-A':'ACT-CASHWORK-A';
   await page.getByRole('button',{name:copy[unit].choice_0,exact:true}).click();
   await expect(page.locator('[data-activity-unit]')).toHaveAttribute('data-activity-unit',unit);
  }
  if(id==='crossing')await expect(page.locator('.van-occupant')).toHaveCount(4);
  if(id==='overhead')await expect(page.locator('.scene-atlas[data-atlas="road-c11-a-20260914"]')).toHaveAttribute('data-cell','0');
  const target=page.locator(id==='ending'?'.result-art':'.world-view');
  await expect(target).toBeVisible();
  const out=resolve('../review/recovery/current-previews',device,id);mkdirSync(out,{recursive:true});
  await target.screenshot({path:resolve(out,'shot-0.png'),animations:'disabled'});
 }
});
