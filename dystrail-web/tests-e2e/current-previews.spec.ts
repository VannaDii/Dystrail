import {test,expect} from '@playwright/test';
import {baseline,importState,waitForLaunch} from './helpers';
import {atTown} from './geography';
import {mkdirSync,readFileSync} from 'node:fs';
import {resolve} from 'node:path';

test('capture current integrated visual-world previews',async({page},info)=>{
 test.setTimeout(90000);
 await page.setViewportSize({width:info.project.name==='mobile'?390:1440,height:1000});
 await page.emulateMedia({reducedMotion:'reduce'});
 const base=await baseline(page);base.seed=42;base.turn_journal_start=null;
 const copy=JSON.parse(readFileSync('i18n/en.json','utf8')).encounter_copy;
 const device=info.project.name==='mobile'?'mobile':'desktop';
 for(const id of ['road','night','town','ally','trade','ending']){
  const s=structuredClone(base);s.clock_minutes=id==='night'?1380:720;
  if(id==='town'||id==='trade') atTown(s,'Spokane');
  if(id==='ally'){
   s.day=1;s.stats.allies=0;
   s.visual_content={edition:1,selections:{'ALLY-02/departure/1':'ALLY-02-A'},outcomes:{},policy_bulletins:[]};
   const e=structuredClone(s.journal[0]);e.title=copy['ALLY-02-A'].name;
   e.message=copy['ALLY-02-A'].desc.replace('{name}','Ari')+' '+copy['ALLY-02-A'].last;
   e.before=structuredClone(s.stats);e.before.allies=1;e.after=structuredClone(s.stats);e.resources=[];e.details=[];
   s.ally_notice=e;s.journal.push(e);
  }
  if(id==='trade'){
   const key=`ACT-BARTERTIRE/town/${s.route_services.stop}`;
   s.route_services.trading=true;s.route_services.traded_at=s.route_services.stop;
   s.visual_content={edition:1,selections:{[key]:'ACT-BARTERTIRE-C'},outcomes:{[key]:0},policy_bulletins:[]};
   s.stats.supplies=7;s.inventory.spares.tire=2;
  }
  if(id==='ending')s.ending={type:'collapse',cause:'hunger'};
  await importState(page,s);await page.reload();await waitForLaunch(page);
  const target=page.locator(id==='ending'?'.result-art':'.world-view');
  await expect(target).toBeVisible();
  const out=resolve('../review/recovery/current-previews',device,id);mkdirSync(out,{recursive:true});
  await target.screenshot({path:resolve(out,'shot-0.png'),animations:'disabled'});
 }
});
